use std::{
    io::{BufRead, BufReader, Write, Result, ErrorKind, Error},
    net::{TcpStream, SocketAddr},
    sync::{Arc, Mutex},
};

use crate::{Packet, PacketType};
use crate::server::{
    client_manager::SharedClients,
};

pub fn broadcast(clients: &SharedClients, packet: &Packet) -> Result<()> {
    let list = clients.lock().unwrap();
    for client in list.iter(){
        if let Err(e) = send_packet(packet, &client.stream) {
            eprintln!("[message_handler] Failed to send to client {} : {}", client.name, e);
        }
    }
    Ok(())
}

/// クライアントからニックネームを取得
pub fn get_nickname(stream: &Arc<Mutex<TcpStream>>) -> Result<String> {
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

/// ニックネーム要求
pub fn request_nickname(stream: &Arc<Mutex<TcpStream>>, _peer: SocketAddr) -> Result<()> {
    let packet = Packet::nickname_request();
    send_packet(&packet, stream)?;
    Ok(())
}

/// パケットをクライアントに送信
pub fn send_packet(packet: &Packet, stream: &Arc<Mutex<TcpStream>>) -> Result<()> {
    let mut guard = stream.lock().unwrap();
    if let Ok(json) = packet.to_json() {
        guard.write_all(format!("{}\n", json).as_bytes())?;
        guard.flush()?;
    }
    Ok(())
}