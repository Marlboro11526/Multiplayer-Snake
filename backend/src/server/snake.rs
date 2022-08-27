use serde::{Serialize, Deserialize};

use super::{Colour, Direction, Point};
use std::{collections::VecDeque, ops::Deref};

#[derive(Debug, Serialize, Deserialize)]
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
        let last = self.parts.pop_back().unwrap();
        let new_head = *self.parts.front().unwrap() + self.direction;
        (new_head, last)
    }

    pub fn killed_restart(&mut self, point: Point) {
        self.parts.clear();
        self.parts.push_front(point);
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}
