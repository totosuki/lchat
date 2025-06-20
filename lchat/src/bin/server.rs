use std::{
    io::{BufRead, BufReader, Write, Result, ErrorKind, Error},
    net::{TcpListener, TcpStream, SocketAddr},
    sync::{Arc, Mutex},
    thread,
    env
};

use lchat::{Packet, PacketType};

/// クライアントごとの保持情報
struct Client {
    stream: Arc<Mutex<TcpStream>>,
    name: String,
}
/// 共有クライアントリスト
type SharedClients = Arc<Mutex<Vec<Client>>>;


fn main() -> Result<()> {
    // コマンドライン引数
    let args: Vec<String> = env::args().collect();
    let port = if args.len() >= 2 {
        &args[1]
    } else {
        "8080"
    };

    // Bind
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("Chat server running on 0.0.0.0:{port}");

    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));

    // 接続待ち
    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("{peer} : connected");

        let client_stream  = Arc::new(Mutex::new(stream));
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
    match request_nickname(&stream, peer) {
        Ok(_) => (),
        Err(e) => eprintln!("{peer} : failed to request nickname: {e}"),
    };

    // ----- 受信一行目をニックネームにする -----
    let nickname = match get_nickname(&stream) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} : failed to request nickname: {}", peer, e);
            return;
        }
    };

    // ---- 共有クライアントリストへ登録 ----
    clients.lock().unwrap().push(Client {
        stream: Arc::clone(&stream),
        name: nickname.clone(),
    });

    // ----- 入室通知 -----
    let join_packet = Packet::join(nickname.clone());
    broadcast(&clients, &join_packet);

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

        let trimmed = msg.trim();
        if let Ok(packet) = Packet::from_json(trimmed) {
            if packet.packet_type == PacketType::Message {
                let message_packet = Packet::message(packet.content, nickname.clone());
                broadcast(&clients, &message_packet);
            }
        }
    }

    // ----- 退出処理 -----
    clients
        .lock()
        .unwrap()
        .retain(|c| !Arc::ptr_eq(&c.stream, &stream));
    let leave_packet = Packet::leave(nickname.clone());
    broadcast(&clients, &leave_packet);
    println!("{peer} ({nickname}) disconnected");
}


fn broadcast(clients: &SharedClients, packet: &Packet) {
    packet.log(); // ログ出力

    let list = clients.lock().unwrap();
    if let Ok(json) = packet.to_json() {
        let message = format!("{}", json);
        for client in list.iter() {
            if let Ok(mut s) = client.stream.lock() {
                let _ = s.write_all(message.as_bytes());
            }
        }
    }
}

fn request_nickname(stream: &Arc<Mutex<TcpStream>>, peer: SocketAddr) -> Result<()> {
    let packet = Packet::nickname_request();
    let mut guard = stream.lock().unwrap();
    if let Ok(json) = packet.to_json() {
        guard.write_all(format!("{}", json).as_bytes())?;
        guard.flush()?;
    }
    Ok(())
}

fn get_nickname(stream: &Arc<Mutex<TcpStream>>) -> Result<String> {
    let guard = stream.lock().unwrap();
    let mut reader = BufReader::new(
        guard
            .try_clone()
            .expect("failed to clone stream for nickname read"),
    );
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    
    let trimmed = buf.trim();
    let packet = Packet::from_json(trimmed)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid JSON packet"))?;
    
    if packet.packet_type == PacketType::NicknameResponse {
        if packet.content.is_empty() {
            Err(Error::new(ErrorKind::InvalidInput, "nickname is empty"))
        } else {
            Ok(packet.content)
        }
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "invalid packet type for nickname"))
    }
}