use std::io::{BufRead, BufReader, Write, Result};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

type SharedClients = Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>;

fn main() -> Result<()> {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Chat server running on 0.0.0.0:8080");
    
    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));

    // 接続待ち
    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("{} : connected", peer);

        // 共有リストに登録
        let client_stream =  Arc::new(Mutex::new(stream));
        clients.lock().unwrap().push(client_stream.clone());

        // 各クライアントを処理するスレッドを作成
        let clients_clone = Arc::clone(&clients);
        thread::spawn(move || handle_client(client_stream, clients_clone, peer));
    }

    Ok(())
}

fn handle_client(
    stream: Arc<Mutex<TcpStream>>,
    clients: SharedClients,
    peer: SocketAddr,
) {
    let reader = {
        let guard = stream.lock().unwrap();
        BufReader::new(guard.try_clone().expect("clone failed"))
    };

    for line in reader.lines() {
        let msg = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let formatted = format!("{} : {}\n", peer, msg);

        // すべてのクライアントに配信
        let clients_guard = clients.lock().unwrap();
        for client in clients_guard.iter() {
            let mut s = client.lock().unwrap();
            if let Err(e) = s.write_all(formatted.as_bytes()) {
                eprintln!("send error : {}", e);
            }
        }
    }

    clients
        .lock()
        .unwrap()
        .retain(|c| !Arc::ptr_eq(c, &stream));
    println!("{} : disconnected", peer);
}