use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Block {
  pub hash: String,
  pub confirmations: i64, 
  pub size: i64, 
  pub strippedsize: i64,
  pub weight: i64,
  pub height: i64,
  pub version: i64,
  #[serde(rename = "versionHex")]
  pub version_hex: String,
  pub merkleroot: String,
  pub tx: Vec<String>, 
  pub time: i64,
  pub mediantime: i64,
  pub nonce: i64,
  pub bits: String,
  pub difficulty: f64,
  pub chainwork: String,
  #[serde(rename = "nTx")]
  pub n_tx: i64,
  pub previousblockhash: String,
  pub nextblockhash: Option<String>
}