use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BinanceSetting {
  pub api_key: String,
  pub api_secret: String,
  pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct Setting {
  pub binance: BinanceSetting,
}

pub fn get_config(file: &str) -> Result<Setting> {
  let mut setting = config::Config::new();
  setting.merge(config::File::with_name(file)).unwrap();
  Ok(setting.try_into()?)
}
