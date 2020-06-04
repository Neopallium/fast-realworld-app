use log::*;

use std::collections::HashMap;

use actix_web::{web};

use crate::error::*;
use crate::app::*;
use crate::db::DbService;

mod user;
mod profile;
mod article;
mod tag;

type BoxService = Box<dyn Service>;

pub trait Service: ServiceClone + Send {
  /// Load Service config from AppConfig.
  fn load_app_config(&mut self, config: &AppConfig, prefix: &str) -> Result<()>;

  /// Setup Service endpoints.
  fn web_config(&self, _web: &mut web::ServiceConfig) {
  }

  fn api_config(&self, _web: &mut web::ServiceConfig) {
  }
}

pub trait ServiceClone {
  fn clone_box(&self) -> BoxService;
}

impl<T> ServiceClone for T
where
    T: 'static + Service + Clone,
{
  fn clone_box(&self) -> BoxService {
    Box::new(self.clone())
  }
}

impl Clone for BoxService {
  fn clone(&self) -> BoxService {
    self.clone_box()
  }
}

#[derive(Clone, Default)]
pub struct Services {
  db_url: String,
  services: Vec<BoxService>,
}

impl Services {
  pub fn new() -> Services {
    Default::default()
  }

  fn load_service(&mut self, name: &str, config: &AppConfig, prefix: &str) -> Result<BoxService> {
    let mut service: BoxService = match name {
      "User" => Box::new(user::new_factory()),
      "Profile" => Box::new(profile::new_factory()),
      "Article" => Box::new(article::new_factory()),
      "Tag" => Box::new(tag::new_factory()),
      _ => {
        panic!("Unknown Service: {}", name);
      },
    };

    service.load_app_config(&config, prefix)?;
    Ok(service)
  }

  /// Load Service config from AppConfig.
  pub fn load_app_config(&mut self, config: &AppConfig, prefix: &str) -> Result<()> {
    // DB config
    self.db_url = config.get_str("db.url")?.expect("db.url must be set");

    let mut loaded: HashMap<String, bool> = HashMap::new();
    let list = config.get_array(&format!("{}.services", prefix))?
      .expect("missing list of services.");
    for name in list.iter() {
      let name = name.clone().into_str()?;
      info!("Loading {}Service config", name);
      // check if it is loaded already.
      if let Some(_) = loaded.get(&name) {
        panic!("can't load service multiple times.")
      }
      loaded.insert(name.clone(), true);
      // load service
      let service = self.load_service(&name, config, prefix)?;
      self.services.push(service);
    }
    Ok(())
  }

  /// Setup Service endpoints.
  pub fn web_config(&self, web: &mut web::ServiceConfig) {
    // Create DbService for worker.
    let db = DbService::new(&self.db_url).expect("Failed to init db.");
    web.data(db);

    for service in self.services.iter() {
      service.web_config(web);
    }
    web.service(
      web::scope("/api")
        .configure(|web| {
          for service in self.services.iter() {
            service.api_config(web);
          }
        })
    );
  }
}

pub fn config_services(config: &AppConfig, prefix: &str) -> Result<Services> {
  let mut services = Services::new();
  services.load_app_config(config, prefix)?;
  Ok(services)
}
