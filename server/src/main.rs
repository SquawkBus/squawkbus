use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;
use interactor::interactor_run;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);
    let mut hub = Hub::new();

    // Create a hub that listens to clients
    tokio::spawn(async move {
            hub.run(server_rx).await;
        }
    );

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let client_tx = client_tx.clone();

        tokio::spawn(async move {
            interactor_run(socket, addr, client_tx).await;
        });
    }
}
