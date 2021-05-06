use crate::binance::api::{CandlestickResp, OrderInput, OrderSide, OrderType};
use crate::binance::websocket::StreamCandle;
use anyhow::{ensure, Result};
use std::collections::VecDeque;

pub struct Turtle {
  n: f64,
  // monotonic queues to capture high and lows with a rolling window
  high_20: VecDeque<(f64, i64)>,
  low_20: VecDeque<(f64, i64)>,
  current_end: i64,   // timestamp that marks the end of current candle
  initial_asset: f64, // in terms of USDT
  usdt_balance: f64,
  btc_balance: f64,
  // record each entry for both position
  long_position: Vec<(f64, f64)>,  // BTC Amount, Price
  short_position: Vec<(f64, f64)>, // BTC Amount, Price
}

impl Turtle {
  pub fn new(candles: Vec<CandlestickResp>, usdt_balance: f64, btc_balance: f64) -> Result<Self> {
    ensure!(candles.len() > 20, "Not enough data supplied");

    // init with starting day high and low
    let mut initial_high_20 = candles[0].high;
    let mut high_timestamp = candles[0].close_time;
    let mut initial_low_20 = candles[0].low;
    let mut low_timestamp = candles[0].close_time;

    let mut atr_sum = initial_high_20 - initial_low_20; // Init the atr sum
    let prev_close = candles[0].close;
    for i in 1..candles.len() {
      let curr_high = candles[i].high;
      let curr_low = candles[i].low;
      let curr_close = candles[i].close_time;
      if curr_high > initial_high_20 {
        initial_high_20 = curr_high;
        high_timestamp = curr_close;
      }

      if curr_low < initial_low_20 {
        initial_low_20 = curr_low;
        low_timestamp = curr_close;
      }

      let tr = vec![
        curr_high - curr_low,
        (curr_high - prev_close).abs(),
        (prev_close - curr_low).abs(),
      ]
      .iter()
      .cloned()
      .fold(0. / 0., f64::max);
      atr_sum += tr;
    }
    let mut high_20 = VecDeque::new();
    let mut low_20 = VecDeque::new();
    high_20.push_back((initial_high_20, high_timestamp));
    low_20.push_back((initial_low_20, low_timestamp));

    let initial_asset = candles.last().unwrap().close * btc_balance + usdt_balance;

    Ok(Self {
      n: atr_sum / 20.0, // For the initial N, it's just the simple avg of 20 day's true range
      high_20,
      low_20,
      current_end: candles.last().unwrap().close_time,
      initial_asset,
      usdt_balance,
      btc_balance,
      // long, short position in terms of btc
      long_position: vec![],
      short_position: vec![],
    })
  }

  pub fn execute(&self, curr_candle: StreamCandle) -> Result<Vec<OrderInput>> {
    let curr_price = curr_candle.close.parse::<f64>()?;
    let total_asset = self.calc_total_asset(curr_price);
    // Turtle trades in terms of unit, if the price exceeds 20 day high,
    // we long 1 unit. If the price is below 20 day low, we short 1 unit
    // unit is in USDT
    let unit = (0.01 * total_asset) / self.n;

    // We make 4 unit position max to limit our exposure
    if curr_price > self.high_20.front().unwrap().0 && self.long_position.len() < 4 {
      return Ok(vec![self.buy_order(curr_candle.symbol, unit)]);
    }

    if curr_price < self.low_20.front().unwrap().0 && self.short_position.len() < 4 {
      return Ok(vec![self.sell_order(curr_candle.symbol, unit)]);
    }

    // take profit
    if total_asset / self.initial_asset > 1.5 {
      return Ok(vec![
        self.exit_long(curr_candle.symbol.clone(), curr_price),
        self.exit_short(curr_candle.symbol, curr_price),
      ]);
    }

    // Curr price is more than 2N lower than our last long position, we
    // should exit
    if self.long_position.len() > 0
      && self.long_position.last().unwrap().1 - curr_price > 2.0 * self.n
    {
      return Ok(vec![self.exit_long(curr_candle.symbol, curr_price)]);
    }

    // Curr price is more than 2N higher than our last short position, we
    // should exit
    if self.short_position.len() > 0
      && curr_price - self.short_position.last().unwrap().1 > 2.0 * self.n
    {
      return Ok(vec![self.exit_short(curr_candle.symbol, curr_price)]);
    }

    Ok(vec![])
  }

  fn calc_total_asset(&self, curr_price: f64) -> f64 {
    let short_profit = self.short_position.iter().fold(0.0, |mut profit, elem| {
      let (amount, price) = elem;
      profit + (curr_price - price) * amount
    });
    curr_price * self.btc_balance + self.usdt_balance + short_profit
  }

  fn exit_long(&self, symbol: String, curr_price: f64) -> OrderInput {
    let total_long_amount = self
      .long_position
      .iter()
      .fold(0.0, |position, elem| position + elem.0)
      * curr_price;
    self.sell_order(symbol, total_long_amount)
  }

  fn exit_short(&self, symbol: String, curr_price: f64) -> OrderInput {
    let total_short_amount = self
      .short_position
      .iter()
      .fold(0.0, |position, elem| position + elem.0)
      * curr_price;
    self.buy_order(symbol, total_short_amount)
  }

  fn buy_order(&self, symbol: String, unit: f64) -> OrderInput {
    OrderInput {
      symbol,
      side: OrderSide::Buy,
      order_type: OrderType::Market,
      time_in_force: None,
      quantity: None,
      quote_order_qty: Some(unit),
      price: None,
      new_client_order_id: "".to_string(),
      stop_price: None,
      iceberg_qty: None,
      new_order_resp_type: None,
      recv_window: None,
      timestamp: 0,
    }
  }

  fn sell_order(&self, symbol: String, unit: f64) -> OrderInput {
    OrderInput {
      symbol,
      side: OrderSide::Sell,
      order_type: OrderType::Market,
      time_in_force: None,
      quantity: None,
      quote_order_qty: Some(unit),
      price: None,
      new_client_order_id: "".to_string(),
      stop_price: None,
      iceberg_qty: None,
      new_order_resp_type: None,
      recv_window: None,
      timestamp: 0,
    }
  }

  fn update_high(&mut self, val: f64, timestamp: i64) {
    while self.high_20.len() > 0 && self.high_20.back().unwrap().0 < val {
      self.high_20.pop_front();
    }
    self.high_20.push_back((val, timestamp))
  }

  fn update_low(&mut self, val: f64, timestamp: i64) {
    while self.low_20.len() > 0 && self.low_20.back().unwrap().0 > val {
      self.low_20.pop_front();
    }
    self.low_20.push_back((val, timestamp))
  }

  fn update_n(&mut self, tr: f64) {
    self.n = (19.0 * self.n + tr) / 20.0;
  }
}
