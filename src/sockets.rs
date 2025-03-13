use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::{StreamExt, future, pin_mut, stream::TryStreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{
    models::{Buy, Item},
    pricing::{Pricer, SimplePricer},
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
type BuysState = Arc<Mutex<Vec<Buy>>>;
type PricerState = Arc<Mutex<dyn Pricer>>;
type ItemState = Arc<Mutex<Vec<Item>>>;

async fn handle_connection(
    peer_map: PeerMap,
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

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let handle_commands = incoming.try_for_each(|msg| {
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
            let peers = peer_map.lock().unwrap();
            let ws_sink = peers.get(&addr).unwrap();
            ws_sink.unbounded_send(Message::text(item_text)).unwrap();
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_commands, receive_from_others);
    future::select(handle_commands, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
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

    let state = PeerMap::new(Mutex::new(HashMap::new()));
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
        tokio::spawn(handle_connection(
            state.clone(),
            buys.clone(),
            items.clone(),
            stream,
            addr,
        ));
    }

    Ok(())
}

pub async fn connect_to_server(port: &str) {
    let url = format!("ws://127.0.0.1:{port}/");

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
