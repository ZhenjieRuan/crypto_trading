use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamCandle {
  #[serde(rename = "t")]
  pub start_time: u128,
  #[serde(rename = "T")]
  pub close_time: u128,
  #[serde(rename = "s")]
  pub symbol: String,
  #[serde(rename = "i")]
  pub interval: String,
  #[serde(rename = "f")]
  pub first_trade_id: u128,
  #[serde(rename = "L")]
  pub last_trade_id: u128,
  #[serde(rename = "o")]
  pub open: String,
  #[serde(rename = "c")]
  pub close: String,
  #[serde(rename = "h")]
  pub high: String,
  #[serde(rename = "l")]
  pub low: String,
  #[serde(rename = "v")]
  pub base_asset_vol: String,
  #[serde(rename = "n")]
  pub num_of_trades: u128,
  #[serde(rename = "x")]
  pub closed: bool,
  #[serde(rename = "q")]
  pub quote_asset_vol: String,
  #[serde(rename = "V")]
  pub taker_buy_base_asset_vol: String,
  #[serde(rename = "Q")]
  pub taker_buy_quote_asset_vol: String,
  #[serde(rename = "B")]
  pub ignore: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candlestick {
  #[serde(rename = "e")]
  pub event_type: String,
  #[serde(rename = "E")]
  pub event_time: u128,
  #[serde(rename = "s")]
  pub symbol: String,
  #[serde(rename = "k")]
  pub candle: StreamCandle,
}
