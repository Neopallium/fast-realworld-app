use log::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use tokio::time::delay_for;

use tokio_postgres::{
  connect, Client, Statement, Row, NoTls,
  types::ToSql,
};

use crate::error::*;

use super::{
  UserService,
  ArticleService,
};

const MAX_RETRIES: u32 = 10;

pub type RefClient = Rc<(u64, Client)>;

/// Client connected state
#[derive(Clone)]
pub enum ClientState {
  Disconnected(u64),
  Connecting(u64),
  Connected(RefClient),
}

/// Wraps a postgres client with a version number.
/// Each time the client reconnects a new version number is generated.
pub struct VersionedClient {
  state: ClientState,
}

impl VersionedClient {
  pub fn new() -> Self {
    Self {
      state: ClientState::Disconnected(0),
    }
  }

  pub fn get_state(&self) -> &ClientState {
    &self.state
  }

  pub fn set_state(&mut self, state: ClientState) {
    self.state = state;
  }
}

/// A postgres client shared with multiple DBServices.
/// Wraps a `VersionedClient`
#[derive(Clone)]
pub struct SharedClient {
  cl: Rc<RefCell<VersionedClient>>,
}

impl SharedClient {
  pub fn new(url: &str) -> Self {
    Self {
      cl: Rc::new(RefCell::new(VersionedClient::new())),
    }.start_client(url.to_string())
  }

  pub fn start_client(self, url: String) -> Self {
    let shared_cl = self.clone();
    actix_rt::spawn(async move {
      shared_cl.spawn_client(url).await;
      eprintln!("client background task stopped.");
    });
    self
  }

  async fn spawn_client(&self, url: String) {
    let mut version = 0;
    debug!("Spawned client background task: ver={}", version);
    loop {
      version += 1;
      debug!("client task: Connecting: ver={}", version);
      self.change_inner_state(ClientState::Connecting(version));
      // Setup tokio-postgres
      let (cl, conn) = loop {
        match connect(&url, NoTls).await {
          Ok((cl, conn)) => {
            debug!("client task: ver={}: connected.", version);
            break (cl, conn);
          },
          Err(e) => {
            debug!("client task: ver={}: connect error: {}", version, e);
            delay_for(Duration::from_millis(500)).await;
          },
        }
      };
      debug!("client task: ver={}: Connecting -> Connected", version);
      self.change_inner_state(ClientState::Connected(
        Rc::new((version, cl))
      ));
      // Process background connection.
      match conn.await {
        Err(e) => {
          debug!("tokio-postgres connection error: {}", e);
        },
        _ => {
          debug!("tokio-postgres connection closed.");
          return;
        },
      }
      debug!("client task: ver={}: Connected -> Connecting", version);
      // wait a little bit before trying to connect.
      delay_for(Duration::from_millis(500)).await;
    }
  }

  pub async fn get_client(&self) -> Result<RefClient> {
    let mut retries = 0u32;
    loop {
      match self.get_inner_state() {
        ClientState::Connected(cl) => return Ok(cl),
        ClientState::Connecting(version) => {
          debug!("get_client: ver={}: Connecting..", version);
          delay_for(Duration::from_millis(100)).await;
        },
        ClientState::Disconnected(version) => {
          debug!("get_client: ver={}: Disconnected -> Connecting", version);
          delay_for(Duration::from_millis(100)).await;
        },
      }
      retries += 1;
      if retries >= MAX_RETRIES {
        return Err(Error::DisconnectedError("Failed to connect to database".to_string()));
      }
    }
  }

  /// Check client version.
  pub fn check_version(&self, version: u64) -> bool {
    match self.cl.borrow().get_state() {
      ClientState::Connected(ref cl) => cl.0 == version,
      _ => false,
    }
  }

  /// get inner VersionedClient state.
  fn get_inner_state(&self) -> ClientState {
    self.cl.borrow().get_state().clone()
  }

  /// Mutate inner VersionedClient state.
  fn change_inner_state(&self, state: ClientState) {
    self.cl.borrow_mut().set_state(state)
  }
}

pub type RefClientStatement = Rc<ClientStatement>;
#[derive(Clone)]
pub struct ClientStatement {
  cl: RefClient,
  statement: Statement,
}

impl ClientStatement {
  pub fn get_version(&self) -> u64 {
    self.cl.0
  }

  pub fn get_cl_statement(&self) -> (&Client, &Statement) {
    (&self.cl.1, &self.statement)
  }
}

/// Prepare statement state
#[derive(Clone)]
enum StatementState {
  Init(u64),
  WaitingClient(u64),
  Preparing(u64),
  Prepared(RefClientStatement),
}

/// Wraps a postgres client with a version number.
/// Each time the client reconnects a new version number is generated.
#[derive(Clone)]
pub struct VersionedStatement {
  /// Shared Client, used for checking the version and reconnecting.
  shared_cl: SharedClient,

  /// Current version and statement state.
  state: RefCell<StatementState>,

  /// Statement query
  query: String,
}

macro_rules! impl_client_method {
  ($method:ident, $res_ty:ty) => {
    pub async fn $method(&self, params: &[&(dyn ToSql + Sync)]) -> Result<$res_ty> {
      let mut retries = 0;
      loop {
        let ref_statement = self.get_statement().await?;
        let (cl, statement) = ref_statement.get_cl_statement();

        match cl.$method(statement, params).await {
          Ok(res) => return Ok(res),
          Err(err) => {
            match err.code() {
              None => {
                // client-side error.
                match err.to_string().as_str() {
                  "connection closed" => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                      return Err(Error::DisconnectedError(
                        "Failed to connect to database".to_string()));
                    }
                    // connection to the DB was closed, try again.
                    info!("DB connection closed, retry query.");
                    delay_for(Duration::from_millis(100)).await;
                  },
                  msg => {
                    error!("Postgres error: {}, query=[[{}]]", msg, self.query);
                    return Err(err.into());
                  },
                }
              },
              Some(_) => {
                // Server-side error.
                error!("Postgres DB error: {:?}, query=[[{}]]", err, self.query);
                return Err(err.into());
              },
            }
          },
        }
      }
    }
  };
}

impl VersionedStatement {
  pub fn new(shared_cl: SharedClient, query: &str) -> Result<Self> {
    Ok(Self {
      shared_cl,
      state: RefCell::new(StatementState::Init(0)),
      query: query.to_string(),
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.get_statement().await?;
    Ok(())
  }

  pub async fn get_statement(&self) -> Result<RefClientStatement> {
    let mut retries = 0u32;
    loop {
      match self.get_state() {
        StatementState::Init(version) => {
          debug!("get_statement: ver={}: Init -> WaitingClient. Get client", version);
          self.set_state(StatementState::WaitingClient(version));
          match self.shared_cl.get_client().await {
            Ok(cl) => {
              let version = cl.0;
              debug!("get_statement: ver={}: WaitingClient -> Preparing. Got client", version);
              self.set_state(StatementState::Preparing(version));
              // Prepare statement
              match cl.1.prepare(&self.query).await {
                Ok(statement) => {
                  debug!("get_statement: ver={}: Preparing -> Prepared. Got statement", version);
                  self.set_state(StatementState::Prepared(
                    Rc::new(ClientStatement{
                      cl,
                      statement,
                    })
                  ));
                },
                Err(err) => {
                  match err.code() {
                    None => {
                      match err.to_string().as_str() {
                        "connection closed" => {
                          // retry connection.  Go back into Init state.
                          self.set_state(StatementState::Init(version));
                        },
                        msg => {
                          error!("Postgres error: {}, query=[[{}]]", msg, self.query);
                          return Err(err.into());
                        },
                      }
                    },
                    Some(_) => {
                      // Server-side error.
                      error!("Postgres DB error: {}, query=[[{}]]", err, self.query);
                      return Err(err.into());
                    },
                  }
                },
              }
            },
            Err(err) => {
              debug!("get_statement: ver={}: Init error: {:?}", version, err);
              // Failed to get client connection.  Go back into Init state.
              self.set_state(StatementState::Init(version));
              return Err(err);
            }
          }
        },
        StatementState::WaitingClient(version) => {
          debug!("get_statement: ver={}: WaitingClient..", version);
          delay_for(Duration::from_millis(100)).await;
        },
        StatementState::Preparing(version) => {
          debug!("get_statement: ver={}: Preparing..", version);
          delay_for(Duration::from_millis(100)).await;
        },
        StatementState::Prepared(cl_statement) => {
          let version = cl_statement.get_version();
          debug!("get_statement: ver={}: Prepared: check version", version);
          if self.shared_cl.check_version(version) {
            // version ok.
            return Ok(cl_statement);
          }
          // old version, need to reconnect, prepare statement.
          self.set_state(StatementState::Init(version));
        },
      }
      retries += 1;
      if retries >= MAX_RETRIES {
        return Err(Error::DisconnectedError("Failed to connect to database".to_string()));
      }
    }
  }

  fn get_state(&self) -> StatementState {
    self.state.borrow().clone()
  }

  fn set_state(&self, state: StatementState) {
    self.state.replace(state);
  }

  impl_client_method!(query, Vec<Row>);
  impl_client_method!(query_one, Row);
  impl_client_method!(query_opt, Option<Row>);
  impl_client_method!(execute, u64);
}

#[derive(Clone)]
pub struct DbService {
  pub shared_cl: SharedClient,
  pub user: UserService,
  pub article: ArticleService,
}

impl DbService {
  pub fn new(db_url: &str) -> Result<DbService> {
    let shared_cl = SharedClient::new(db_url);

    Ok(DbService {
      user: UserService::new(shared_cl.clone())?,
      article: ArticleService::new(shared_cl.clone())?,
      shared_cl: shared_cl,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    info!("DBService: Prepare UserService.");
    self.user.prepare().await?;
    info!("DBService: Prepare ArticleService.");
    self.article.prepare().await?;

    info!("DBService: finished.");
    Ok(())
  }
}
