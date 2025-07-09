use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

/// クライアントごとの保持情報
pub struct Client {
    pub stream: Arc<Mutex<TcpStream>>,
    pub name: String,
}

/// 共有クライアントリスト
pub type SharedClients = Arc<Mutex<Vec<Client>>>;

/// 新しいSharedClientsインスタンスを作成
pub fn create_clients_list() -> SharedClients {
    Arc::new(Mutex::new(Vec::new()))
}

/// クライアントを追加
pub fn add_client(clients: &SharedClients, stream: Arc<Mutex<TcpStream>>, name: String) {
    clients.lock().unwrap().push(Client { stream, name });
}

/// クライアントを削除
pub fn remove_client(clients: &SharedClients, stream: &Arc<Mutex<TcpStream>>) {
    clients
        .lock()
        .unwrap()
        .retain(|c| !Arc::ptr_eq(&c.stream, stream));
}

/// 接続中のクライアント数を取得
pub fn get_client_count(clients: &SharedClients) -> usize {
    clients.lock().unwrap().len()
}