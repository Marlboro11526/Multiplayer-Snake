pub mod messages;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Port to use
    #[clap(short, value_parser, default_value_t = 43210)]
    name: u16,

    /// Maximum players count
    #[clap(short, value_parser, default_value_t = 25)]
    max_players_count: u8,

    /// Field width in blocks
    #[clap(short, value_parser, default_value_t = 15)]
    field_width: u8,
    
    /// Field height in blocks
    #[clap(short, value_parser, default_value_t = 10)]
    field_height: u8,

    /// Game tick in miliseconds
    #[clap(short, value_parser, default_value_t = 50)]
    game_tick: u8,

    /// Move speed in pixels
    #[clap(short, value_parser, default_value_t = 30)]
    game_speed: u8,

    
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}
