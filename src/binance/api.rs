use serde::{Deserialize, Serialize};
use std::convert::From;

/// Spot APIs
/// API Spec: https://binance-docs.github.io/apidocs/spot/en/#spot-account-trade
pub enum Spot {
  OpenOrders,
  TestNewOrder,
  NewOrder,
  AccountInfo,
}

impl From<Spot> for String {
  fn from(endpoint: Spot) -> Self {
    String::from(match endpoint {
      Spot::OpenOrders => "/api/v3/openOrders",
      Spot::TestNewOrder => "/api/v3/order/test",
      Spot::NewOrder => "/api/v3/order",
      Spot::AccountInfo => "/api/v3/account",
    })
  }
}

/// Enum Spec: https://binance-docs.github.io/apidocs/spot/en/#public-api-definitions
#[derive(Clone)]
pub enum OrderSide {
  Buy,
  Sell,
}

impl From<OrderSide> for String {
  fn from(item: OrderSide) -> Self {
    match item {
      OrderSide::Buy => String::from("BUY"),
      OrderSide::Sell => String::from("SELL"),
    }
  }
}

#[derive(Clone)]
pub enum OrderType {
  Limit,
  Market,
  StopLoss,
  StopLossLimit,
  TakeProfit,
  TakeProfitLimit,
  LimitMaker,
}

impl From<OrderType> for String {
  fn from(item: OrderType) -> Self {
    match item {
      OrderType::Limit => String::from("LIMIT"),
      OrderType::Market => String::from("MARKET"),
      OrderType::StopLoss => String::from("STOP_LOSS"),
      OrderType::StopLossLimit => String::from("STOP_LOSS_LIMIT"),
      OrderType::TakeProfit => String::from("TAKE_PROFIT"),
      OrderType::TakeProfitLimit => String::from("TAKE_PROFIT_LIMIT"),
      OrderType::LimitMaker => String::from("LIMIT_MAKER"),
    }
  }
}

pub enum TimeInForce {
  // Good Until Canceled
  GTC,
  // Immediate or Cancel
  IOC,
  // Fill or Kill
  FOK,
}

impl From<TimeInForce> for String {
  fn from(item: TimeInForce) -> Self {
    match item {
      TimeInForce::GTC => String::from("GTC"),
      TimeInForce::IOC => String::from("IOC"),
      TimeInForce::FOK => String::from("FOK"),
    }
  }
}

pub enum OrderRespType {
  Ack,
  Result,
  Full,
}

impl From<OrderRespType> for String {
  fn from(item: OrderRespType) -> Self {
    match item {
      OrderRespType::Ack => String::from("ACK"),
      OrderRespType::Result => String::from("RESULT"),
      OrderRespType::Full => String::from("FULL"),
    }
  }
}

pub struct OrderInput {
  pub symbol: String,
  pub side: OrderSide,
  pub order_type: OrderType,
  pub time_in_force: Option<TimeInForce>,
  pub quantity: Option<f64>,
  pub quote_order_qty: Option<f64>,
  pub price: Option<f64>,
  pub new_client_order_id: String,
  pub stop_price: Option<f64>,
  pub iceberg_qty: Option<f64>,
  pub new_order_resp_type: Option<OrderRespType>,
  pub recv_window: Option<u64>, // Can't be greater than 60000,
  pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountBalanceInfo {
  pub asset: String,
  pub free: String,
  pub locked: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoResp {
  pub maker_commission: i64,
  pub taker_commission: i64,
  pub buyer_commission: i64,
  pub seller_commission: i64,
  pub can_trade: bool,
  pub can_withdraw: bool,
  pub can_deposit: bool,
  pub update_time: i64,
  pub account_type: String,
  pub balances: Vec<AccountBalanceInfo>,
}
/// Market Data Endpoints
pub enum Market {
  Kline,
}

impl From<Market> for String {
  fn from(endpoint: Market) -> Self {
    String::from(match endpoint {
      Market::Kline => "/api/v3/klines",
    })
  }
}

pub struct KlineInput {
  pub symbol: String,
  pub interval: String,
  pub start_time: Option<i64>,
  pub end_time: Option<i64>,
  pub limit: Option<u64>, // Default 500, max 1000
}

#[derive(Debug)]
pub struct KlineResp {
  pub open_time: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: f64,
  pub close_time: i64,
  pub quote_asset_vol: f64,
  pub num_trades: i64,
  pub taker_buy_base_asset_vol: f64,
  pub taker_buy_quote_asset_vol: f64,
}
