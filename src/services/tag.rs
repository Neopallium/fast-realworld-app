use actix_web::{
  get, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;

use crate::db::DbService;

/// Get list of tags
#[get("/tags")]
async fn list(
  db: web::Data<DbService>,
) -> Result<HttpResponse, Error> {
  // Get list of tags
  let tags = db.tag.get_tags().await?;
  Ok(HttpResponse::Ok().json(tags))
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
      .service(list);
  }
}

pub fn new_factory() -> TagService {
  Default::default()
}
