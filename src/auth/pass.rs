use libreauth::pass::{Algorithm, HashBuilder, Hasher};

use crate::error::*;

pub const PWD_ALGORITHM: Algorithm = Algorithm::Argon2;
pub const PWD_SCHEME_VERSION: usize = 1;

// If the Hasher changes, make sure to increment PWD_SCHEME_VERSION
lazy_static! {
  pub static ref HASHER: Hasher = {
    HashBuilder::new()
      .algorithm(PWD_ALGORITHM)
      .version(PWD_SCHEME_VERSION)
      .finalize()
      .unwrap()
  };
}

#[derive(Debug)]
pub struct CheckedPass {
  pub is_valid: bool,
  pub needs_update: bool,
}

impl CheckedPass {
  pub fn new(is_valid: bool, needs_update: bool) -> Self {
    Self {
      is_valid, needs_update
    }
  }
}

pub fn check_password(stored: &str, password: &str) -> Result<CheckedPass> {
  let checker = HashBuilder::from_phc(stored)?;
  if checker.is_valid(password) {
    if checker.needs_update(Some(PWD_SCHEME_VERSION)) {
      Ok(CheckedPass::new(true, true))
    } else {
      Ok(CheckedPass::new(true, false))
    }
  } else {
    Ok(CheckedPass::new(false, false))
  }
}

pub fn hash_password(password: &str) -> Result<String> {
  Ok(HASHER.hash(password)?)
}

