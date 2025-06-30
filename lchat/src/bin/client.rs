use std::{
    env,
    io::{self, BufRead, BufReader, Result, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::Duration
};

use lchat::{Packet, PacketType};
use crossterm::{
    cursor::{MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    QueueableCommand,
};

fn get_arguments() -> String {
    let args: Vec<String> = env::args().collect();
    let iport = if args.len() >= 2 {
        match args[1].find(":") {
            Some(index) => args[1].clone(),
            None => format!("{}:8080", &args[1]),
        }
    } else {
        "localhost:8080".to_string()
    };
    let ip = iport.split(":").next().unwrap();
    let port = iport.split(":").skip(1).next().unwrap();
    format!("{}:{}", ip, port)
}

fn recv_thread(reader: TcpStream, tx: mpsc::Sender<String>) {
    thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            if buf_reader.read_line(&mut line).unwrap_or(0) == 0 { break; }
            
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
                eprintln!("Failed to parse JSON: {:?}", trimmed); // デバッグ用
            }
        }
    });
}

fn main() -> Result<()> {
    // ----- コマンドライン引数から取得 -----
    let addr = get_arguments();

    // ----- TCP接続 -----
    let tcp = TcpStream::connect(addr)?;
    let mut writer = tcp.try_clone()?; // 送信用
    let mut reader = tcp;              // 受信用

    // ----- 端末をrawにする -----
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;

    // ----- 受信用 -----
    let (tx, rx) = mpsc::channel::<String>();
    recv_thread(reader, tx);

    // ----- UI メインループ -----
    let (mut cols, mut rows) = terminal::size()?; // 初期サイズ
    let mut chat_lines: Vec<String> = Vec::new();           // 画面表示用バッファ
    let mut input = String::new();                  // ユーザー入力
    let mut cursor = 0usize;                         // 入力カーソル位置
    let mut nickname: Option<String> = None;                // ニックネーム

    loop {
        // ----- 非ブロッキングで受信メッセージ取得 -----
        for msg in rx.try_iter() {
            chat_lines.push(msg);
            let limit = rows.saturating_sub(1) as usize; // 最下行は入力用
            if chat_lines.len() > limit {
                chat_lines.drain(.. chat_lines.len()-limit);
            }
        }

        // ----- キー入力処理 -----
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
                    // Windows で二重入力を防ぐため、KeyPressのみ処理
                    if kind != KeyEventKind::Press {
                        continue;
                    }
                    match (code, modifiers) {
                        // 終了系
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) |
                        (KeyCode::Esc, _) => break,

                        // 移動系
                        (KeyCode::Left, _) => { if cursor > 0 { cursor -= 1; } },
                        (KeyCode::Right, _) => { if cursor < input.len() { cursor += 1; } },

                        // 編集系
                        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                            chat_lines = Vec::new();
                        },
                        (KeyCode::Backspace, _) => { 
                            if cursor > 0 {
                                cursor -= 1;
                                input.remove(cursor);
                            }
                        },
                        (KeyCode::Char(c), _) => {
                            input.insert(cursor, c);
                            cursor += 1;
                        },
                        (KeyCode::Enter, _) => {
                            if nickname == None {
                                // ニックネーム送信
                                nickname = Some(input.clone());
                                let packet = Packet::nickname_response(input.clone());
                                if let Ok(json) = packet.to_json() {
                                    writer.write_all(format!("{}\n", json).as_bytes())?;
                                }
                            } else {
                                // メッセージ送信
                                let packet = Packet::message(input.clone(), nickname.clone().unwrap());
                                if let Ok(json) = packet.to_json() {
                                    writer.write_all(format!("{}\n", json).as_bytes())?;
                                }
                            }
                            input.clear();
                            cursor = 0;
                        },
                        (KeyCode::Tab, _) => {
                            let packet = Packet::info_request(format!("connection"));
                            if let Ok(json) = packet.to_json() {
                                writer.write_all(format!("{}\n", json).as_bytes())?;
                            }
                        },
                        _ => {}
                    }
                },
                Event::Resize(c, r) => { cols = c; rows = r; },
                _ => {}
            }
        }

        // ----- 画面全体を再描画 -----
        stdout.queue(Clear(ClearType::All))?;

        // ----- チャット表示 -----
        let chat_height = rows.saturating_sub(1);
        for (i, line) in chat_lines.iter().rev().enumerate() {
            if i as u16 >= chat_height { break; }
            let y = chat_height - 1 - i as u16;
            stdout.queue(MoveTo(0, y))?
                  .queue(Print(line.trim_end()))?;
        }

        // ----- 入力行描画 -----
        let prompt = "> ";
        stdout.queue(MoveTo(0, rows - 1))?
              .queue(Print(format!("{prompt}{input}")))?;
        
        // ----- カーソル位置調整 -----
        let cursor_x = prompt.len() as u16 + cursor as u16;
        stdout.queue(MoveTo(cursor_x, rows - 1))?;

        stdout.flush()?;
    }

    // ----- 後処理 -----
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
