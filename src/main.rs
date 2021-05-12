use chrono::Utc;
use crypto_trading::shared::{config::Setting, csv_schema::CsvDataType};
use crypto_trading::strategy::turtle_trade::Turtle;
use crypto_trading::strategy::CandleStick;
use crypto_trading::{binance::client::Client, shared::config::get_config};
use crypto_trading::{
  binance::websocket::{Kline, StreamTrade},
  shared::csv_schema::Trade,
};
use crypto_trading::{
  binance::{api::KlineInput, data_stream::MarketStream, websocket::StreamOrderbook},
  shared::utils::get_csv_writer,
};
use serde_json::Value;

#[tokio::main]
async fn main() {
  let argv: Vec<String> = std::env::args().collect();
  let config = get_config(&argv[1]).unwrap();
  pretty_env_logger::init();

  let (sender, receiver) = crossbeam_channel::unbounded();
  let trade_stream = format!("{}@trade", config.binance.symbol);
  let orderbook_stream = format!("{}@depth20@100ms", config.binance.symbol);
  let stream = format!("stream?streams={}/{}", trade_stream, orderbook_stream);
  let ws_base = config.binance.ws_base.clone();

  // Websocket stream receiver.
  // This will run in another thread
  tokio::spawn(async move {
    let market_stream = MarketStream::new(ws_base);
    market_stream.subscribe(stream, sender).await
  });

  let mut dump_date = chrono::Utc::now().format("%Y%m%d").to_string();
  let csv_dir = config.binance.csv_dir.unwrap();
  let mut trade_csv_writer = get_csv_writer(
    &csv_dir,
    &config.binance.symbol,
    CsvDataType::Trade,
    &dump_date,
  );
  let mut orderbook_csv_writer = get_csv_writer(
    &csv_dir,
    &config.binance.symbol,
    CsvDataType::OrderBook,
    &dump_date,
  );

  while let Ok(msg) = receiver.recv_timeout(std::time::Duration::new(5, 0)) {
    let curr_date = chrono::Utc::now().format("%Y%m%d").to_string();
    if curr_date != dump_date {
      dump_date = curr_date;
      trade_csv_writer = get_csv_writer(
        &csv_dir,
        &config.binance.symbol,
        CsvDataType::Trade,
        &dump_date,
      );
      orderbook_csv_writer = get_csv_writer(
        &csv_dir,
        &config.binance.symbol,
        CsvDataType::OrderBook,
        &dump_date,
      );
    }
    let raw_value = serde_json::from_str::<Value>(&msg).unwrap();
    let mut stream = raw_value
      .get("stream")
      .expect("Multi stream should contain stream tag")
      .to_string();
    // Get rid of the quotation mark(")
    stream.pop();
    stream.remove(0);
    let data = raw_value
      .get("data")
      .expect("Multi stream should contain data")
      .to_owned();
    if stream == trade_stream {
      let trade_record = serde_json::from_value::<Trade>(data).unwrap();
      log::debug!("{:#?}", trade_record);
      trade_csv_writer.serialize(trade_record).unwrap();
    } else if stream == orderbook_stream {
      let orderbook = serde_json::from_value::<StreamOrderbook>(data).unwrap();
      if orderbook.bids.len() < 10 || orderbook.asks.len() < 10 {
        log::error!("Orderbook data malformed, not enough length");
        continue;
      }
      let mut record = (0..10).fold(
        vec![chrono::Utc::now().timestamp_millis().to_string()],
        |mut record, i| {
          let bid = orderbook.bids[i][0].parse::<f64>().unwrap();
          let bid_amount = orderbook.bids[i][1].parse::<f64>().unwrap();
          let ask = orderbook.asks[i][0].parse::<f64>().unwrap();
          let ask_amount = orderbook.asks[i][1].parse::<f64>().unwrap();
          let mid = (bid + ask) / 2.0;
          record.append(&mut vec![
            bid.to_string(),
            ask.to_string(),
            bid_amount.to_string(),
            ask_amount.to_string(),
            mid.to_string(),
          ]);
          record
        },
      );
      orderbook_csv_writer.write_record(record).unwrap();
    }
  }
  trade_csv_writer.flush().unwrap();
  orderbook_csv_writer.flush().unwrap();
}

async fn trade(config: Setting) {
  let binance_client = Client::new(
    config.binance.api_key.clone(),
    config.binance.api_secret.clone(),
    config.binance.host.clone(),
    config.binance.proxy.clone(),
  )
  .unwrap();

  let spot_testnet_client = Client::new(
    config.binance.spot_test_api_key.clone(),
    config.binance.spot_test_api_secret.clone(),
    config.binance.test_host.clone(),
    config.binance.proxy.clone(),
  )
  .unwrap();

  let now = Utc::now();
  let start_time = now
    .checked_sub_signed(chrono::Duration::days(21))
    .unwrap()
    .timestamp_millis();
  let end_time = now.timestamp_millis();
  let kline_req = KlineInput {
    symbol: "BTCUSDT".into(),
    interval: "1d".into(),
    start_time: Some(start_time),
    end_time: Some(end_time),
    limit: None,
  };

  let klines = binance_client.kline(kline_req).await.unwrap();
  let spot_account_info = spot_testnet_client.spot_account_info().await.unwrap();

  let mut btc_balance = 0.0;
  let mut usdt_balance = 0.0;

  for balance in &spot_account_info.balances {
    if btc_balance != 0.0 && usdt_balance != 0.0 {
      break;
    }
    if balance.asset == "BTC" {
      btc_balance = balance.free.parse::<f64>().unwrap();
    }
    if balance.asset == "USDT" {
      usdt_balance = balance.free.parse::<f64>().unwrap();
    }
  }

  log::info!(
    "BTC Balance: {} USDT Balance: {}",
    btc_balance,
    usdt_balance
  );
  log::info!("Klines length: {:#?}", klines.len());

  let turtle_strat = Turtle::new(klines, usdt_balance, btc_balance).unwrap();
  start_test_turtle_strat(config, turtle_strat, spot_testnet_client).await;
}

async fn start_test_turtle_strat(config: Setting, turtle: Turtle, test_client: Client) {
  let wss_endpoint = config.binance.ws_base;
  let mut turtle = turtle;
  let (sender, receiver) = crossbeam_channel::unbounded();

  tokio::spawn(async move {
    let market_stream = MarketStream::new(wss_endpoint);
    market_stream
      .subscribe("btcusdt@kline_1d".into(), sender)
      .await
  });

  while let Ok(msg) = receiver.recv_timeout(std::time::Duration::new(5, 0)) {
    let curr_candle: CandleStick = serde_json::from_str::<Kline>(&msg).unwrap().candle.into();
    let orders = turtle
      .execute(curr_candle)
      .map_err(|e| log::error!("Error executing turtle strat: {:#?}", e))
      .unwrap();
    for order in orders {
      test_client.new_order(order, false).await.unwrap()
    }
  }
}
