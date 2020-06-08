use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

use crate::models::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Comment {
  pub id: i32,
  pub article_id: i32,
  pub user_id: i32,
  pub body: String,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommentDetails {
  pub id: i32,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
  pub body: String,
  pub author: user::Profile,
}

