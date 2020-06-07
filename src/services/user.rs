use log::*;

use std::convert::TryFrom;

use actix_web::{
  get, post, put, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;
use crate::forms::*;
use crate::auth::AuthData;

use crate::db::DbService;
use crate::auth::pass;

use crate::middleware::Auth;

/// login user
#[post("/users/login")]
async fn login(
  db: web::Data<DbService>,
  login: web::Json<UserOut<LoginUser>>,
) -> Result<HttpResponse, Error> {
  let login = &login.user;
  // Get user from database
  let user = match db.user.get_by_email(&login.email).await? {
    Some(user) => user,
    _ => {
      // invalid user.
      return Ok(HttpResponse::NotFound().finish());
    }
  };

  let res = pass::check_password(&user.password, &login.password)?;
  info!("login: res={:?}", res);
  if res.is_valid {
    if res.needs_update {
      // Rehash password.
      db.user.update_password(user.id, &login.password).await?;
    }
    Ok(HttpResponse::Ok().json(UserResponse::try_from(user)?))
  } else {
    Ok(HttpResponse::Unauthorized().json(json!({
      "error": "Invalid user/password",
    })))
  }
}

/// register new user
#[post("/users")]
async fn register(
  cfg: web::Data<UserService>,
  db: web::Data<DbService>,
  register: web::Json<UserOut<RegisterUser>>,
) -> Result<HttpResponse, Error> {
  if !cfg.allow_register {
    return Ok(HttpResponse::Forbidden().finish());
  }

  let user = match db.user.register_user(&register.user).await? {
    Some(user) => user,
    _ => {
      return Ok(HttpResponse::InternalServerError().json("Failed to get user info."));
    },
  };

  Ok(HttpResponse::Ok().json(UserResponse::try_from(user)?))
}

/// get current user
#[get("/user", wrap="Auth::required()")]
async fn get_user(
  auth: AuthData,
  db: web::Data<DbService>,
) -> Result<HttpResponse, Error> {
  // Get auth user from database
  match db.user.get_by_id(auth.user_id).await? {
    Some(user) => {
      Ok(HttpResponse::Ok().json(UserResponse::try_from(user)?))
    },
    _ => {
      // invalid user.
      Ok(HttpResponse::NotFound().finish())
    }
  }
}

/// update user
#[put("/user", wrap="Auth::required()")]
async fn update(
  _auth: AuthData,
  _db: web::Data<DbService>,
  user: web::Json<UserOut<UpdateUser>>,
) -> Result<HttpResponse, Error> {
  let user = user.into_inner().user;

  info!("TODO");
  Ok(HttpResponse::Ok().json(user))
}

#[derive(Debug, Clone, Default)]
pub struct UserService {
  pub allow_register: bool,
}

impl super::Service for UserService {
  fn load_app_config(&mut self, config: &AppConfig, _prefix: &str) -> Result<()> {
    self.allow_register = config.get_bool("User.allow_register")?.unwrap_or(false);
    Ok(())
  }

  fn api_config(&self, web: &mut web::ServiceConfig) {
    web
      .data(self.clone())
      .service(register)
      .service(login)
      .service(update)
      .service(get_user);
  }
}

pub fn new_factory() -> UserService {
  Default::default()
}
