use crate::error::*;

use crate::auth::*;
use crate::models::*;
use crate::forms::*;

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

  // get profile
  get_profile: VersionedStatement,

  // (un)follow
  follow_user: VersionedStatement,
  unfollow_user: VersionedStatement,
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

  static ref FOLLOWER_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "followers",
      columns: vec![
        column("user_id"),
        column("follower_id"),
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

fn profile_from_row(row: &Row) -> Profile {
  let following: i32 = row.get(4);
  Profile {
    user_id: row.get(0),
    username: row.get(1),
    bio: row.get(2),
    image: row.get(3),
    following: (following > 0),
  }
}

fn profile_from_opt_row(row: &Option<Row>) -> Option<Profile> {
  if let Some(ref row) = row {
    Some(profile_from_row(row))
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

    // get profile
    let get_profile = VersionedStatement::new(cl.clone(),
        r#"SELECT u.id, u.username, u.bio, u.image,
          (CASE WHEN f.user_id IS NOT NULL THEN
            1 ELSE 0 END)::integer AS Following
        FROM users u LEFT JOIN followers f
          ON f.user_id = u.id AND follower_id = $1
        WHERE username = $2"#)?;

    // (un)follow
    let follow_user = VersionedStatement::new(cl.clone(),
        &FOLLOWER_COLUMNS.build_upsert("(user_id, follower_id)", true))?;
    let unfollow_user = VersionedStatement::new(cl.clone(),
        "DELETE FROM followers WHERE user_id = $1 AND follower_id = $2")?;

    Ok(UserService {
      user_by_id,
      user_by_email,
      user_by_username,

      insert_user,

      update_user_password,

      get_profile,

      follow_user,
      unfollow_user,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.user_by_id.prepare().await?;
    self.user_by_email.prepare().await?;
    self.user_by_username.prepare().await?;

    self.insert_user.prepare().await?;

    self.update_user_password.prepare().await?;

    self.get_profile.prepare().await?;

    self.follow_user.prepare().await?;
    self.unfollow_user.prepare().await?;
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

  pub async fn get_profile(&self, auth: Option<AuthData>, username: &str) -> Result<Option<Profile>> {
    let user_id = auth.unwrap_or_default().user_id;
    let row = self.get_profile.query_opt(&[&user_id, &username]).await?;
    Ok(profile_from_opt_row(&row))
  }

  pub async fn follow(&self, auth: AuthData, user_id: i32) -> Result<u64> {
    Ok(self.follow_user.execute(&[&user_id, &auth.user_id]).await?)
  }

  pub async fn unfollow(&self, auth: AuthData, user_id: i32) -> Result<u64> {
    Ok(self.unfollow_user.execute(&[&user_id, &auth.user_id]).await?)
  }

}
