use crate::binance::api::{OrderInput, Spot};
use crate::shared::utils;
use anyhow::Result;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{self, HeaderValue};
use sha2::Sha256;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

pub struct Client {
  api_secret: String,
  host: String,
  client: reqwest::Client,
}

impl Client {
  pub fn new(api_key: String, api_secret: String, host: String) -> Result<Self> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&api_key)?);
    let client = reqwest::Client::builder()
      .connect_timeout(Duration::new(5, 0))
      .default_headers(headers)
      .build()?;
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

  pub async fn current_open_orders(&self, symbol: String) {
    let timestamp = utils::get_timestamp().unwrap();
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
        let mut signed_key = HmacSha256::new_from_slice(self.api_secret.as_bytes()).unwrap();
        signed_key.update(request.as_bytes());
        let signature = hex::encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("{}&signature={}", request, signature);
        format!("{}{}?{}", self.host, endpoint, request_body)
      }
      None => {
        let signed_key = HmacSha256::new_from_slice(self.api_secret.as_bytes()).unwrap();
        let signature = hex::encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("&signature={}", signature);
        format!("{}{}?{}", self.host, endpoint, request_body)
      }
    }
  }
}
