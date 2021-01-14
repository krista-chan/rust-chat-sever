extern crate serde_json;
extern crate tokio;
extern crate tungstenite;
extern crate uuid;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use tungstenite::{
    protocol::{frame::coding, CloseFrame},
    server::accept,
    Message,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let users: Arc<RwLock<std::collections::HashMap<String, User>>> =
        Arc::new(RwLock::new(std::collections::HashMap::new()));                                    // declare a structure to hold the connections as they connect and disconnect from the server
    let _channels: std::collections::HashMap<String, Channel> = std::collections::HashMap::new();   // declare a structure to hold text channels
    let server = TcpListener::bind("127.0.0.1:9001").expect("Unable to start server");              // start the ws server (BLOCKING)
    for connection in server.incoming() {                                                           // as a connection is recieved ...
        let users = users.clone();                                                                  // ... clone users to be able to write a new user to the hashmap ...
        tokio::spawn(async move {                                                                   // ... and then spawn a new async thread with tokio
            let mut ws = accept(connection.unwrap()).unwrap();                                      // accept the ws connection (this will be our rw socket)
            ws.write_message("{\"message\":\"You connected\"}".into())                              // send a message on connect (this will arrive as a buffer)
                .unwrap();
            loop {                                                                                  // start a blocking loop ...
                let users = users.clone();                                                          // ... where users is cloned again
                let msg = ws.read_message().unwrap();                                               // since this is triggered every time a new connection occurs each thread will have its own socket
                let msg_json: WSMessage = from_str(&msg.to_text().unwrap()).unwrap();               // use serde_json to parse the incoming message to a weak typed json-like structure
                let close_opts: CloseFrame = CloseFrame {                                           // instantiate a CloseFrame incase we need to close the ws connection
                    code: coding::CloseCode::Normal,
                    reason: std::borrow::Cow::default(),
                };
                match msg_json.t {                                                                  // here we handle message opcodes
                    Opcodes::Connect => {                                                           // opcode Connect will check for lack of data ...
                        if msg_json.d == Value::Null {
                            ws.write_message(Message::Binary(
                                "{\"error\":\"missing login data\"}".into(),                        // ... and throw an error to the user ...
                            ))
                            .unwrap();
                            ws.close(Option::from(close_opts)).unwrap();                            // ... after which it will close the ws and terminate the thread
                        } else {
                            if msg_json.d["login_pkg"] == Value::Null {                             // here, if there's no data in the login_pkg field then generate two uuids for the token and the user ID
                                let token = gen_uuid();
                                let id = gen_uuid();
                                users.write().unwrap().insert(
                                    id.clone(),
                                    User::new(
                                        String::from("Unnamed user"),
                                        token.clone(),
                                        id.clone(),
                                    ),
                                );
                                ws.write_message(Message::Binary(
                                    format!(
                                        "{{\"connection\": {{\"token\":\"{}\", \"id\":\"{}\"}}}}",  // now we can send the newly generated login data to the user so that they can use it for later and whatnot
                                        token.to_owned(),
                                        id.to_owned()
                                    )
                                    .into(),
                                ))
                                .unwrap();
                                println!("{:?}", users.read().unwrap());                            // as a dev thing, log the users hashmap
                            }
                        }
                    }
                }
            }
        });
    }
}

pub fn gen_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Opcodes {
    Connect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WSMessage {
    t: Opcodes,
    d: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    id: String,
    name: String,
    token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {}

impl User {
    pub fn new(name: String, token: String, id: String) -> Self {
        User {
            name: name,
            token: token,
            id: id,
        }
    }
}
