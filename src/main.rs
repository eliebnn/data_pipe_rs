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
type PeerMap = Arc<Mutex<HashMap<String, (Tx, Vec<String>)>>>;

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

async fn accept_connection(state: PeerMap, raw_stream: tokio::net::TcpStream) {
    
    let channels = ["channel1".to_string(), "channel2".to_string()];
    let addr = raw_stream.peer_addr().unwrap().to_string();

    let ws_stream = accept_async(raw_stream).await.expect("Failed to accept");
    let (tx, mut rx) = ws_stream.split();

    state.lock().await.insert(addr.clone(), (tx, vec![]));

    tokio::spawn(async move {
        let mut interval = interval_at(Instant::now() + Duration::from_secs(5), Duration::from_secs(5));
        loop {

            tokio::select! {

                // Handling Subscriptions messaging

                _ = interval.tick() => {

                    let mut clients = state.lock().await;

                    if let Some(client) = clients.get_mut(&addr) {

                        match handle_channel1(&mut client.0, &client.1, &addr).await {
                            Some(_) => {println!("Sending channel1 to {}", &addr);},
                            None => {
                                println!("------------\r\nFailed to send message to {}", &addr);
                                println!("Closing the Stream.");
                                clients.remove(&addr);
                                break;
                            }
                        }
                        match handle_channel2(&mut client.0, &client.1, &addr).await {
                            Some(_) => {println!("Sending channel2 to {}", &addr);},
                            None => {
                                println!("------------\r\nFailed to send message to {}", &addr);
                                println!("Closing the Stream.");
                                clients.remove(&addr);
                                break;
                            }
                        }
                    }
                }

                // Handling coming messages

                msg = rx.next() => {

                    match msg {
                        Some(Ok(value)) => {
                            // Getting a subscription message
                            println!("Subscription attempt: `{}`", value.clone());

                            if channels.contains(&value.to_string()) {

                                let mut clients = state.lock().await;

                                if let Some(client) = clients.get_mut(&addr){
                                    println!("Valid Subscription: `{}`", value.clone());
                                    client.1.push(value.to_string())

                                }
                            }
                        },
                        _ => {
                            // Getting a connection closed message
                            println!("WebSocket closed for {}", &addr);
                            state.lock().await.remove(&addr);
                            break;
                        }
                    };
                }
                
            }
        }
    });
}

async fn handle_channel1(client_tx: &mut Tx, client_channels: &Vec<String>, addr: &String) -> Option<bool>{

    let success = if client_channels.contains(&"channel1".to_string()) {

        // Generate Message specific to `channel1`
        let m = format!("Channel 1 - {}: {}", Utc::now(), addr);
        let message = Some(Message::Text(m.to_string()));

        match message {
            Some(to_send) => {
                if let Err(_e) = client_tx.send(to_send.clone()).await {
                    println!("Failed to send message to {}", addr);
                    println!("Error: {}", _e);
                    false
                } else {true}
            },
            None => true,
        }

    } else {true};

    return if success {Some(true)} else {None};
}

async fn handle_channel2(client_tx: &mut Tx, client_channels: &Vec<String>, addr: &String) -> Option<bool>{

    let success = if client_channels.contains(&"channel2".to_string()) {

        // Generate Message specific to `channel2`
        let m = format!("Channel 2 - {}: {}", Utc::now(), addr);
        let message = Some(Message::Text(m.to_string()));

        match message {
            Some(to_send) => {
                if let Err(_e) = client_tx.send(to_send.clone()).await {
                    println!("Failed to send message to {}", addr);
                    println!("Error: {}", _e);
                    false
                } else {true}
            },
            None => true,
        }

    } else {true};

    return if success {Some(true)} else {None};
}

