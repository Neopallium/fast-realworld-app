use log::*;

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::Value as JsonValue;

use libreauth::pass;

use jsonwebtoken::errors::Error as JwtError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  // 401
  #[error("unauthorized: {0}")]
  Unauthorized(JsonValue),

  // 404
  #[error("not found: {0}")]
  NotFound(JsonValue),

  // 422
  #[error("unprocessable entity: {0}")]
  UnprocessableEntity(JsonValue),

  // 500
  #[error("internal server error")]
  InternalServerError,

  // 400
  #[error("bad request: {0}")]
  BadRequest(String),

  // Json error
  #[error("Json error: {source}")]
  JsonError {
    #[from]
    source: serde_json::Error,
  },

  // Password error
  #[error("Password error: {0}")]
  PasswordError(String),

  #[error("JWT error")]
  JwtError {
    #[from]
    source: JwtError,
  },

  #[error("disconnected: {0}")]
  DisconnectedError(String),

  #[error("postgres error")]
  PgError {
    #[from]
    source: tokio_postgres::error::Error,
  },

  #[error("crossbeam recv error")]
  RecvError {
    #[from]
    source: crossbeam_channel::RecvError,
  },

  #[error("utf8 error")]
  FromUtf8Error {
    #[from]
    source: std::string::FromUtf8Error,
  },

  #[error("std io error")]
  IOError {
    #[from]
    source: std::io::Error,
  },

  #[error("config error")]
  ConfigError {
    #[from]
    source: config::ConfigError,
  },

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl From<pass::ErrorCode> for Error {
  fn from(code: pass::ErrorCode) -> Self {
    Error::PasswordError(format!("code={:?}", code))
  }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// the ResponseError trait lets us convert errors to http responses with appropriate data
// https://actix.rs/docs/errors/
impl ResponseError for Error {
  fn error_response(&self) -> HttpResponse {
    match self {
      Error::Unauthorized(ref message) => HttpResponse::Unauthorized().json(message),
      Error::NotFound(ref message) => HttpResponse::NotFound().json(message),
      Error::UnprocessableEntity(ref message) => {
        HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(message)
      },
      Error::BadRequest(ref message) => {
        HttpResponse::build(StatusCode::BAD_REQUEST).json(message)
      },
      Error::DisconnectedError(ref message) => {
        HttpResponse::build(StatusCode::BAD_GATEWAY).json(message)
      },
      ref err => {
        error!("InternalServerError: {:?}", err);
        HttpResponse::InternalServerError().json("Internal Server Error")
      },
    }
  }
}
