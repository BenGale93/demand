use std::{
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    time,
};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    models::{Buy, Item},
    pricing::{Pricer, SimplePricer},
};

type BuysState = Arc<Mutex<Vec<Buy>>>;
type PricerState = Arc<Mutex<dyn Pricer>>;
type ItemState = Arc<Mutex<Vec<Item>>>;

async fn handle_connection(
    buys: BuysState,
    items: ItemState,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        let msg = msg.unwrap();
        if msg.is_close() {
            break;
        }

        let msg_text = msg.to_text().unwrap();
        println!("Received a message from {}: {}", addr, msg_text);
        let (_, command) = crate::commands::parse_command(msg_text).unwrap();
        match command {
            crate::commands::Command::Buy((name, cost)) => {
                buys.lock().unwrap().push(Buy { name, cost });
            }
            crate::commands::Command::View => {
                let item_text: Vec<_> = items
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect();
                let item_text = item_text.join(", ");
                ws_sender
                    .send(Message::Text(item_text.into()))
                    .await
                    .unwrap();
            }
        }
    }
}

async fn update_pricer(buys: BuysState, pricer: PricerState) {
    let mut interval = time::interval(time::Duration::from_secs(2));
    loop {
        interval.tick().await;
        let mut buy_lock = buys.lock().unwrap();
        pricer.lock().unwrap().update(&mut buy_lock);
    }
}

async fn reprice(items: ItemState, pricer: PricerState) {
    let mut interval = time::interval(time::Duration::from_secs(2));
    loop {
        interval.tick().await;
        let mut items_lock = items.lock().unwrap();
        pricer.lock().unwrap().price(&mut items_lock);
    }
}

pub async fn pricer_server(port: &str) -> Result<(), IoError> {
    let addr = format!("127.0.0.1:{port}");

    let buys = Arc::new(Mutex::new(vec![]));
    let pricer = Arc::new(Mutex::new(SimplePricer::new()));
    let items = Arc::new(Mutex::new(vec![
        Item::new("test1".to_string(), 1000, 100, 100),
        Item::new("test2".to_string(), 3000, 300, 50),
    ]));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    tokio::spawn(update_pricer(buys.clone(), pricer.clone()));
    tokio::spawn(reprice(items.clone(), pricer.clone()));

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(buys.clone(), items.clone(), stream, addr));
    }

    Ok(())
}
