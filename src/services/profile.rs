use log::*;

use actix_web::{
  get, post, delete, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;
use crate::db::DbService;

/// get profile by username
#[get("/profiles/{username}")]
async fn get_profile(
  _db: web::Data<DbService>,
  _username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  info!("TODO");
  Ok(HttpResponse::Ok().finish())
}

/// follow a user
#[post("/profiles/{username}/follow")]
async fn follow(
  _db: web::Data<DbService>,
  _username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  Ok(HttpResponse::Ok().finish())
}

/// unfollow a user
#[delete("/profiles/{username}/follow")]
async fn unfollow(
  _db: web::Data<DbService>,
  _username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Clone, Default)]
pub struct ProfileService {
}

impl super::Service for ProfileService {
  fn load_app_config(&mut self, _config: &AppConfig, _prefix: &str) -> Result<()> {
    Ok(())
  }

  fn api_config(&self, web: &mut web::ServiceConfig) {
    web
      .data(self.clone())
      .service(get_profile)
      .service(follow)
      .service(unfollow);
  }
}

pub fn new_factory() -> ProfileService {
  Default::default()
}
