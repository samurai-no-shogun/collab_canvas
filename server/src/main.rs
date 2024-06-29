use warp::{Filter, ws::WebSocket, Reply};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use futures::{StreamExt, SinkExt};
use std::env;

type GridState = Arc<RwLock<HashMap<usize, String>>>;
type Clients = Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>>;

#[derive(Serialize, Deserialize, Clone)]
struct Update {
    index: usize,
    color: String,
}

#[tokio::main]
async fn main() {
    let grid_state: GridState = Arc::new(RwLock::new(HashMap::new()));
    let clients: Clients = Arc::new(RwLock::new(Vec::new()));

    let grid_state_filter = warp::any().map(move || grid_state.clone());
    let clients_filter = warp::any().map(move || clients.clone());

    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(grid_state_filter.clone())
        .and(clients_filter.clone())
        .map(|ws: warp::ws::Ws, grid_state, clients| {
            ws.on_upgrade(move |socket| handle_websocket(socket, grid_state, clients))
        });

    let get_state = warp::path("state")
        .and(grid_state_filter.clone())
        .and_then(get_grid_state);

    let routes = websocket
        .or(get_state)
        .or(warp::fs::dir("../www"));

    let port = env::var("PORT").unwrap_or_else(|_| "3030".to_string());
    let addr = ([0, 0, 0, 0], port.parse().unwrap());
    println!("Server starting on :{}", port);
    warp::serve(routes).run(addr).await;
}

async fn get_grid_state(grid_state: GridState) -> Result<impl Reply, warp::Rejection> {
    let state = grid_state.read().await;
    Ok(warp::reply::json(&*state))
}

async fn handle_websocket(ws: WebSocket, grid_state: GridState, clients: Clients) {
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
            let mut state = grid_state.write().await;
            state.insert(update.index, update.color.clone());
            
            let clients = clients.read().await;
            let update_msg = warp::ws::Message::text(serde_json::to_string(&update).unwrap());
            for client in clients.iter() {
                let _ = client.send(update_msg.clone());
            }
        }
    }

    clients.write().await.retain(|client| client.send(warp::ws::Message::text("close")).is_ok());
}