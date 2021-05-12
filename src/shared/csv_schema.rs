use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum CsvDataType {
  Trade,
  OrderBook,
}

impl From<CsvDataType> for String {
  fn from(data_type: CsvDataType) -> Self {
    match data_type {
      CsvDataType::Trade => "trade".to_string(),
      CsvDataType::OrderBook => "orderbook".to_string(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
  #[serde(rename(serialize = "md_time", deserialize = "T"))]
  pub trade_time: i64,
  #[serde(rename(deserialize = "p"))]
  pub price: String,
  #[serde(rename(deserialize = "q"))]
  pub amount: String,
}
