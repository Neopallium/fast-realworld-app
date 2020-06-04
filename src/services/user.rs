use log::*;

use actix_web::{
  get, post, put, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;
use crate::forms::*;
use crate::db::DbService;

/// get current user
#[get("/user")]
async fn get_user(
  _db: web::Data<DbService>,
) -> Result<HttpResponse, Error> {
  info!("TODO");
  Ok(HttpResponse::Ok().finish())
}

/// update user
#[put("/user")]
async fn update(
  _db: web::Data<DbService>,
  user: web::Json<UserOut<UpdateUser>>,
) -> Result<HttpResponse, Error> {
  let user = user.into_inner().user;

  info!("TODO");
  Ok(HttpResponse::Ok().json(user))
}

/// register new user
#[post("/users")]
async fn register(
  _db: web::Data<DbService>,
  user: web::Json<UserOut<RegisterUser>>,
) -> Result<HttpResponse, Error> {
  let user = user.into_inner().user;

  info!("TODO");
  Ok(HttpResponse::Ok().json(user))
}

/// login user
#[post("/users/login")]
async fn login(
  _db: web::Data<DbService>,
  _body: String,
) -> Result<HttpResponse, Error> {
  Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Clone, Default)]
pub struct UserService {
}

impl super::Service for UserService {
  fn load_app_config(&mut self, _config: &AppConfig, _prefix: &str) -> Result<()> {
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
