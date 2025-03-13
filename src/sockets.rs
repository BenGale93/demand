use std::{
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    time,
};
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};

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
        if msg_text.contains("buy") {
            buys.lock().unwrap().push(Buy {
                name: "test1".to_string(),
                cost: 100,
            });
        }
        if msg_text.contains("view") {
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

pub async fn connect_to_server(port: &str) {
    let url = format!("ws://127.0.0.1:{port}/");

    println!("Connecting to - {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    // Handle incoming messages in a separate task
    let read_handle = tokio::spawn(handle_incoming_messages(read));

    // Read from command line and send messages
    let write_handle = tokio::spawn(read_and_send_messages(write));

    // Await both tasks (optional, depending on your use case)
    let _ = tokio::try_join!(read_handle, write_handle);
}

async fn handle_incoming_messages(
    mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}

async fn read_and_send_messages(
    mut write: SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
) {
    let mut reader = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        if !line.trim().is_empty() {
            write
                .send(Message::Text(line.into()))
                .await
                .expect("Failed to send message");
        }
    }
}
