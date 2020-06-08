use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleOut<T> {
  pub article: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleList<T> {
  pub articles: Vec<T>,
  pub articles_count: usize,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ArticleRequest {
  pub tag: Option<String>,
  pub author: Option<String>,
  pub favorited: Option<String>,
  pub limit: Option<i64>,
  pub offset: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FeedRequest {
  pub limit: Option<i64>,
  pub offset: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
  pub title: String,
  pub description: String,
  pub body: String,
  pub tag_list: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
  pub title: Option<String>,
  pub description: Option<String>,
  pub body: Option<String>,
  pub tag_list: Vec<String>,
}

