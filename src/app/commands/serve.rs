use log::*;

use std::convert::TryInto;
use std::thread;
use futures::executor;

use crossbeam_channel::{
  bounded, Sender, Receiver,
};

use actix_rt::System;
use actix_web::{get, web, middleware, HttpResponse, App, HttpServer};

use crate::{
  error::*,
  app::*,
  db::DbService,
  services::config_services,
};

#[derive(Debug)]
enum StopEvent {
  Shutdown,
  StopServer,
  StopServerFinished(u32),
}

#[get("/stop")]
async fn stop_server(waiter: web::Data<ServerWaiter>) -> HttpResponse {
  info!("Got shutdown request.");
  waiter.main_shutdown();

  HttpResponse::Ok().body("Shutting down.")
}

#[derive(Clone)]
struct ServerStopper {
  id: u32,
  tx: Sender<StopEvent>,
}

#[derive(Clone)]
struct ServerWaiter {
  id: u32,
  main_tx: Sender<StopEvent>,
  rx: Receiver<StopEvent>,
}

impl ServerStopper {
  pub fn new(id: u32, main_tx: Sender<StopEvent>) -> (Self, ServerWaiter) {
    let (tx, rx) = bounded(1);
    (Self{
      id,
      tx,
    }, ServerWaiter{
      id,
      main_tx,
      rx,
    })
  }

  pub fn shutdown(&self) {
    debug!("Signal server to stop.");
    self.tx.send(StopEvent::StopServer).unwrap();
  }
}

impl ServerWaiter {
  pub fn wait_shutdown(&self) -> Result<StopEvent> {
    debug!("Server waiting for shutdown signal.");
    Ok(self.rx.recv()?)
  }

  pub fn server_stopped(&self) {
    debug!("Server stopped, let main thread know.");
    self.main_tx.send(StopEvent::StopServerFinished(self.id)).unwrap();
  }

  pub fn main_shutdown(&self) {
    info!("Signal main thread to shutdown.");
    self.main_tx.send(StopEvent::Shutdown).unwrap();
  }
}

#[derive(Clone)]
struct MainStopper {
  tx: Sender<StopEvent>,
  rx: Receiver<StopEvent>,
  servers: Vec<ServerStopper>,
}

impl MainStopper {
  pub fn new() -> Self {
    let (tx, rx) = bounded(1);
    Self { tx, rx,
      servers: Vec::new(),
    }
  }

  pub fn new_server(&mut self) -> ServerWaiter {
    let id = self.servers.len();
    let (stopper, waiter) = ServerStopper::new(id as u32, self.tx.clone());
    self.servers.push(stopper);
    waiter
  }

  pub fn wait_shutdown(&self) {
    // wait on main stopper
    debug!("Wait for shutdown signal");
    let mut stopped_counter = 0usize;
    // wait for shutdown signal.
    while stopped_counter < self.servers.len() {
      match self.rx.recv() {
        Err(err) => {
          error!("Main thread waiter received error: {:?}", err);
          return;
        },
        Ok(StopEvent::Shutdown) => {
          info!("Got shutdown signal.  Stop servers.");
          break;
        },
        Ok(StopEvent::StopServerFinished(id)) => {
          let len = self.servers.len();
          stopped_counter += 1;
          if stopped_counter < len {
            let remain = len - stopped_counter;
            debug!("Server({}) stopped.  Remaining {}", id, remain);
          } else {
            debug!("Server({}) stopped.  All servers stopped.  Stop main thread", id);
            return;
          }
        },
        Ok(ev) => {
          panic!("Main thread received invalid event: {:?}", ev);
        },
      }
    }

    // Tell all servers to shutdown.
    let mut counter = 0;
    for stopper in self.servers.iter() {
      stopper.shutdown();
      counter += 1;
    }
    // Wait for all servers to shutdown.
    while counter > 0 {
      match self.rx.recv() {
        Err(err) => {
          error!("Main thread waiter received error during shutdown: {:?}", err);
          return;
        },
        Ok(StopEvent::StopServerFinished(id)) => {
          counter -= 1;
          debug!("Server({}) stopped.  Remaining {}", id, counter);
        },
        Ok(ev) => {
          panic!("Main thread received invalid event during shutdown: {:?}", ev);
        },
      }
    }
    info!("Stopped all servers.");
  }
}

pub fn execute(config: AppConfig) -> Result<()> {
  // Stopper for main thread.
  let mut main_stopper = MainStopper::new();

  let servers = config.get_array("servers")?.expect("Missing list of servers");
  for server in servers.iter() {
    let server = server.clone().into_str()?;
    let cfg = config.clone();
    let waiter = main_stopper.new_server();
    debug!("Spawn server: {}", server);
    thread::spawn(move || {
      match run_server(&cfg, &server, waiter) {
        Err(err) => {
          error!("Error from server({}): {:?}", server, err);
        },
        _ => (),
      }
      debug!("run_server: stopped.");
    });
  }

  // wait on main stopper
  main_stopper.wait_shutdown();

  info!("main thread: stopped.");
  Ok(())
}

async fn test_db(url: String) -> Result<()> {
  let db = DbService::new(&url)?;
  db.prepare().await
}

fn run_server(config: &AppConfig, prefix: &str, waiter: ServerWaiter) -> Result<()> {
  let mut sys = System::new(format!("system.{}", prefix));

  let debug = config.get_bool("debug")?.unwrap_or(false);
  debug!("Debug = {:?}", debug);

  if debug {
    // configure db service factory
    let db_url = config.get_str("db.url")?.expect("db.url must be set");

    // Test db prepared statements.
    sys.block_on(test_db(db_url.to_string()))?;
  }

  // configure services
  info!("Serve.Services: configure services. prefix={}", prefix);
  let services = config_services(&config, prefix)?;

  // Check if stopper is enabled for this server
  let stopper = if config.get_bool(&format!("{}.stopper", prefix))?.unwrap_or_default() {
    Some(waiter.clone())
  } else {
    None
  };

  // Start http server
  let mut server = HttpServer::new(move || {
    // change default limits
    let form = web::FormConfig::default().limit(256 * 1024);

    let mut app = App::new()
      .app_data(form)
      // enable logger
      //.wrap(middleware::Logger::default())
      .wrap(middleware::Compress::default())
      .configure(|web| services.web_config(web));

    if let Some(ref stopper) = stopper {
      // Server stopper
      app = app.data(stopper.clone())
      .service(stop_server);
    }

    app
  });

  // workers
  if let Some(workers) = config.get_int(&format!("{}.workers", prefix))? {
    info!("Workers: {}", workers);
    server = server.workers(workers.try_into().expect("Workers must be > 0"));
  }

  // listen backlog
  if let Some(backlog) = config.get_int(&format!("{}.backlog", prefix))? {
    info!("Listen backlog: {}", backlog);
    server = server.backlog(backlog as i32);
  }

  // setup binds.
  let listen = config.get_str(&format!("{}.listen", prefix))?
    .expect(&format!("Missing {}.listen", prefix));
  info!("{} services listening on: {}", prefix, listen);
  server = server.bind(listen)?;

  // start server
  let server = server.run();

  if debug {
    let srv = server.clone();
    let waiter = waiter.clone();
    thread::spawn(move || {
      debug!("Wait for shutdown signal");
      // wait for shutdown signal.
      match waiter.wait_shutdown() {
        Err(_) => (),
        Ok(StopEvent::StopServer) => {
          debug!("Got shutdown signal.  Stop server: {}", waiter.id);
          executor::block_on(srv.stop(true));
          // notify main thread that we have stopped.
          waiter.server_stopped();
        },
        Ok(ev) => {
          error!("Server waiter received invalid event: {:?}", ev);
        },
      }
    });
  }

  // run server future
  let res = sys.block_on(server);
  waiter.server_stopped();
  Ok(res?)
}

