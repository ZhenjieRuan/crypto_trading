use binance::api::{OrderInput, OrderSide, OrderType, TimeInForce};
use shared::utils;

mod binance;
mod shared;

#[tokio::main]
async fn main() {
  let argv: Vec<String> = std::env::args().collect();
  let config = shared::config::get_config(&argv[1]).unwrap();
  let binance_api = binance::client::Client::new(
    config.binance.api_key.clone(),
    config.binance.api_secret.clone(),
    config.binance.host.clone(),
  )
  .unwrap();

  let timestamp = utils::get_timestamp().unwrap();
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
