use actix_web::{
  get, post, delete, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;

use crate::forms::*;

use crate::db::DbService;

use crate::auth::AuthData;
use crate::middleware::Auth;


/// get profile by username
#[get("/profiles/{username}", wrap="Auth::optional()")]
async fn get_profile(
  auth: Option<AuthData>,
  db: web::Data<DbService>,
  username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let auth = auth.unwrap_or_default();

  match db.user.get_profile(&auth, &username).await? {
    Some(profile) => {
      Ok(HttpResponse::Ok().json(ProfileOut {
        profile,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Profile not found",
      })))
    }
  }
}

/// follow a user
#[post("/profiles/{username}/follow", wrap="Auth::required()")]
async fn follow(
  auth: AuthData,
  db: web::Data<DbService>,
  username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  match db.user.get_profile(&auth, &username).await? {
    Some(mut profile) => {
      // Check if the current user is already following them.
      if !profile.following {
        // update DB to mark the current user as following them.
        db.user.follow(&auth, profile.user_id).await?;
        profile.following = true;
      }
      Ok(HttpResponse::Ok().json(ProfileOut {
        profile,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Profile not found",
      })))
    }
  }
}

/// unfollow a user
#[delete("/profiles/{username}/follow", wrap="Auth::required()")]
async fn unfollow(
  auth: AuthData,
  db: web::Data<DbService>,
  username: web::Path<String>,
) -> Result<HttpResponse, Error> {
  match db.user.get_profile(&auth, &username).await? {
    Some(mut profile) => {
      // Check if the current user is already following them.
      if profile.following {
        // update DB to mark the current user as not following them.
        db.user.unfollow(&auth, profile.user_id).await?;
        profile.following = false;
      }
      Ok(HttpResponse::Ok().json(ProfileOut {
        profile,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Profile not found",
      })))
    }
  }
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
