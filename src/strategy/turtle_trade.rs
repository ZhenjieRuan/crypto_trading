use crate::binance::api::{KlineResp, OrderInput, OrderSide, OrderType};
use crate::strategy::CandleStick;
use anyhow::{ensure, Result};
use std::collections::VecDeque;

pub struct Turtle {
  n: f64,
  // monotonic queues to capture high and lows with a rolling window
  high_20: VecDeque<(f64, i64)>,
  low_20: VecDeque<(f64, i64)>,
  time_anchor: i64, // timestamp that marks the end of a day
  prev_close: f64,
  initial_asset: f64, // in terms of USDT
  usdt_balance: f64,
  btc_balance: f64,
  // record each entry for both position
  long_position: Vec<(f64, f64)>,  // BTC Amount, Price
  short_position: Vec<(f64, f64)>, // BTC Amount, Price
}

impl Turtle {
  pub fn new(candles: Vec<KlineResp>, usdt_balance: f64, btc_balance: f64) -> Result<Self> {
    ensure!(candles.len() > 20, "Not enough data supplied");

    // init with starting day high and low
    let mut initial_high_20 = candles[0].high;
    let mut high_timestamp = candles[0].close_time;
    let mut initial_low_20 = candles[0].low;
    let mut low_timestamp = candles[0].close_time;

    let mut atr_sum = initial_high_20 - initial_low_20; // Init the atr sum
    let mut prev_close = candles[0].close;
    for i in 1..candles.len() {
      prev_close = candles[i - 1].close;
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
      time_anchor: candles.last().unwrap().close_time,
      prev_close,
      initial_asset,
      usdt_balance,
      btc_balance,
      // long, short position in terms of btc
      long_position: vec![],
      short_position: vec![],
    })
  }

  pub fn execute(&mut self, curr_candle: CandleStick) -> Result<Vec<OrderInput>> {
    let curr_price = curr_candle.close;
    let curr_high_20 = self.high_20.front().unwrap().0;
    let curr_low_20 = self.low_20.front().unwrap().0;
    let total_asset = self.calc_total_asset(curr_price);

    log::info!(
      "Symbol: {} Curr Price: {} High 20: {} Low 20: {} Total Asset: {}",
      curr_candle.symbol,
      curr_price,
      curr_high_20,
      curr_low_20,
      total_asset
    );
    // Turtle trades in terms of unit, if the price exceeds 20 day high,
    // we long 1 unit. If the price is below 20 day low, we short 1 unit
    // unit is in USDT
    let unit = total_asset / self.n;

    // Update 20 day rolling high&low per 24hr period
    if chrono::Utc::now().timestamp_millis() > self.time_anchor {
      self.pop_old_high_low();
      self.update_high(curr_candle.high, curr_candle.close_time);
      self.update_low(curr_candle.low, curr_candle.close_time);
      self.update_n(curr_candle.high, curr_candle.low, curr_candle.close);

      self.time_anchor = (chrono::Utc::now() + chrono::Duration::days(1)).timestamp_millis();
    };

    // We make 4 unit position max to limit our exposure
    if curr_price > curr_high_20 && self.long_position.len() < 4 {
      let order = self.buy_order(curr_candle.symbol, unit);
      log::info!("Sending Long Order: {:#?}", order);
      // TODO: We should really be adding the orders to the order
      //  list when we heard back from user stream that this order
      //  is filled, but for simplicity let's just add it here for now
      self.long_position.push((unit, curr_price));
      return Ok(vec![order]);
    }

    if curr_price < curr_low_20 && self.short_position.len() < 4 {
      let order = self.sell_order(curr_candle.symbol, unit);
      log::info!("Sending Short Order: {:#?}", order);
      self.short_position.push((unit, curr_price));
      return Ok(vec![order]);
    }

    // take profit
    if total_asset / self.initial_asset > 1.5 {
      log::info!("Profit Taking");
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
      log::info!("Closing Long");
      return Ok(vec![self.exit_long(curr_candle.symbol, curr_price)]);
    }

    // Curr price is more than 2N higher than our last short position, we
    // should exit
    if self.short_position.len() > 0
      && curr_price - self.short_position.last().unwrap().1 > 2.0 * self.n
    {
      log::info!("Closing Short");
      return Ok(vec![self.exit_short(curr_candle.symbol, curr_price)]);
    }

    Ok(vec![])
  }

  fn calc_total_asset(&self, curr_price: f64) -> f64 {
    let short_profit = self.short_position.iter().fold(0.0, |profit, elem| {
      let (amount, price) = elem;
      profit + (curr_price - price) * amount
    });
    curr_price * self.btc_balance + self.usdt_balance + short_profit
  }

  fn exit_long(&mut self, symbol: String, curr_price: f64) -> OrderInput {
    let total_long_amount = self
      .long_position
      .iter()
      .fold(0.0, |position, elem| position + elem.0)
      * curr_price;
    self.long_position = vec![];
    self.sell_order(symbol, total_long_amount)
  }

  fn exit_short(&mut self, symbol: String, curr_price: f64) -> OrderInput {
    let total_short_amount = self
      .short_position
      .iter()
      .fold(0.0, |position, elem| position + elem.0)
      * curr_price;
    self.short_position = vec![];
    self.buy_order(symbol, total_short_amount)
  }

  fn buy_order(&self, symbol: String, unit: f64) -> OrderInput {
    let now = chrono::Utc::now().timestamp_millis();
    let order_id = format!("long_{}", now);
    OrderInput {
      symbol,
      side: OrderSide::Buy,
      order_type: OrderType::Market,
      time_in_force: None,
      quantity: None,
      quote_order_qty: Some(unit as f32),
      price: None,
      new_client_order_id: order_id,
      stop_price: None,
      iceberg_qty: None,
      new_order_resp_type: None,
      recv_window: None,
      timestamp: now,
    }
  }

  fn sell_order(&self, symbol: String, unit: f64) -> OrderInput {
    let now = chrono::Utc::now().timestamp_millis();
    let order_id = format!("short_{}", now);
    OrderInput {
      symbol,
      side: OrderSide::Sell,
      order_type: OrderType::Market,
      time_in_force: None,
      quantity: None,
      quote_order_qty: Some(unit as f32),
      price: None,
      new_client_order_id: order_id,
      stop_price: None,
      iceberg_qty: None,
      new_order_resp_type: None,
      recv_window: None,
      timestamp: now,
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

  fn pop_old_high_low(&mut self) {
    let high_front_timestamp = self.high_20.front().unwrap().1;
    let low_front_timestamp = self.low_20.front().unwrap().1;
    let time_limit = chrono::Utc::now()
      .checked_sub_signed(chrono::Duration::days(20))
      .unwrap()
      .timestamp();
    if high_front_timestamp < time_limit {
      self.high_20.pop_front();
    }
    if low_front_timestamp < time_limit {
      self.low_20.pop_front();
    }
  }

  fn update_n(&mut self, curr_high: f64, curr_low: f64, curr_close: f64) {
    let tr = vec![
      curr_high - curr_low,
      (curr_high - self.prev_close).abs(),
      (self.prev_close - curr_low).abs(),
    ]
    .iter()
    .cloned()
    .fold(0. / 0., f64::max);
    self.n = (19.0 * self.n + tr) / 20.0;
    self.prev_close = curr_close;
  }
}
