/*
Server for ciyun
TODO:
  - Upload qoute
  * SQL Storage
  - password encryption
  * Response qoute request
*/

use futures_util::lock::Mutex;
use std::sync::Arc;
use std::{io::Error as IoError, net::SocketAddr};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;

use sqlx::{postgres::PgPoolOptions, Postgres, Row};

use rand::Rng;

const SERVER_LISTEN_IP: &str = "127.0.0.1";
const SERVER_LISTEN_PORT: &str = "23333";

const POSTGRES_USERNAME: &str = "postgres";
const POSTGRES_PASSWORD: &str = "fillme";
const POSTGRES_HOST: &str = "localhost";
const POSTGRES_DBNAME: &str = "ciyun";

// static SAMPLE_QOUTES: [&str; 5] = ["111", "222", "333", "444", "555"];

#[tokio::main]
async fn main() -> Result<(), IoError> {
    // setup a connection pool for postgresql
    println!("Starting server...");
    print!("[INFO] Connecting to Database...");
    let postgres_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(
            format!(
                "postgres://{}:{}@{}/{}",
                POSTGRES_USERNAME, POSTGRES_PASSWORD, POSTGRES_HOST, POSTGRES_DBNAME
            )
            .as_str(),
        )
        .await
        .unwrap();
    let postgres_pool = Arc::new(Mutex::new(postgres_pool));
    println!("Success");

    print!("[INFO] Setting-up websocket interface...");
    // setup websocket service and listen connection on pre-defined address
    let addr = format!("{}:{}", SERVER_LISTEN_IP, SERVER_LISTEN_PORT);
    // let server_state = PeerMap::new(Mutex::new(HashMap::new()));

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Success");

    println!("Server started succesfully.");
    println!("[INFO] Server now listening on: {}", addr);

    // We need spawn a dedicated instance for every request, async-ly.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handler_connection(stream, addr, postgres_pool.clone()));
    }

    Ok(())
}

async fn handler_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    pool: Arc<Mutex<sqlx::Pool<Postgres>>>,
) {
    println!("[INFO] TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (mut msg_send, mut msg_recv) = ws_stream.split();

    let msg = msg_recv.next().await.unwrap().unwrap().to_string();
    println!("[INFO] Received a message from {}: {}", addr, msg);
    // TODO: use match exp to deal with variable request

    // // To get a qoute
    let pool = pool.lock().await;
    let qoute_count = sqlx::query("SELECT COUNT(*) FROM qoute;")
        .fetch_one(&*pool)
        .await
        .unwrap()
        .try_get::<i64, _>("count")
        .unwrap()
        + 1;

    // ThreadRng cannot be send through threads safely, thus we use OsRng instead.
    let mut rand = rand::rngs::OsRng::default();
    let msg = String::from(
        sqlx::query(
            format!(
                "SELECT qoute_text FROM qoute WHERE qoute_id = {}",
                rand.gen_range(1..qoute_count)
            )
            .as_str(),
        )
        .fetch_one(&*pool)
        .await
        .unwrap()
        .try_get::<&str, _>("qoute_text")
        .unwrap(),
    );
    drop(pool); // remember to unlock the Mutex

    msg_send.send(Message::Text(msg.clone())).await.unwrap();
    println!("Send message to {}: {}", addr, msg);

    // when using "for_each", we need to manually close the connection
    // or to use one shot instead?

    println!("{} disconnected", &addr);
}
