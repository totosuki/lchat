use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PacketType {
    NicknameRequest,
    NicknameResponse,
    Message,
    Join,
    Leave,
    InfoRequest,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub content: String,
    pub nickname: Option<String>,
}

impl Packet {
    pub fn new(packet_type: PacketType, content: String, nickname: Option<String>) -> Self {
        Self {
            packet_type,
            content,
            nickname,
        }
    }

    pub fn nickname_request() -> Self {
        Self::new(PacketType::NicknameRequest, "Please enter your nickname: ".to_string(), None)
    }

    pub fn nickname_response(nickname: String) -> Self {
        Self::new(PacketType::NicknameResponse, nickname, None)
    }

    pub fn message(content: String, nickname: String) -> Self {
        Self::new(PacketType::Message, content, Some(nickname))
    }

    pub fn join(nickname: String) -> Self {
        Self::new(PacketType::Join, format!("*** {} joined ***", nickname), Some(nickname))
    }

    pub fn leave(nickname: String) -> Self {
        Self::new(PacketType::Leave, format!("*** {} left ***", nickname), Some(nickname))
    }

    pub fn info_request(info: String) -> Self {
        Self::new(PacketType::InfoRequest, format!("{}", info), None)
    } 

    pub fn error(content: String) -> Self {
        Self::new(PacketType::Error, content, None)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn log(&self) {
        match self.packet_type {
            PacketType::NicknameRequest => println!("[NicknameRequest] : {}", self.content),
            PacketType::NicknameResponse => println!("[NicknameResponse] : {}", self.content),
            PacketType::Message => println!("[Message] {} : {}", self.nickname.as_deref().unwrap_or("Unknown"), self.content),
            PacketType::Join => println!("[Join] {}", self.content),
            PacketType::Leave => println!("[Leave] {}", self.content),
            PacketType::InfoRequest => println!("[InfoRequest] {}", self.content),
            PacketType::Error => eprintln!("[Error] {}", self.content),
        }
    }
}