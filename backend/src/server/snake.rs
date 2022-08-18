use std::collections::VecDeque;
use super::{Direction, Point, Colour};

pub struct Snake {
    parts: VecDeque<Point>,
    colour: Colour,
    direction: Direction
}

impl Snake {
    pub fn new(parts: VecDeque<Point>, colour: Colour, direction: Direction) -> Self {
        Snake {parts, colour, direction}
    }
}