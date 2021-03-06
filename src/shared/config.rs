use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct BinanceSetting {
  pub spot_test_api_key: String,
  pub spot_test_api_secret: String,
  pub api_key: String,
  pub api_secret: String,
  pub test_host: String,
  pub host: String,
  pub ws_base: String,
  pub proxy: Option<String>,
  pub csv_dir: Option<String>,
  pub symbol: String
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
