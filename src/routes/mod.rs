pub mod vote;
pub mod server_config;
pub mod login;
pub mod server_tables;


use rouille::Response;
use serde::{Serialize, Deserialize};


#[derive(Deserialize, Serialize)]
struct ReturnMsg {
    message: String
}


impl ReturnMsg {
    pub fn with(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}


pub trait MessageJson {
    fn message_json(with: &str) -> Self;
}


impl MessageJson for Response {
    fn message_json(message: &str) -> Self {
        Response::json(&ReturnMsg { message: message.to_string() })
    }
}