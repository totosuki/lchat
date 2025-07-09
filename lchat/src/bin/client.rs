use std::io::Result;

use lchat::client::{
    network::{self, NetworkManager},
    input::{InputHandler, KeyAction},
    ui::{ClientUI, AppState},
};

fn main() -> Result<()> {
    // ----- ネットワーク接続 -----
    let addr = network::get_connection_address();
    let (mut network_manager, rx) = NetworkManager::new(addr)?;

    // ----- UI初期化 -----
    let mut ui = ClientUI::new()?;
    let mut state = AppState::new()?;
    let input_handler = InputHandler::new();

    // ----- メインループ -----
    loop {
        // ----- 非ブロッキングで受信メッセージ取得 -----
        for msg in rx.try_iter() {
            state.add_message(msg);
        }

        // ----- キー入力処理 -----
        match input_handler.handle_input(&mut state)? {
            KeyAction::Exit => break,
            KeyAction::ClearChat => state.clear_chat(),
            KeyAction::SendNickname(nickname) => {
                network_manager.send_nickname(nickname)?;
            },
            KeyAction::SendMessage(message) => {
                if let Some(ref nickname) = state.nickname {
                    network_manager.send_message(message, nickname.clone())?;
                }
            },
            KeyAction::InfoRequest => {
                network_manager.send_info_request("connection".to_string())?;
            },
            KeyAction::None => {},
        }

        // ----- 画面描画 -----
        ui.render(&state)?;
    }

    // ----- 後処理 -----
    ui.cleanup()?;
    Ok(())
}