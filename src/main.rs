/*
Server for ciyun
TODO:
  - Upload qoute
  - SQL Storage
  - Response qoute request
*/

use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

const SERVER_IP: &str = "127.0.0.1";
const SERVER_PORT: &str = "23333";

static sample_qoutes: [&str; 5] = ["111", "222", "333", "444", "555"];

async fn handler_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();

    let (outgoing, incoming) = ws_stream.split();

    use rand::Rng;
    let mut rand_gen = rand::thread_rng();

    let send_msg = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );

        tx.unbounded_send(sample_qoutes[2].clone()).unwrap();

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(send_msg, receive_from_others);
    future::select(send_msg, receive_from_others).await;

    println!("{} disconnected", &addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let server_addr = format!("{}:{}", SERVER_IP, SERVER_PORT);
    // let server_state = PeerMap::new(Mutex::new(HashMap::new()));

    let try_socket = TcpListener::bind(&server_addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", server_addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        tokio::spawn(handler_connection(stream, client_addr));
    }

    Ok(())
}
