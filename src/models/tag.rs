use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Tag {
  pub article_id: i32,
  pub tag_name: String,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TagName(pub String);

