use std::{
    io::{BufRead, BufReader, Result},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{Packet, PacketType};
use crate::server::{
    client_manager::SharedClients,
    network,
};

/// メッセージ受信とパケット処理のメインループ
pub fn handle_client_messages(
    stream: &Arc<Mutex<TcpStream>>,
    clients: &SharedClients,
    nickname: &str,
) -> Result<()> {
    let reader = {
        let guard = stream.lock().unwrap();
        BufReader::new(
            guard
                .try_clone()
                .expect("failed to clone stream for main read loop"),
        )
    };

    for line in reader.lines() {
        let msg = line?;
        let trimmed = msg.trim();
        
        if let Ok(packet) = Packet::from_json(trimmed) {
            packet.log();

            match packet.packet_type {
                PacketType::Message => handle_message(clients, &packet, nickname),
                PacketType::InfoRequest => handle_info_request(clients, &packet, stream),
                _ => Ok(()),
            }?;
        }
    }

    Ok(())
}

fn handle_info_request(
    clients: &SharedClients,
    packet: &Packet,
    stream: &Arc<Mutex<TcpStream>>
) -> Result<()> {
    match packet.content.as_str() {
        "connection" => {
            let count = clients.lock().unwrap().len();
            let connection_packet = Packet::connection(count);
            network::send_packet(&connection_packet, stream)?;
        },
        _ => {},
    }
    Ok(())
}

fn handle_message(
    clients: &SharedClients,
    packet: &Packet,
    nickname: &str
) -> Result<()> {
    let message_packet = Packet::message(packet.content.clone(), nickname.to_string());
    network::broadcast(clients, &message_packet)?;
    Ok(())
}