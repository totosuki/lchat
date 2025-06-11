use std::{
    io::{self, BufRead, BufReader, Read, Result, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::Duration,
    env,
};

use crossterm::{
    cursor::{MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    QueueableCommand,
};

fn main() -> Result<()> {
    // ----- コマンドライン引数から取得 -----
    let mut args = env::args().skip(1);
    let addr_str = args.next().unwrap_or_else(|| {
        eprintln!("Usage: client <IP>[:PORT]");
        std::process::exit(1);
    });
    let addr = if addr_str.contains(':') {
        addr_str
    } else {
        format!("{addr_str}:8080")
    };

    // ----- TCP接続 -----
    let tcp = TcpStream::connect(&addr)?;
    let mut writer = tcp.try_clone()?; // 送信用
    let mut reader = tcp;              // 受信用

    // ----- 端末をrawにする -----
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;

    // ----- 受信用 -----
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let mut prompt = Vec::new();
        let mut byte   = [0u8; 1];
        loop {
            if reader.read_exact(&mut byte).is_err() { return; }
            prompt.push(byte[0]);
            if prompt.ends_with(b": ") { break; }
        }
        tx.send(String::from_utf8_lossy(&prompt).into()).ok();

        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            if buf_reader.read_line(&mut line).unwrap_or(0) == 0 { break; }
            tx.send(line.clone()).ok();
        }
    });

    // ----- UI メインループ -----
    let (mut cols, mut rows) = terminal::size()?; // 初期サイズ
    let mut chat_lines: Vec<String> = Vec::new();           // 画面表示用バッファ
    let mut input = String::new();                  // ユーザー入力
    let mut cursor = 0usize;                         // 入力カーソル位置

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
                Event::Key(KeyEvent { code, modifiers, .. }) => match (code, modifiers) {
                    // 終了系
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) |
                    (KeyCode::Esc, _) => break,

                    // 移動系
                    (KeyCode::Left, _) => { if cursor > 0 { cursor -= 1; } },
                    (KeyCode::Right, _) => { if cursor < input.len() { cursor += 1; } },

                    // 編集系
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
                        writer.write_all(format!("{}\n", input).as_bytes())?;
                        input.clear();
                        cursor = 0;
                    }
                    _ => {}
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
