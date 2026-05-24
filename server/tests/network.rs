use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};
use tty_mate_server::run_server;

async fn spawn_test_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
 
    tokio::spawn(async move {
        let _ = run_server(listener).await;
    });

    port
}

#[tokio::test]
async fn matchmaking_handshake() {
    let port = spawn_test_server().await;
    let addr = format!("127.0.0.1:{}", port);

    let mut client1 = TcpStream::connect(&addr).await.unwrap();
    let (reader1, _) = client1.split();
    let mut reader1 = BufReader::new(reader1).lines();

    let mut client2 = TcpStream::connect(&addr).await.unwrap();
    let (reader2, _) = client2.split();
    let mut reader2 = BufReader::new(reader2).lines();

    let msg1 = reader1.next_line().await.unwrap().unwrap();
    let msg2 = reader2.next_line().await.unwrap().unwrap();

    // If the handshake is successful, both clients should receive a message indicating their color and game ID
    assert_eq!(msg1, "s:0:w");
    assert_eq!(msg2, "s:0:b");
}

#[tokio::test]
async fn gameplay_validation() {
    let port = spawn_test_server().await;
    let addr = format!("127.0.0.1:{}", port);

    let mut client1 = TcpStream::connect(&addr).await.unwrap();
    let (reader1, mut writer1) = client1.split();
    let mut reader1 = BufReader::new(reader1).lines();

    let mut client2 = TcpStream::connect(&addr).await.unwrap();
    let (reader2, mut writer2) = client2.split();
    let mut reader2 = BufReader::new(reader2).lines();

    reader1.next_line().await.unwrap().unwrap();
    reader2.next_line().await.unwrap().unwrap();

    writer2.write_all(b"m:51:35\n").await.unwrap();

    let error_msg = reader2.next_line().await.unwrap().unwrap();
    assert_eq!(error_msg, "e:m");

    writer1.write_all(b"m:12:28\n").await.unwrap();

    let relayed_move = reader2.next_line().await.unwrap().unwrap();
    assert_eq!(relayed_move, "m:12:28");
}
