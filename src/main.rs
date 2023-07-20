use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, Instant, interval_at};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::StreamExt;
use chrono::Utc;

use futures_util::SinkExt;

type Tx = futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>;
type PeerMap = Arc<Mutex<HashMap<String, Tx>>>;

#[tokio::main]
async fn main() {
    let state: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening on: {}", "localhost:8080");

    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            println!("New client: {}", addr);
            tokio::spawn(accept_connection(state.clone(), stream));
        }
    }
}

async fn accept_connection(state: PeerMap, raw_stream: TcpStream) {

    let addr = raw_stream.peer_addr().unwrap().to_string();
    let ws_stream = accept_async(raw_stream).await.expect("Failed to accept");
    let (tx, _rx) = ws_stream.split();

    state.lock().await.insert(addr.clone(), tx);

    let mut interval = interval_at(Instant::now() + Duration::from_secs(5), Duration::from_secs(5));

    // Sending a message every 5 seconds to the client
    loop {
        tokio::select! {
            _ = interval.tick() => {

                let m = format!("{}: {}", Utc::now(), &addr);
                let msg = Message::Text(m.to_string());

                let mut clients = state.lock().await;

                if let Some(client) = clients.get_mut(&addr) {
                    println!("Sending to {}", &addr);

                    if let Err(_e) = 
                    client.send(msg.clone()).await {
                        println!("Failed to send message to {}", &addr);
                        println!("Closing the Stream.");
                        break;
                    }
                }
            }
        }
    }
}
