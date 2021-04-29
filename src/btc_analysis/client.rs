use crate::btc_analysis::structs::Block;

use anyhow::{Result};
use jsonrpc::{Client, Response};
use serde_json;
pub struct ChainRpc {
  rpc_client: Client,
}

impl ChainRpc {
  pub fn new(username: String, password: String, endpoint: String) -> Self {
    let transport = jsonrpc::simple_http::SimpleHttpTransport::builder()
      .auth(username, Some(password))
      .url(&endpoint)
      .unwrap()
      .build();
    let rpc_client = Client::with_transport(transport);
    return Self { rpc_client };
  }

  fn send_request(
    &self,
    method: &str,
    args: &[Box<serde_json::value::RawValue>],
  ) -> Result<Response, jsonrpc::Error> {
    let req = self.rpc_client.build_request(method, args);
    self.rpc_client.send_request(req)
  }

  pub fn get_best_block_hash(&self) -> Result<String> {
    let method = "getbestblockhash";
    let block_hash = self
      .send_request(method, &[])
      .map_err(|e| anyhow::Error::new(e))?
      .result::<String>()?;

    Ok(block_hash)
  }

  pub fn get_block(&self, hash: String, verbosity: i64) -> Result<Block> {
    let method = "getblock";
    let res = self
      .send_request(method, &[jsonrpc::arg(hash), jsonrpc::arg(verbosity)])
      .unwrap()
      .result::<Block>()
      .unwrap();
    Ok(res)
  }
}
