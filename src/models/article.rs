use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

use crate::models::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Article {
  pub id: i32,
  pub author_id: i32,
  pub slug: String,
  pub title: String,
  pub description: String,
  pub body: String,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDetails {
  pub slug: String,
  pub title: String,
  pub description: String,
  pub body: String,
  pub tag_list: Vec<String>,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
  pub favorited: bool,
  pub favorites_count: i64,
  pub author: user::Profile,
}

