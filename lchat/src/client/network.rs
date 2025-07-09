use std::{
    env,
    io::{BufRead, BufReader, Write, Result},
    net::TcpStream,
    sync::mpsc,
    thread,
};

use crate::{Packet, PacketType};

/// ネットワーク管理
pub struct NetworkManager {
    writer: TcpStream,
}

impl NetworkManager {
    pub fn new(addr: String) -> Result<(Self, mpsc::Receiver<String>)> {
        let tcp = TcpStream::connect(addr)?;
        let writer = tcp.try_clone()?;
        let reader = tcp;

        // 受信用スレッド
        let (tx, rx) = mpsc::channel::<String>();
        recv_thread(reader, tx);

        Ok((Self { writer }, rx))
    }

    pub fn send_message(&mut self, message: String, nickname: String) -> Result<()> {
        let packet = Packet::message(message, nickname);
        self.send_packet(&packet)
    }
    
    pub fn send_nickname(&mut self, nickname: String) -> Result<()> {
        let packet = Packet::nickname_response(nickname);
        self.send_packet(&packet)
    }

    pub fn send_info_request(&mut self, info: String) -> Result<()> {
        let packet = Packet::info_request(info);
        self.send_packet(&packet)
    }

    fn send_packet(&mut self, packet: &Packet) -> Result<()> {
        if let Ok(json) = packet.to_json() {
            self.writer.write_all(format!("{}\n", json).as_bytes())?;
        }
        Ok(())
    }
}

/// コマンドライン引数から接続先を取得
pub fn get_connection_address() -> String {
    let args: Vec<String> = env::args().collect();
    let iport = if args.len() >= 2 {
        match args[1].find(':') {
            Some(_) => args[1].clone(),
            None => format!("{}:8080", &args[1]),
        }
    } else {
        "localhost:8080".to_string()
    };
    let ip = iport.split(':').next().unwrap();
    let port = iport.split(':').nth(1).unwrap();
    format!("{}:{}", ip, port)
}

/// 受信スレッドを開始
pub fn recv_thread(reader: TcpStream, tx: mpsc::Sender<String>) {
    thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            buf_reader.read_line(&mut line).unwrap();

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(packet) = Packet::from_json(trimmed) {
                match packet.packet_type {
                    PacketType::NicknameRequest => {
                        tx.send(packet.content).ok();
                    },
                    PacketType::NicknameResponse => {
                        // ニックネーム応答は通常表示しない
                    },
                    PacketType::Message => {
                        if let Some(nickname) = packet.nickname {
                            tx.send(format!("{} : {}", nickname, packet.content)).ok();
                        }
                    },
                    PacketType::Join | PacketType::Leave => {
                        tx.send(packet.content).ok();
                    },
                    PacketType::InfoRequest => {
                        eprintln!("This is the packet type sent by the client.")
                    },
                    PacketType::Connection => {
                        tx.send(format!("Connected clients: {}", packet.content)).ok();
                    },
                    PacketType::Error => {
                        tx.send(format!("Error: {}", packet.content)).ok();
                    },
                }
            } else {
                eprintln!("Failed to parse JSON: {:?}", trimmed);
            }
        }
    });
}