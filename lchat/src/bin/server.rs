use std::io::{BufRead, BufReader, Write, Result};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

/// クライアントごとの保持情報
struct Client {
    stream: Arc<Mutex<TcpStream>>,
    name: String,
}
/// 共有クライアントリスト
type SharedClients = Arc<Mutex<Vec<Client>>>;


fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Chat server running on 0.0.0.0:8080");

    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));

    // 接続待ち
    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("{peer} : connected");

        let client_stream = Arc::new(Mutex::new(stream));
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
    // ----- ニックネーム要求 -----
    {
        let mut guard = stream.lock().unwrap();
        if guard
            .write_all(b"Please enter your nickname: ")
            .and_then(|_| guard.flush())
            .is_err()
        {
            eprintln!("{peer} : failed to send prompt");
            return;
        }
    }

    // ----- 受信一行目をニックネームにする -----
    let nickname = {
        let guard = stream.lock().unwrap();
        let mut reader = BufReader::new(
            guard
                .try_clone()
                .expect("failed to clone stream for nickname read"),
        );
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(0) | Err(_) => {
                eprintln!("{peer} : disconnected before sending nickname");
                return;
            }
            Ok(_) => buf.trim().to_string(),
        }
    };
    if nickname.is_empty() {
        eprintln!("{peer} : empty nickname – closing");
        return;
    }

    // ---- 共有クライアントリストへ登録 ----
    {
        clients
            .lock()
            .unwrap()
            .push(Client {
                stream: Arc::clone(&stream),
                name: nickname.clone(),
            });
    }

    // ----- 入室通知 -----
    broadcast(&clients, &format!("*** {nickname} joined ***\n"));

    // ----- メッセージ転送 -----
    let reader = {
        let guard = stream.lock().unwrap();
        BufReader::new(
            guard
                .try_clone()
                .expect("failed to clone stream for main read loop"),
        )
    };
    for line in reader.lines() {
        let msg = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        broadcast(&clients, &format!("{nickname} : {msg}\n"));
    }

    // ----- 退出処理 -----
    {
        clients
            .lock()
            .unwrap()
            .retain(|c| !Arc::ptr_eq(&c.stream, &stream));
    }
    broadcast(&clients, &format!("*** {nickname} left ***\n"));
    println!("{peer} ({nickname}) disconnected");
}


fn broadcast(clients: &SharedClients, message: &str) {
    let list = clients.lock().unwrap();
    for client in list.iter() {
        if let Ok(mut s) = client.stream.lock() {
            let _ = s.write_all(message.as_bytes());
        }
    }
}
