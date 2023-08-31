/*
Server for ciyun
TODO:
  - Upload qoute
  - SQL Storage
  * Response qoute request
*/

use std::{io::Error as IoError, net::SocketAddr};

use futures_util::{SinkExt, StreamExt};
use tungstenite::Message;

use tokio::net::{TcpListener, TcpStream};

use rand::Rng;

const LISTEN_IP: &str = "127.0.0.1";
const LISTEN_PORT: &str = "23333";

static SAMPLE_QOUTES: [&str; 5] = ["111", "222", "333", "444", "555"];

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = format!("{}:{}", LISTEN_IP, LISTEN_PORT);
    // let server_state = PeerMap::new(Mutex::new(HashMap::new()));

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // We need spawn a dedicated instance for every request, async-ly.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handler_connection(stream, addr));
    }

    Ok(())
}

async fn handler_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (mut msg_send, mut msg_recv) = ws_stream.split();

    let msg = msg_recv.next().await.unwrap().unwrap().to_string();
    println!("Received a message from {}: {}", addr, msg);

    // ThreadRng cannot be send through threads safely, thus we use OsRng instead.
    let mut rand = rand::rngs::OsRng::default();
    let msg = SAMPLE_QOUTES[rand.gen_range(0..5)].to_string();

    // TODO: use match exp to deal with variable request
    msg_send.send(Message::Text(msg.clone())).await.unwrap();
    println!("Send message to {}: {}", addr, msg);

    // when using "for_each", we need to manually close the connection
    // or to use one shot instead?

    println!("{} disconnected", &addr);
}
