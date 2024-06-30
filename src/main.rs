use warp::{Filter, ws::WebSocket};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use futures::{StreamExt, SinkExt};

type Canvas = Arc<RwLock<Vec<String>>>;
type Clients = Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>>;

#[derive(Serialize, Deserialize)]
struct Update {
    x: usize,
    y: usize,
    color: String,
}

#[tokio::main]
async fn main() {
    let canvas: Canvas = Arc::new(RwLock::new(vec!["#FFFFFF".to_string(); 50 * 500]));
    let clients: Clients = Arc::new(RwLock::new(Vec::new()));

    let canvas = warp::any().map(move || canvas.clone());
    let clients = warp::any().map(move || clients.clone());

    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(canvas.clone())
        .and(clients.clone())
        .map(|ws: warp::ws::Ws, canvas, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, canvas, clients))
        });

    let routes = websocket.or(warp::fs::dir("static"));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: WebSocket, canvas: Canvas, clients: Clients) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    clients.write().await.push(tx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            ws_tx.send(message).await.unwrap_or_else(|e| {
                eprintln!("websocket send error: {}", e);
            });
        }
    });

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => break,
        };
        
        if let Ok(update) = serde_json::from_str::<Update>(msg.to_str().unwrap_or_default()) {
            let mut canvas = canvas.write().await;
            canvas[update.y * 50 + update.x] = update.color.clone();
            
            let clients = clients.read().await;
            let update_msg = warp::ws::Message::text(serde_json::to_string(&update).unwrap());
            for client in clients.iter() {
                let _ = client.send(update_msg.clone());
            }
        }
    }

    clients.write().await.retain(|client| client.send(warp::ws::Message::text("close")).is_ok());
}