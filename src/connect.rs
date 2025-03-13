use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncWrite};
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};

pub async fn connect_to_server(port: &str) {
    let url = format!("ws://127.0.0.1:{port}/");

    println!("Connecting to - {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    // Handle incoming messages in a separate task
    let read_handle = tokio::spawn(handle_incoming_messages(read));

    // Read from command line and send messages
    let write_handle = tokio::spawn(read_and_send_messages(write));

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
