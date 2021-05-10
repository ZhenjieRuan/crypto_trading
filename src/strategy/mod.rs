use crate::binance::websocket::StreamCandle;

pub mod turtle_trade;

#[derive(Debug)]
pub struct CandleStick {
  pub symbol: String,
  pub open_time: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: f64,
  pub close_time: i64,
}

impl From<StreamCandle> for CandleStick {
  fn from(stream_candle: StreamCandle) -> Self {
    Self {
      symbol: stream_candle.symbol,
      open_time: stream_candle.start_time,
      open: stream_candle.open.parse::<f64>().unwrap(),
      high: stream_candle.high.parse::<f64>().unwrap(),
      low: stream_candle.low.parse::<f64>().unwrap(),
      close: stream_candle.close.parse::<f64>().unwrap(),
      volume: stream_candle.base_asset_vol.parse::<f64>().unwrap(),
      close_time: stream_candle.close_time,
    }
  }
}
