use std::io::{self, Write, Result};
use crossterm::{
    cursor::MoveTo,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    QueueableCommand,
};

/// アプリケーションの状態
pub struct AppState {
    pub chat_lines: Vec<String>,
    pub input: String,
    pub cursor: usize,
    pub nickname: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let (cols, rows) = terminal::size()?;
        Ok(Self {
            chat_lines: Vec::new(),
            input: String::new(),
            cursor: 0,
            nickname: None,
            cols,
            rows,
        })
    }

    pub fn add_message(&mut self, message: String) {
        self.chat_lines.push(message);
        let limit = self.rows.saturating_sub(1) as usize;
        if self.chat_lines.len() > limit {
            self.chat_lines.drain(..self.chat_lines.len() - limit);
        }
    }

    pub fn update_size(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub fn clear_chat(&mut self) {
        self.chat_lines.clear();
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    pub fn take_input(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        self.cursor = 0;
        input
    }
}

/// ターミナルUI管理
pub struct ClientUI {
    stdout: io::Stdout,
}

impl ClientUI {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Clear(ClearType::All))?;
        
        Ok(Self { stdout })
    }

    pub fn render(&mut self, state: &AppState) -> Result<()> {
        // 画面全体を再描画
        self.stdout.queue(Clear(ClearType::All))?;

        // チャット表示
        let chat_height = state.rows.saturating_sub(1);
        for (i, line) in state.chat_lines.iter().rev().enumerate() {
            if i as u16 >= chat_height {
                break;
            }
            let y = chat_height - 1 - i as u16;
            self.stdout
                .queue(MoveTo(0, y))?
                .queue(Print(line.trim_end()))?;
        }

        // 入力行描画
        let prompt = "> ";
        self.stdout
            .queue(MoveTo(0, state.rows - 1))?
            .queue(Print(format!("{prompt}{}", state.input)))?;

        // カーソル位置調整
        let cursor_x = prompt.len() as u16 + state.cursor as u16;
        self.stdout.queue(MoveTo(cursor_x, state.rows - 1))?;

        self.stdout.flush()?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.stdout.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}