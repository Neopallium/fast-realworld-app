use serde::{Deserialize, Serialize};

use crate::models::tag::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TagList {
  pub tags: Vec<TagName>,
}
