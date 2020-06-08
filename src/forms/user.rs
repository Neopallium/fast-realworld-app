use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::auth::jwt::*;
use crate::models::{User, Profile};

#[derive(Debug, Deserialize)]
pub struct UserOut<T> {
  pub user: T,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LoginUser {
  pub email: String,
  pub password: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileOut {
  pub profile: Profile,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct UserResponseInner {
  pub username: String,
  pub token: String,
  pub email: String,
  pub bio: Option<String>,
  pub image: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct UserResponse {
  pub user: UserResponseInner,
}

impl TryFrom<User> for UserResponse {
  type Error = Error;

  fn try_from(user: User) -> Result<Self> {
    let token = user.generate_jwt()?;
    Ok(UserResponse {
      user: UserResponseInner {
        username: user.username,
        email: user.email,
        token,
        bio: user.bio,
        image: user.image,
      }
    })
  }
}
