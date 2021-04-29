mod btc_analysis;

fn main() {
  let client = btc_analysis::client::ChainRpc::new(
    "bitcoinrpc".to_string(),
    "db4d929a9b0749f31cfbc466b66fb2ba".to_string(),
    "127.0.0.1".to_string(),
  );
  let latest_hash = client.get_best_block_hash().unwrap();
  let block = client.get_block(latest_hash, 1).unwrap();
  println!(
    "Block Hash: {}, Num Transactions: {}, Prev Block: {}",
    block.hash, block.n_tx, block.previousblockhash
  );
}
