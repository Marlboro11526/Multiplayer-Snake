use std::collections::HashSet;

use crate::server::Direction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::snake::Snake;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Turn { direction: Direction },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Register {
        name: String,
        field_width: usize,
        field_height: usize,
    },
    Turn {
        players: Vec<Snake>,
    },
}

// #[cfg(test)]
// mod tests {
//     use super::ClientMessage;

//     #[test]
//     fn deserialization() {
//         let msg = ClientMessage::Register { name: "Bartek".into() };
//         let output = serde_json::to_string(&msg).unwrap();
//         println!("{}", output);
//     }
//     #[test]
//     fn serialization() {
//         let fake_message = r#"
//         {
//             "Register" : {
//                 "name" : "Bartek"
//             }
//         }
//         "#;

//         let msg : ClientMessage = serde_json::from_str(fake_message).unwrap();
//         println!{"{:?}", msg};
//     }
// }
