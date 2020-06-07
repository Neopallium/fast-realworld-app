use serde::{Deserialize, Serialize};

use chrono::{Duration, Utc};

use jsonwebtoken::{
  encode, Header, EncodingKey,
  decode, DecodingKey,
  Validation
};

use crate::error::*;
use crate::models::User;

#[derive(Debug, Default, Clone)]
pub struct AuthData {
  pub user_id: i32,
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub id: i32,
  pub exp: i64,
}

pub trait GenerateJwt {
  fn generate_jwt(&self) -> Result<String>;
}

pub trait DecodeJwt {
  fn decode_jwt(&self) -> Result<AuthData>;
}

impl GenerateJwt for User {
  fn generate_jwt(&self) -> Result<String> {
    let claims = Claims{
      id: self.id,
      exp: (Utc::now() + Duration::days(21)).timestamp(),
    };

    let header = Header::default();
    let secret = &EncodingKey::from_secret(get_secret().as_ref());
    let token = encode(&header, &claims, secret)?;

    Ok(token)
  }
}

impl DecodeJwt for String {
  fn decode_jwt(&self) -> Result<AuthData> {
    let secret = get_secret();
    let secret_key = DecodingKey::from_secret(secret.as_ref());
    let token = decode::<Claims>(&self, &secret_key, &Validation::default())?;
    Ok(AuthData{
      user_id: token.claims.id,
      token: self.to_string(),
    })
  }
}

fn get_secret() -> String {
  dotenv::var("JWT_SECRET")
    .expect("Missing JWT_SECRET environment variable.")
}

