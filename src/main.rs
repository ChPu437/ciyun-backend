/*
Server for ciyun
TODO:
  - Upload qoute
  - SQL Storage
  * Response qoute request
*/

use std::{io::Error as IoError, net::SocketAddr};

use futures_channel::mpsc::unbounded;
use futures_util::{future, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};

use rand::Rng;

const LISTEN_IP: &str = "127.0.0.1";
const LISTEN_PORT: &str = "23333";

static SAMPLE_QOUTES: [&str; 5] = ["111", "222", "333", "444", "555"];

async fn handler_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (tx, _rx) = unbounded();

    let (_outgoing, incoming) = ws_stream.split();

    incoming
        .try_for_each(|msg| {
            println!(
                "Received a message from {}: {}",
                addr,
                msg.to_text().unwrap()
            );

            let mut rand = rand::thread_rng();

            let msg_send = SAMPLE_QOUTES[rand.gen_range(0..5)].clone();
            tx.unbounded_send(msg_send).unwrap();
            println!("Send message to {}: {}", addr, msg_send);

            future::ok(())
        })
        .await
        .unwrap();

    println!("{} disconnected", &addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = format!("{}:{}", LISTEN_IP, LISTEN_PORT);
    // let server_state = PeerMap::new(Mutex::new(HashMap::new()));

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handler_connection(stream, addr));
    }

    Ok(())
}
