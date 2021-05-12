use anyhow::Result;
use crossbeam_channel::Sender;
use futures::{stream::StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct MarketStream {
  endpoint: String,
}

impl MarketStream {
  pub fn new(endpoint: String) -> Self {
    Self { endpoint }
  }

  pub async fn subscribe(&self, stream: String, sender: Sender<String>) -> Result<()> {
    let stream_url = format!("{}/{}", self.endpoint, stream);
    log::info!("Connecting to {}", stream_url);
    let (mut stream, resp) = connect_async(stream_url)
      .await
      .map_err(|e| log::error!("Error connecting to stream: {:#?}", e))
      .unwrap();
    log::debug!("Websocket server response: {:#?}", resp);
    while let Some(item) = stream.next().await {
      match item {
        Ok(msg) => match msg {
          Message::Text(data) => {
            sender.send(data).map_err(|e| {
              log::error!("Failed to send data to receiver: {:#?}", e);
              e
            })?;
          }
          Message::Ping(ping) => {
            log::info!("Received Ping Msg");
            stream.send(Message::Pong(ping)).await.map_err(|e| {
              log::error!("Failed to send pong to server: {:#?}", e);
              e
            })?;
          }
          _ => log::error!("Received unsupported data type"),
        },
        Err(e) => log::error!("Failed to get message from stream: {:#?}", e),
      }
    }

    Ok(())
  }
}
