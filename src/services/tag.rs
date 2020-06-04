use log::*;

use actix_web::{
  get, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;

/// Get list of tags
#[get("/tags")]
async fn list(
  _cfg: web::Data<TagService>,
) -> Result<HttpResponse, Error> {
  // TODO
  info!("Tag - list: TODO");
  Ok(HttpResponse::Ok().body("{}"))
}

#[derive(Debug, Clone, Default)]
pub struct TagService {
}

impl super::Service for TagService {
  fn load_app_config(&mut self, _config: &AppConfig, _prefix: &str) -> Result<()> {
    Ok(())
  }

  fn api_config(&self, web: &mut web::ServiceConfig) {
    web
      .data(self.clone())
      .service(list);
  }
}

pub fn new_factory() -> TagService {
  Default::default()
}
