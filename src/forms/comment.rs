use serde::{Deserialize, Serialize};

use crate::models::comment::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentOut<T> {
  pub comment: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentList {
  pub comments: Vec<CommentDetails>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CreateComment {
  pub body: String,
}
