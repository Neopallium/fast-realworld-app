use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub email: String,
  pub password: String,
  pub bio: Option<String>,
  pub image: Option<String>,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Profile {
  pub username: String,
  pub bio: Option<String>,
  pub image: Option<String>,
  pub following: bool,
}
