use async_std::task;
use env_logger;
use futures::StreamExt;

use async_std_utp::{UtpListener, UtpSocket};

async fn handle_client(mut s: UtpSocket) {
    let mut buf = vec![0; 1500];

    // Reply to a data packet with its own payload, then end the connection
    match s.recv_from(&mut buf).await {
        Ok((nread, src)) => {
            println!("<= [{}] {:?}", src, &buf[..nread]);
            drop(s.send_to(&buf[..nread]).await);
        }
        Err(e) => println!("{}", e),
    }
}

#[async_std::main]
async fn main() {
    // Start logger
    env_logger::init();

    // Create a listener
    let addr = "127.0.0.1:8080";
    let listener = UtpListener::bind(addr)
        .await
        .expect("Error binding listener");

    let mut stream = listener.incoming();
    while let Some(connection) = stream.next().await {
        // Spawn a new handler for each new connection
        match connection {
            Ok((socket, _src)) => {
                task::spawn(handle_client(socket));
            }
            _ => (),
        }
    }
}
