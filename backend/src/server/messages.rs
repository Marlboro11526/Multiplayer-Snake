use crate::server::Direction;
use serde::{Deserialize, Serialize};

use super::{
    types::{FieldHeightT, FieldWidthT, PlayerInfo},
    Point,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Register { name: String },
    Turn { direction: Direction },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Register {
        field_width: FieldWidthT,
        field_height: FieldHeightT,
    },
    Turn {
        players: Vec<PlayerInfo>,
        food: Vec<Point>,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use uuid::Uuid;

    use crate::server::{snake::Snake, Colour, Direction, Point};

    use super::*;
    #[test]
    fn serialization() {
        let msg = ServerMessage::Turn {
            players: vec![(
                Snake::new(
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
                ),
                Uuid::new_v4(),
                "Bartek".into(),
                123,
            )],
            food: vec![
                Point { x: 1, y: 2 },
                Point { x: 1, y: 3 },
                Point { x: 1, y: 5 },
            ],
        };

        let serialized = serde_json::to_string(&msg);
        println!("{:?}", serialized);

        let msg = ClientMessage::Turn {
            direction: Direction::Up,
        };

        let serialized = serde_json::to_string(&msg);
        println!("{:?}", serialized);
    }
}
