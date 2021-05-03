use anyhow::{ensure, Result};
use std::collections::BTreeMap;

use crate::binance::api::{OrderInput, OrderType};

pub fn get_timestamp() -> Result<u128> {
  Ok(
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)?
      .as_millis(),
  )
}

pub fn construct_query(params: BTreeMap<String, String>) -> String {
  let mut req = String::new();
  for (key, val) in params {
    req.push_str(format!("{}={}&", key, val).as_ref());
  }
  req.pop();
  req
}

pub fn build_order_query(request: OrderInput) -> Result<String> {
  // Sanity check for order input
  match request.order_type {
    OrderType::Limit => {
      ensure!(request.time_in_force.is_some(), "Missing Time In Force");
      ensure!(request.quantity.is_some(), "Missing Quantity");
      ensure!(request.price.is_some(), "Missing Price");
    }
    OrderType::Market => {
      ensure!(request.quantity.is_some(), "Missing Quantity");
      ensure!(request.quote_order_qty.is_some(), "Missing Quote Order Qty");
    }
    OrderType::StopLoss | OrderType::TakeProfit => {
      ensure!(request.quantity.is_some(), "Missing Quantity");
      ensure!(request.stop_price.is_some(), "Missing Stop Price");
    }
    OrderType::StopLossLimit | OrderType::TakeProfitLimit => {
      ensure!(request.time_in_force.is_some(), "Missing Time In Force");
      ensure!(request.quantity.is_some(), "Missing Quantity");
      ensure!(request.price.is_some(), "Missing Price");
      ensure!(request.stop_price.is_some(), "Missing Stop Price");
    }
    OrderType::LimitMaker => {
      ensure!(request.quantity.is_some(), "Missing Quantity");
      ensure!(request.price.is_some(), "Missing Price");
    }
  }

  let mut params: BTreeMap<String, String> = BTreeMap::new();
  params.insert("symbol".into(), request.symbol.into());
  params.insert("side".into(), request.side.into());
  params.insert("type".into(), request.order_type.into());
  params.insert("timestamp".into(), request.timestamp.to_string());
  params.insert("newClientOrderId".into(), request.new_client_order_id);
  if let Some(time_in_force) = request.time_in_force {
    params.insert("timeInForce".into(), time_in_force.into());
  }
  if let Some(quantity) = request.quantity {
    params.insert("quantity".into(), quantity.to_string());
  }
  if let Some(quote_order_qty) = request.quote_order_qty {
    params.insert("quoteOrderQty".into(), quote_order_qty.to_string());
  }
  if let Some(price) = request.price {
    params.insert("price".into(), price.to_string());
  }
  if let Some(stop_price) = request.stop_price {
    params.insert("stopPrice".into(), stop_price.to_string());
  }
  if let Some(iceberg_qty) = request.iceberg_qty {
    params.insert("icebergQty".into(), iceberg_qty.to_string());
  }
  if let Some(resp_type) = request.new_order_resp_type {
    params.insert("newOrderRespType".into(), resp_type.into());
  }
  if let Some(recv_window) = request.recv_window {
    params.insert("recvWindow".into(), recv_window.to_string());
  }

  Ok(construct_query(params))
}
