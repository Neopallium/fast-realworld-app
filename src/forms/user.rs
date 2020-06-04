use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserOut<T> {
  pub user: T,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RegisterUser {
  pub username: String,
  pub email: String,
  pub password: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateUser {
  pub username: Option<String>,
  pub email: Option<String>,
  pub password: Option<String>,
  pub bio: Option<String>,
  pub image: Option<String>,
}

