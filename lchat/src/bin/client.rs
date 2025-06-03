use std::io::{self, BufRead, BufReader, Write, Result};
use std::net::TcpStream;
use std::thread;

fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to chat server");

    let mut write_half = stream.try_clone()?;        // 送信用
    let mut read_half = BufReader::new(stream);      // 受信用

    // 受信用スレッド
    thread::spawn(move || {
        let mut line = String::new();
        loop {
            line.clear();
            // サーバーからの入力を受け取る
            if read_half.read_line(&mut line).is_err() {
                println!("Connection closed");
                break;
            }
            
            print!("{line}");
        }
    });

    // 標準入力 → サーバーへ転送
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let mut msg = line?;
        msg.push('\n');
        write_half.write_all(msg.as_bytes())?;
    }

    Ok(())
}
