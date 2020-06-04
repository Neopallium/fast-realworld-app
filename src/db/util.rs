use log::*;

use std::collections::HashMap;
use std::io::Write;

use tokio_postgres::{Row, types::Type};

use crate::error::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnNote {
  Primary,
  Extra,
  None,
}

#[derive(Debug, Clone)]
pub struct ColumnMapper {
  pub name: String,
  pub column: String,
  pub note: ColumnNote,
}

impl Default for ColumnMapper {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      column: "".to_string(),
      note: ColumnNote::None,
    }
  }
}

pub fn column(name: &'static str) -> ColumnMapper {
  ColumnMapper {
    name: name.to_string(),
    column: name.to_string(),
    note: ColumnNote::None,
  }
}

pub fn quoted(name: &'static str) -> ColumnMapper {
  ColumnMapper {
    name: name.to_string(),
    column: format!(r#""{}""#, name),
    note: ColumnNote::None,
  }
}

#[derive(Debug, Default, Clone)]
pub struct ColumnMappers {
  pub table_name: &'static str,
  pub columns: Vec<ColumnMapper>,
}

impl ColumnMappers {
  pub fn get_columns(&self, all_columns: bool) -> String {
    self.columns.iter().filter_map(|col| {
      if all_columns || col.note != ColumnNote::Extra {
        Some(col.column.clone())
      } else {
        None
      }
    }).collect::<Vec<String>>().join(", ")
  }

  pub fn build_select_query(&self, all_columns: bool) -> String {
    let mut buf = Vec::new();
    let mut first = true;
    write!(buf, "SELECT ").unwrap();
    for col in self.columns.iter() {
      if all_columns || col.note != ColumnNote::Extra {
        if first {
          write!(buf, "{}", col.column).unwrap();
          first = false;
        } else {
          write!(buf, ", {}", col.column).unwrap();
        }
      }
    }
    write!(buf, " FROM {}", self.table_name).unwrap();
    String::from_utf8_lossy(&buf).to_string()
  }

  pub fn build_insert_query(&self, all_columns: bool) -> String {
    let mut buf = Vec::new();
    let mut idx = 0;
    let mut values = Vec::new();
    write!(buf, "INSERT INTO {}(", self.table_name).unwrap();
    for col in self.columns.iter() {
      if all_columns || col.note != ColumnNote::Extra {
        if idx > 0 {
          write!(buf, ",").unwrap();
        }
        idx += 1;
        values.push(format!("${}", idx));
        write!(buf, "{}", col.column).unwrap();
      }
    }
    write!(buf, ") VALUES({})", values.join(", ")).unwrap();
    String::from_utf8_lossy(&buf).to_string()
  }

  pub fn build_upsert(&self, on_conflict: &str, all_columns: bool) -> String {
    let mut buf = Vec::new();
    let mut idx = 0;
    let mut values = Vec::new();
    write!(buf, "INSERT INTO {}(", self.table_name).unwrap();
    for col in self.columns.iter() {
      if all_columns || col.note != ColumnNote::Extra {
        if idx > 0 {
          write!(buf, ",").unwrap();
        }
        idx += 1;
        values.push(format!("${}", idx));
        write!(buf, "{}", col.column).unwrap();
      }
    }
    write!(buf, r#") VALUES({})
      ON CONFLICT {}
    DO UPDATE SET "#, values.join(", "), on_conflict).unwrap();
    idx = 0;
    for col in self.columns.iter() {
      if all_columns || col.note != ColumnNote::Extra {
        if idx > 0 {
          write!(buf, ",").unwrap();
        }
        idx += 1;
        write!(buf, " {} = EXCLUDED.{}", col.column, col.column).unwrap();
      }
    }
    String::from_utf8_lossy(&buf).to_string()
  }

  pub fn build_update_where(&self, lookup: &str, all_columns: bool) -> String {
    let mut buf = Vec::new();
    let mut idx = 0;
    let mut lookup_column = lookup.to_string();
    write!(buf, "UPDATE {} SET ", self.table_name).unwrap();
    for col in self.columns.iter() {
      if col.name == lookup {
        lookup_column = col.column.clone();
      } else if all_columns || col.note != ColumnNote::Extra {
        if idx > 0 {
          write!(buf, ",").unwrap();
        }
        idx += 1;
        write!(buf, " {} = ${}", col.column, idx).unwrap();
      }
    }
    idx += 1;
    write!(buf, " WHERE {} = ${}", lookup_column, idx).unwrap();
    String::from_utf8_lossy(&buf).to_string()
  }

  pub fn get_update_set_columns(&self, all_columns: bool) -> (u32, String) {
    let mut buf = Vec::new();
    let mut idx: u32 = 0;
    for col in self.columns.iter() {
      if all_columns || col.note != ColumnNote::Extra {
        if idx > 0 {
          write!(buf, ",").unwrap();
        }
        idx += 1;
        write!(buf, " {} = ${}", col.column, idx).unwrap();
      }
    }
    (idx, String::from_utf8_lossy(&buf).to_string())
  }

  pub fn row_to_map(&self, row: &Row, map: &mut HashMap<String, String>) -> Result<()> {
    let columns = row.columns();
    let len = columns.len();
    for (idx, col) in self.columns.iter().enumerate() {
      if col.note != ColumnNote::Extra { continue; }
      if idx >= len { break; }
      match row_value_to_string(row, idx, columns[idx].type_()) {
        Ok(Some(val)) => {
          map.insert(col.name.to_string(), val);
        },
        Ok(None) => {
        },
        Err(err) => {
          info!("-- Error decoding '{}': {:?}", col.name, err);
        },
      }
    }
    Ok(())
  }
}

fn row_value_to_string(row: &Row, idx: usize, col_type: &Type) -> Result<Option<String>> {
  match *col_type {
    Type::VARCHAR => {
      let val: Option<String> = row.try_get(idx)?;
      Ok(val)
    },
    Type::INT2 => {
      let val: Option<i16> = row.try_get(idx)?;
      Ok(val.map(|v| v.to_string()))
    },
    Type::INT4 => {
      let val: Option<i32> = row.try_get(idx)?;
      Ok(val.map(|v| v.to_string()))
    },
    Type::INT8 => {
      let val: Option<i64> = row.try_get(idx)?;
      Ok(val.map(|v| v.to_string()))
    },
    _ => {
      info!("Unhandled column type: {:?}", col_type);
      Ok(None)
    },
  }
}

