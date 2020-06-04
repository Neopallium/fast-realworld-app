use crate::error::*;
use crate::models::*;

use crate::db::*;
use crate::db::util::*;

use tokio_postgres::Row;

#[derive(Clone)]
pub struct UserService {
  // gets
  user_by_email: VersionedStatement,
  user_by_username: VersionedStatement,
}

lazy_static! {
  static ref USER_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "users",
      columns: vec![
        column("id"),
        column("username"),
        column("email"),
        column("password"),
        column("bio"),
        column("image"),
        column("created_at"),
        column("updated_at"),
      ],
    }
  };
}

fn user_from_row(row: &Row) -> User {
  User {
    id: row.get(0),
    username: row.get(1),
    email: row.get(2),
    password: row.get(3),
    bio: row.get(4),
    image: row.get(5),
    created_at: row.get(6),
    updated_at: row.get(7),
  }
}

fn user_from_opt_row(row: &Option<Row>) -> Option<User> {
  if let Some(ref row) = row {
    Some(user_from_row(row))
  } else {
    None
  }
}

impl UserService {
  pub fn new(cl: SharedClient) -> Result<UserService> {
    let select = USER_COLUMNS.build_select_query(false);
    // Build user_by_* queries
    let user_by_email = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE email = $1"#, select))?;
    let user_by_username = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE username = $1"#, select))?;

    Ok(UserService {
      user_by_email,
      user_by_username,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.user_by_email.prepare().await?;
    self.user_by_username.prepare().await?;

    Ok(())
  }

  pub async fn get_by_email(&self, email: &str) -> Result<Option<User>> {
    let row = self.user_by_email.query_opt(&[&email]).await?;
    Ok(user_from_opt_row(&row))
  }

  pub async fn get_by_username(&self, username: &str) -> Result<Option<User>> {
    let row = self.user_by_username.query_opt(&[&username]).await?;
    Ok(user_from_opt_row(&row))
  }
}
