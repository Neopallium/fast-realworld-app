use serde::de::Deserialize;

use async_std::path::PathBuf;

use std::collections::HashMap;

use clap::ArgMatches;
use config::{Config, ConfigError, Value, File, Environment};

use crate::error::*;

#[derive(Debug, Clone)]
pub struct AppConfig {
  pub conf: Config
}

impl AppConfig {
  pub fn new_clap(cli: &ArgMatches) -> Result<Self> {
    let mut conf = Config::default();
    // Load defaults
    conf.merge(File::with_name("conf/default"))?;

    if let Some(ref config_file) = cli.value_of("config") {
      conf.merge(File::with_name(config_file))?;
    } else {
      // Get RUN_MODE from environment
      let env = std::env::var("RUN_MODE").unwrap_or("development".into());
      conf.merge(File::with_name(&format!("conf/{}", env)).required(false))?;

      // Allow overrides from environment
      conf.merge(Environment::with_prefix("app").separator("_"))?;
    }

    Ok(AppConfig {
      conf,
    })
  }

  pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
    Ok(self.conf.get(key).or_else(|e| {
      match e {
        ConfigError::NotFound(_) => Ok(None),
        err => Err(err),
      }
    })?)
  }

  pub fn get_str(&self, key: &str) -> Result<Option<String>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Value::into_str(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_path(&self, key: &str) -> Result<Option<PathBuf>> {
    let val = if let Some(val) = self.get(key)? {
      Some(PathBuf::from(Value::into_str(val)?))
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Value::into_float(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Value::into_int(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Value::into_bool(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_table(&self, key: &str) -> Result<Option<Table>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Table(Value::into_table(val)?))
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_array(&self, key: &str) -> Result<Option<Vec<Value>>> {
    let val = if let Some(val) = self.get(key)? {
      Some(Value::into_array(val)?)
    } else {
      None
    };
    Ok(val)
  }
}

#[derive(Debug, Default, Clone)]
pub struct Table(HashMap<String, Value>);

impl Table {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn into_inner(self) -> HashMap<String, Value> {
    self.0
  }

  pub fn get(&self, key: &str) -> Option<Value> {
    match self.0.get(key) {
      Some(val) => Some(val.clone()),
      _ => None,
    }
  }

  pub fn get_str(&self, key: &str) -> Result<Option<String>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_str(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_path(&self, key: &str) -> Result<Option<PathBuf>> {
    let val = if let Some(val) = self.get(key) {
      Some(PathBuf::from(Value::into_str(val)?))
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_float(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_int(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_bool(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_table(&self, key: &str) -> Result<Option<HashMap<String, Value>>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_table(val)?)
    } else {
      None
    };
    Ok(val)
  }

  pub fn get_array(&self, key: &str) -> Result<Option<Vec<Value>>> {
    let val = if let Some(val) = self.get(key) {
      Some(Value::into_array(val)?)
    } else {
      None
    };
    Ok(val)
  }
}
