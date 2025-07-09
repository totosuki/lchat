use std::{
    io::Result,
    net::{TcpListener, SocketAddr},
    sync::Arc,
    thread,
    env
};

use lchat::Packet;
use lchat::server::{
    client_manager::{self, SharedClients},
    message_handler,
    network,
};

fn main() -> Result<()> {
    // コマンドライン引数
    let args: Vec<String> = env::args().collect();
    let port = if args.len() >= 2 { &args[1] } else { "8080" };

    // Bind
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("Chat server running on 0.0.0.0:{port}");

    let clients = client_manager::create_clients_list();

    // 接続待ち
    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("{peer} : connected");

        let client_stream = Arc::new(std::sync::Mutex::new(stream));
        let clients_clone = Arc::clone(&clients);
        thread::spawn(move || handle_client(client_stream, clients_clone, peer));
    }

    Ok(())
}

fn handle_client(
    stream: Arc<std::sync::Mutex<std::net::TcpStream>>,
    clients: SharedClients,
    peer: SocketAddr,
) -> Result<()> {
    // ----- ニックネーム要求 -----
    match network::request_nickname(&stream, peer) {
        Ok(_) => (),
        Err(e) => eprintln!("{peer} : failed to request nickname: {e}"),
    };

    // ----- 受信一行目をニックネームにする -----
    let nickname = network::get_nickname(&stream)?;

    // ---- 共有クライアントリストへ登録 ----
    client_manager::add_client(&clients, Arc::clone(&stream), nickname.clone());

    // ----- 入室通知 -----
    let join_packet = Packet::join(nickname.clone());
    network::broadcast(&clients, &join_packet)?;

    // ----- メッセージ転送 -----
    message_handler::handle_client_messages(&stream, &clients, &nickname)?;

    // ----- 退出処理 -----
    client_manager::remove_client(&clients, &stream);
    let leave_packet = Packet::leave(nickname.clone());
    network::broadcast(&clients, &leave_packet)?;
    println!("{peer} ({nickname}) disconnected");

    Ok(())
}