use std::{collections::VecDeque, ops::Add, sync::atomic::AtomicBool};

use dashmap::{DashMap, DashSet};
use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use super::snake::Snake;

pub type Score = usize;
pub type Name = String;
pub type FieldWidthT = isize;
pub type FieldHeightT = isize;

pub type PlayerInfo = (Snake, Uuid, Name, Score);

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, direction: Direction) -> Self::Output {
        let mut x = self.x;
        let mut y = self.y;
        match direction {
            Direction::Up => y -= 1,
            Direction::Right => x += 1,
            Direction::Down => y += 1,
            Direction::Left => x -= 1,
        }
        Point { x, y }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: FieldWidthT,
    pub y: FieldWidthT,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        let rand_direction = rng.gen_range(0..3);
        match rand_direction {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            _ => Direction::Left,
        }
    }
}

impl Distribution<Point> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Point {
        let (rand_x, rand_y) = rng.gen();
        Point {
            x: rand_x,
            y: rand_y,
        }
    }
}

impl Distribution<Colour> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Colour {
        let (r, g, b) = rng.gen();
        Colour { r, g, b }
    }
}

#[derive(Debug)]
pub struct PlayerData {
    pub name: String,
    pub snake: Snake,
    pub last_move: Option<Direction>,
    pub tx: Sender<()>,
    pub score: Score,
}

impl PlayerData {
    pub fn new(
        name: String,
        starting_point: Point,
        colour: Colour,
        direction: Direction,
        tx: Sender<()>,
    ) -> Self {
        PlayerData {
            name,
            snake: Snake::new(VecDeque::from([starting_point]), colour, direction),
            last_move: None,
            tx,
            score: 1,
        }
    }

    pub fn killed_restart(&mut self, starting_point: Point, direction: Direction) {
        self.snake.killed_restart(starting_point, direction);
        self.last_move = None;
    }
}

pub struct State {
    pub players: DashMap<Uuid, PlayerData>,
    pub map_state: DashMap<Point, Uuid>,
    pub is_running: AtomicBool,
    pub food: DashSet<Point>,
}
