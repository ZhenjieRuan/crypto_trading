use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct BinanceSetting {
  pub api_key: String,
  pub api_secret: String,
  pub test_host: String,
  pub host: String,
  pub ws_base: String,
  pub proxy: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Setting {
  pub binance: BinanceSetting,
}

pub fn get_config(file: &str) -> Result<Setting> {
  let mut setting = config::Config::new();
  setting.merge(config::File::with_name(file)).unwrap();
  Ok(setting.try_into()?)
}
