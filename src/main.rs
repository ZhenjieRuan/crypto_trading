use crate::binance::client::Client;
use crate::strategy::turtle_trade::Turtle;
use crate::strategy::CandleStick;
use binance::api::{KlineInput, OrderInput, OrderSide, OrderType, TimeInForce};
use binance::websocket::Kline;
use chrono::Utc;
use shared::{config::Setting, utils};

mod binance;
mod shared;
mod strategy;

#[tokio::main]
async fn main() {
  let argv: Vec<String> = std::env::args().collect();
  let config = shared::config::get_config(&argv[1]).unwrap();
  pretty_env_logger::init();

  let binance_client = binance::client::Client::new(
    config.binance.api_key.clone(),
    config.binance.api_secret.clone(),
    config.binance.host.clone(),
    config.binance.proxy.clone(),
  )
  .unwrap();

  let spot_testnet_client = binance::client::Client::new(
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
  let kline_req = binance::api::KlineInput {
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
    let market_stream = binance::data_stream::MarketStream::new(wss_endpoint);
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
