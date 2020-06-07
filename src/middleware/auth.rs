use log::*;

use std::task::{Context, Poll};

use futures::future::{ok, err, Either, Ready};

use actix_web::{
  http::header::{
    HeaderMap, AUTHORIZATION
  },
  error::ErrorNotFound,
  Error, HttpMessage,
  HttpResponse, ResponseError,
  HttpRequest, FromRequest
};
use actix_web::dev::{
  Service, Transform,
  ServiceRequest, ServiceResponse,
  Payload,
};

use crate::error::Result;
use crate::auth::jwt::*;

const TOKEN_PREFIX: &str = "Token ";

pub fn decode_jwt_claims(headers: &HeaderMap) -> Result<Option<AuthData>> {
  let token = match headers.get(AUTHORIZATION) {
    Some(token) => {
      let token = token.to_str().map_err(|_| {
        crate::error::Error::Unauthorized(json!({
          "error": "Invalid authorization token",
        }))
      })?;
      if !token.starts_with(TOKEN_PREFIX) {
        return Err(crate::error::Error::Unauthorized(json!({
          "error": "Invalid authorization method",
        })));
      }
      // remove prefix
      token.replacen(TOKEN_PREFIX, "", 1)
    },
    None => {
      // No authorization provided.  Allow caller to decide if this is an error.
      return Ok(None);
    },
  };

  let auth_data = token.decode_jwt()?;

  Ok(Some(auth_data))
}

impl FromRequest for AuthData {
  type Error = Error;
  type Future = Ready<Result<Self, Self::Error>>;
  type Config = ();

  fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
    match req.extensions().get::<AuthData>() {
      Some(auth) => {
        ok(auth.clone())
      },
      None => {
        err(ErrorNotFound("No authoration token"))
      }
    }
  }
}

pub struct Auth {
  pub is_optional: bool,
}

impl Auth {
  pub fn required() -> Self {
    Self {
      is_optional: false,
    }
  }

  pub fn optional() -> Self {
    Self {
      is_optional: true,
    }
  }
}

impl<S, B> Transform<S> for Auth
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(AuthMiddleware {
      is_optional: self.is_optional,
      service
    })
  }
}

pub struct AuthMiddleware<S> {
  is_optional: bool,
  service: S,
}

impl<S, B> Service for AuthMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

  fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, req: ServiceRequest) -> Self::Future {
    let has_auth = match decode_jwt_claims(req.headers()) {
      Ok(Some(auth_data)) => {
        debug!("Has authorization token: {:?}", auth_data);
        req.extensions_mut().insert(Some(auth_data));

        true
      },
      Ok(None) => {
        debug!("No authorization token");
        false
      },
      Err(err) => {
        error!("Error getting JWT claims: {:?}", err);
        return Either::Right(ok(req.into_response(
          err.error_response().into_body()
        )));
      },
    };

    debug!("Auth check: has_auth={}, optional={}", has_auth, self.is_optional);
    if has_auth || self.is_optional {
      Either::Left(self.service.call(req))
    } else {
      Either::Right(ok(req.into_response(
        HttpResponse::Unauthorized().json(json!({
          "error": "authorization required",
        }))
        .into_body()
      )))
    }
  }
}
