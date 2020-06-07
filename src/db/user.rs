use crate::error::*;
use crate::models::*;
use crate::forms::*;
use crate::auth::pass;

use crate::db::*;
use crate::db::util::*;

use tokio_postgres::Row;

#[derive(Clone)]
pub struct UserService {
  // gets
  user_by_id: VersionedStatement,
  user_by_email: VersionedStatement,
  user_by_username: VersionedStatement,

  // register user
  insert_user: VersionedStatement,

  // update password
  update_user_password: VersionedStatement,

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
    let user_by_id = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE id = $1"#, select))?;
    let user_by_email = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE email = $1"#, select))?;
    let user_by_username = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE username = $1"#, select))?;

    // register user
    let insert_user = VersionedStatement::new(cl.clone(),
        r#"INSERT INTO users(username, email, password)
        VALUES($1, $2, $3)"#)?;

    // update user password
    let update_user_password = VersionedStatement::new(cl.clone(),
        r#"UPDATE users SET password = $1 WHERE id = $2"#)?;

    Ok(UserService {
      user_by_id,
      user_by_email,
      user_by_username,

      insert_user,

      update_user_password
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.user_by_id.prepare().await?;
    self.user_by_email.prepare().await?;
    self.user_by_username.prepare().await?;

    self.insert_user.prepare().await?;

    self.update_user_password.prepare().await?;

    Ok(())
  }

  pub async fn get_by_id(&self, id: i32) -> Result<Option<User>> {
    let row = self.user_by_id.query_opt(&[&id]).await?;
    Ok(user_from_opt_row(&row))
  }

  pub async fn get_by_email(&self, email: &str) -> Result<Option<User>> {
    let row = self.user_by_email.query_opt(&[&email]).await?;
    Ok(user_from_opt_row(&row))
  }

  pub async fn get_by_username(&self, username: &str) -> Result<Option<User>> {
    let row = self.user_by_username.query_opt(&[&username]).await?;
    Ok(user_from_opt_row(&row))
  }

  pub async fn register_user(&self, user: &RegisterUser) -> Result<Option<User>> {
    let hash = pass::hash_password(&user.password)?;
    match self.insert_user.execute(&[&user.username, &user.email, &hash]).await? {
      0 => {
        // Insert user failed.
        Ok(None)
      },
      _ => {
        self.get_by_email(&user.email).await
      }
    }
  }

  pub async fn update_password(&self, user_id: i32, password: &str) -> Result<u64> {
    let hash = pass::hash_password(&password)?;
    Ok(self.update_user_password.execute(&[&hash, &user_id]).await?)
  }
}
