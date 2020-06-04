use std::str::FromStr;

// db <-> chrono util functions.

pub fn from_str_timestamp(secs: &str) -> Option<chrono::NaiveDateTime> {
  let secs = secs.trim();
  if secs.len() == 0 {
    return None;
  }
  match i64::from_str(secs) {
    Ok(secs) => Some(chrono::NaiveDateTime::from_timestamp(secs, 0)),
    Err(err) => {
      log::info!("Failed to parse string timestamp: {:?}", err);
      None
    },
  }
}

pub fn opt_timestamp_to_string(ts: Option<chrono::NaiveDateTime>) -> String {
  match ts {
    Some(ts) => ts.timestamp().to_string(),
    None => "".to_string(),
  }
}

pub fn from_timestamp(secs: i32) -> chrono::NaiveDateTime {
  chrono::NaiveDateTime::from_timestamp(secs as i64, 0)
}

pub fn from_opt_timestamp(val: Option<i32>) -> Option<chrono::NaiveDateTime> {
  val.map(from_timestamp)
}

pub fn from_naive_date_time(val: chrono::NaiveDateTime) -> i32 {
  val.timestamp() as _
}

pub fn from_opt_naive_date_time(val: Option<chrono::NaiveDateTime>) -> i32 {
  from_naive_date_time(val.unwrap_or_else(|| chrono::Utc::now().naive_utc()))
}

