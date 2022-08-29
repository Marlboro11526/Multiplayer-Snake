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
        field_width: usize,
        field_height: usize,
    },
    Turn {
        players: Vec<Snake>,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::server::{snake::Snake, Colour, Point};

    use super::ServerMessage;
    #[test]
    fn serialization() {
        let msg = ServerMessage::Turn {
            players: vec![Snake::new(
                VecDeque::from(vec![
                    Point { x: 1, y: 2 },
                    Point { x: 1, y: 3 },
                    Point { x: 1, y: 5 },
                ]),
                Colour {
                    r: 123,
                    g: 0,
                    b: 255,
                },
                crate::server::Direction::Down,
            )],
        };

        let serialized = serde_json::to_string(&msg);
        println!("{:?}", serialized);
    }
}
