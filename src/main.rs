use binance::api::{OrderInput, OrderSide, OrderType, TimeInForce};
use binance::websocket::Candlestick;
use chrono::Utc;
use shared::{config::Setting, utils};

mod binance;
mod shared;

#[tokio::main]
async fn main() {
  let argv: Vec<String> = std::env::args().collect();
  let config = shared::config::get_config(&argv[1]).unwrap();
  pretty_env_logger::init();

  let binance_client = binance::client::Client::new(
    config.binance.api_key.clone(),
    config.binance.api_secret.clone(),
    config.binance.host.clone(),
  )
  .unwrap();

  let now = Utc::now();
  let start_time = now
    .checked_sub_signed(chrono::Duration::days(21))
    .unwrap()
    .timestamp_millis();
  let end_time = now
    .checked_sub_signed(chrono::Duration::days(1))
    .unwrap()
    .timestamp_millis();
  let candlestick_req = binance::api::CandlestickInput {
    symbol: "BTCUSDT".into(),
    interval: "1d".into(),
    start_time: Some(start_time),
    end_time: Some(end_time),
    limit: None,
  };

  let candle_sticks = binance_client
    .market_candlestick(candlestick_req)
    .await
    .unwrap();
}

async fn test_market_data_stream(config: Setting) {
  let wss_endpoint = config.binance.ws_base;
  let (sender, receiver) = crossbeam_channel::unbounded();

  tokio::spawn(async {
    let market_stream = binance::data_stream::MarketStream::new(wss_endpoint);
    market_stream
      .subscribe("btcusdt@kline_1d".into(), sender)
      .await
  });

  while let Ok(msg) = receiver.recv_timeout(std::time::Duration::new(5, 0)) {
    let candle_stick: Candlestick = serde_json::from_str(&msg).unwrap();
    // log::info!("{:#?}", candle_stick)
  }
}

async fn test_order_api(config: Setting) {
  let binance_api = binance::client::Client::new(
    config.binance.api_key.clone(),
    config.binance.api_secret.clone(),
    config.binance.host.clone(),
  )
  .unwrap();

  let timestamp = utils::get_timestamp();
  let test_id = format!("test_order_{}", timestamp);
  let test_order = OrderInput {
    symbol: "BTCUSDT".into(),
    side: OrderSide::Buy,
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::GTC),
    quantity: Some(0.01),
    quote_order_qty: None,
    price: Some(12000.0),
    new_client_order_id: test_id,
    stop_price: None,
    iceberg_qty: None,
    new_order_resp_type: None,
    recv_window: None,
    timestamp,
  };

  binance_api.new_order(test_order, false).await.unwrap();
  binance_api.current_open_orders("BTCUSDT".to_string()).await;
}
