use crate::binance::websocket::Candlestick;
use anyhow::Result;
use crossbeam_channel::Sender;
use futures::stream::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct MarketStream {
  endpoint: String,
}

impl MarketStream {
  pub fn new(endpoint: String) -> Self {
    Self { endpoint }
  }

  pub async fn subscribe(&self, stream: String, sender: Sender<Message>) -> Result<()> {
    let stream_url = format!("{}/{}", self.endpoint, stream);
    log::info!("Connecting to {}", stream_url);
    let (mut stream, _) = connect_async(stream_url).await?;
    while let Some(item) = stream.next().await {
      match item {
        Ok(msg) => {
          sender.send(msg).expect("Failed to send message");
        }
        Err(e) => log::error!("Failed to get message from stream: {:#?}", e),
      }
    }

    Ok(())
  }
}
