use crate::binance::api::AccountInfoResp;
use crate::shared::utils;
use crate::{
  binance::api::{KlineInput, KlineResp, Market, OrderInput, Spot},
  shared::utils::{to_f64, to_i64},
};
use anyhow::Result;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{self, HeaderValue};
use serde_json::Value;
use sha2::Sha256;
use std::time::Duration;

pub struct Client {
  api_secret: String,
  host: String,
  client: reqwest::Client,
}

impl Client {
  pub fn new(
    api_key: String,
    api_secret: String,
    host: String,
    proxy: Option<String>,
  ) -> Result<Self> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&api_key)?);
    let mut client_builder = reqwest::Client::builder()
      .connect_timeout(Duration::new(10, 0))
      .default_headers(headers);
    if let Some(proxy) = proxy {
      client_builder = client_builder.proxy(reqwest::Proxy::https(proxy)?);
    }
    let client = client_builder.build()?;

    Ok(Self {
      api_secret,
      host,
      client,
    })
  }

  pub async fn new_order(&self, input: OrderInput, is_test: bool) -> Result<()> {
    let query = utils::build_order_query(input)?;
    let endpoint = match is_test {
      true => Spot::TestNewOrder,
      false => Spot::NewOrder,
    };
    let signed_req = self.sign_request(endpoint.into(), Some(query));
    let res = self
      .client
      .post(signed_req)
      .send()
      .await
      .unwrap()
      .text()
      .await
      .unwrap();
    println!("New Order Res: {:#?}", res);
    Ok(())
  }

  pub async fn spot_account_info(&self) -> Result<AccountInfoResp> {
    let query = utils::build_spot_account_info_query(None)?;
    let signed_req = self.sign_request(Spot::AccountInfo.into(), Some(query));
    Ok(
      self
        .client
        .get(signed_req)
        .send()
        .await
        .map_err(|e| {
          log::error!("Error sending account info request: {:#?}", e);
          e
        })?
        .json::<AccountInfoResp>()
        .await?,
    )
  }

  pub async fn kline(&self, input: KlineInput) -> Result<Vec<KlineResp>> {
    let query = utils::build_kline_query(input)?;
    let req_url = format!("{}{}?{}", self.host, String::from(Market::Kline), query);
    let raw_values = self
      .client
      .get(req_url)
      .send()
      .await?
      .json::<Vec<Value>>()
      .await?;
    Ok(
      raw_values
        .iter()
        .map(|row| KlineResp {
          open_time: to_i64(&row[0]),
          open: to_f64(&row[1]),
          high: to_f64(&row[2]),
          low: to_f64(&row[3]),
          close: to_f64(&row[4]),
          volume: to_f64(&row[5]),
          close_time: to_i64(&row[6]),
          quote_asset_vol: to_f64(&row[7]),
          num_trades: to_i64(&row[8]),
          taker_buy_base_asset_vol: to_f64(&row[9]),
          taker_buy_quote_asset_vol: to_f64(&row[10]),
        })
        .collect::<Vec<KlineResp>>(),
    )
  }

  pub async fn current_open_orders(&self, symbol: String) {
    let timestamp = utils::get_timestamp();
    let mut params = std::collections::BTreeMap::new();
    params.insert("symbol".to_string(), symbol);
    params.insert("timestamp".to_string(), timestamp.to_string());
    let query = utils::construct_query(params);
    let signed_req = self.sign_request(Spot::OpenOrders.into(), Some(query));
    let res = self
      .client
      .get(signed_req)
      .send()
      .await
      .unwrap()
      .text()
      .await
      .unwrap();
    println!("Current Open Order Res: {}", res);
  }
  fn sign_request(&self, endpoint: String, req: Option<String>) -> String {
    match req {
      Some(request) => {
        let mut signed_key = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
        signed_key.update(request.as_bytes());
        let signature = hex::encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("{}&signature={}", request, signature);
        format!("{}{}?{}", self.host, endpoint, request_body)
      }
      None => {
        let signed_key = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
        let signature = hex::encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("&signature={}", signature);
        format!("{}{}?{}", self.host, endpoint, request_body)
      }
    }
  }
}
