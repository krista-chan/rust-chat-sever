extern crate tungstenite;
extern crate tokio;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use tungstenite::{Message, server::accept, protocol::{frame::coding, CloseFrame}};
use serde_json::{from_str, Value};

#[tokio::main]
async fn main() {
    let users: std::collections::HashMap<&str, User> = std::collections::HashMap::new();
    let server = TcpListener::bind("127.0.0.1:9001").expect("Unable to start server");
    for connection in server.incoming() {
        tokio::spawn(async move {
            let mut ws = accept(connection.unwrap()).unwrap();
            ws.write_message("You connected".into()).unwrap();
            loop {
                let msg = ws.read_message().unwrap();
                let msg_json: WSMessage = from_str(&msg.to_text().unwrap()).unwrap();
                let close_opts: CloseFrame = CloseFrame {code: coding::CloseCode::Normal, reason: std::borrow::Cow::default()};
                match msg_json.opcode {
                    Opcodes::Connect => {
                        if msg_json.data["login_pkg"] == Value::Null {
                            ws.write_message(Message::Binary("{\"error\":\"missing login data\"}".into())).unwrap();
                            ws.close(Option::from(close_opts)).unwrap();
                        } else {
                            println!("{}", msg_json.data["login_pkg"]);
                        }
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Opcodes {
    Connect,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WSMessage {
    opcode: Opcodes,
    data: serde_json::Value
}

pub struct User {

}
