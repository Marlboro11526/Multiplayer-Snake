use log::debug;
use serde::{Deserialize, Serialize};

use super::{Colour, Direction, Point};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snake {
    parts: VecDeque<Point>,
    colour: Colour,
    direction: Direction,
}

impl Snake {
    pub fn new(parts: VecDeque<Point>, colour: Colour, direction: Direction) -> Self {
        Snake {
            parts,
            colour,
            direction,
        }
    }

    pub fn do_move(&mut self) -> (Point, Point) {
        let new_head = *self.parts.front().unwrap() + self.direction;
        let last = self.parts.pop_back().unwrap();
        self.parts.push_front(new_head);
        (new_head, last)
    }

    pub fn killed_restart(&mut self, point: Point, direction: Direction) {
        self.parts.clear();
        self.parts.push_front(point);
        self.direction = direction;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        debug!("Really setting direction to {:#?}", direction);
        self.direction = direction;
    }
}
