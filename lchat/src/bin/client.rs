use std::io::{self, BufRead, BufReader, Read, Result, Write};
use std::net::TcpStream;
use std::thread;

fn main() -> Result<()> {
    // ----- initialization -----
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    let mut read_stream = stream.try_clone()?; // 受信用
    let mut write_stream = stream;         // 送信用

    // ----- サーバーからプロンプトを１行受信 -----
    let mut prompt= Vec::<u8>::new();
    let mut byte = [0u8; 1];
    loop {
        if read_stream.read(&mut byte)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "server closed before sending prompt",
            ));
        }
        prompt.push(byte[0]);

        // 末尾が “: ”（0x3A 0x20）になったら終了
        if prompt.ends_with(b": ") {
            break;
        }
    }
    print!("{}", String::from_utf8_lossy(&prompt));
    io::stdout().flush()?; 

    // ----- ニックネーム入力 -----
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim();
    if nickname.is_empty() {
        println!("Nickname cannot be empty.");
        return Ok(());
    }
    write_stream.write_all(format!("{nickname}\n").as_bytes())?;

    // ----- 受信用スレッド -----
    let read_stream = write_stream.try_clone()?;
    thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => {
                    println!("Server closed connection");
                    break;
                }
                Ok(_) => println!("{buf}"),
                Err(e) => {
                    println!("Read error: {e}");
                    break;
                }
            }
        }
    });

    // ----- 標準入力 -----
    println!("Type messages ( /quit で終了)");
    for line in io::stdin().lock().lines() {
        let mut msg = line?;
        if msg.trim() == "/quit" {
            break;
        }
        msg.push('\n');
        write_stream.write_all(msg.as_bytes())?;
    }

    Ok(())
}
