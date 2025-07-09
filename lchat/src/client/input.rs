use std::{time::Duration, io::Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};

use crate::client::AppState;

/// キーボード入力に対するアクション
#[derive(Debug)]
pub enum KeyAction {
    Exit,
    ClearChat,
    SendMessage(String),
    SendNickname(String),
    InfoRequest,
    None,
}

/// キーボード入力処理
pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_input(&self, state: &mut AppState) -> Result<KeyAction> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
                    // Windows で二重入力を防ぐため、KeyPressのみ処理
                    if kind != KeyEventKind::Press {
                        return Ok(KeyAction::None);
                    }
                    
                    match (code, modifiers) {
                        // 終了系
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Ok(KeyAction::Exit),
                        (KeyCode::Esc, _) => Ok(KeyAction::Exit),

                        // 移動系
                        (KeyCode::Left, _) => {
                            state.move_cursor_left();
                            Ok(KeyAction::None)
                        },
                        (KeyCode::Right, _) => {
                            state.move_cursor_right();
                            Ok(KeyAction::None)
                        },

                        // 編集系
                        (KeyCode::Char('l'), KeyModifiers::CONTROL) => Ok(KeyAction::ClearChat),
                        (KeyCode::Backspace, _) => {
                            state.backspace();
                            Ok(KeyAction::None)
                        },
                        (KeyCode::Char(c), _) => {
                            state.insert_char(c);
                            Ok(KeyAction::None)
                        },

                        // 送信系
                        (KeyCode::Enter, _) => {
                            let input = state.take_input();
                            if state.nickname.is_none() {
                                state.nickname = Some(input.clone());
                                Ok(KeyAction::SendNickname(input))
                            } else {
                                Ok(KeyAction::SendMessage(input))
                            }
                        },
                        (KeyCode::Tab, _) => Ok(KeyAction::InfoRequest),

                        _ => Ok(KeyAction::None),
                    }
                },
                Event::Resize(c, r) => {
                    state.update_size(c, r);
                    Ok(KeyAction::None)
                },
                _ => Ok(KeyAction::None),
            }
        } else {
            Ok(KeyAction::None)
        }
    }
}