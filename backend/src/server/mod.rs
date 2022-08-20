pub mod messages;
pub mod snake;

use std::{fmt, net::SocketAddr, sync::{Arc}, collections::VecDeque, time::Duration};
use dashmap::DashMap;
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{net::{TcpListener, TcpStream}, time::sleep, io::AsyncWriteExt};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use log::{debug, error, info};
use uuid::Uuid;

use self::snake::Snake;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Port to use
    #[clap(short='p', value_parser, default_value_t = 43210)]
    port: u16,

    /// Maximum players count
    #[clap(short='c', value_parser, default_value_t = 25)]
    max_players_count: usize,

    /// Field width in blocks
    #[clap(short='w', value_parser, default_value_t = 15)]
    field_width: u8,
    
    /// Field height in blocks
    #[clap(short='h', value_parser, default_value_t = 10)]
    field_height: u8,

    /// Game tick in miliseconds
    #[clap(short='t', value_parser, default_value_t = 500)]
    game_tick: u64,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

type Point = (u8, u8);
type Colour = (u8, u8, u8);


pub struct Server {
    args: Args,
    state: Arc<State>,
}

struct PlayerData {
    snake: Snake,
    last_move: Option<Direction>,
    tx: Sender<()>,
    rx: Receiver<()>,
}

impl PlayerData {
    fn new(starting_point: Point, colour: Colour, direction: Direction) -> Self {
        let (tx, rx) = channel::<()>(1);
        PlayerData { 
            snake: Snake::new(
                VecDeque::from([starting_point]), 
                colour, 
                direction),
            last_move: None, 
            tx, 
            rx}
    }
}

struct State {
    players: DashMap<Uuid, PlayerData>,
}

impl State {
    fn new() -> Self {
        State {
            players: DashMap::new(),
            
        }
    }
}

#[derive(Debug)]
pub struct ServerError;

#[derive(Debug)]
pub struct ConnectionError;

#[derive(Debug)]
pub struct GameError;

impl fmt::Display for ServerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Server error")
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Connection error")
    }

}
impl fmt::Display for GameError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Game error")
    }
}

impl Context for ServerError {}
impl Context for ConnectionError {}
impl Context for GameError {}

impl Server {
    pub fn new(args: Args) -> Self {
        Server { 
            args,
            state: Arc::new(State::new()),
        }
    }

    pub async fn run(self: &Arc<Self>) -> Result<(), ServerError> {
        let addr = format!("127.0.0.1:{}", self.args.port).to_string();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| {
                Report::new(ServerError)
                    .attach_printable(format!("Unable to start server! {}", e))
            })?;

        info!("Server listening on {:?}", listener.local_addr());

        while let Ok((stream, addr)) = listener.accept().await {
            debug!("New connection from {}", addr);

            let me = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(e) = me.handle_connection(stream, addr)
                    .await
                    .attach_printable_lazy(|| format!("Error in connection {}", addr)) {
                        info!("{}", e);
                    }
                
            });
        }

        Ok(())
    }

    async fn handle_connection(self :Arc<Self>, mut stream: TcpStream, addr: SocketAddr) -> Result<(), ConnectionError> {
        
        if self.state.players.len() == self.args.max_players_count {
            debug!("Connection limit reached. Disconnecting {}", addr);
            _ = stream.shutdown().await;
            return Ok(())
        }

        loop {
            
        }
        sleep(Duration::from_secs(100)).await;
        Ok(())
    }

    async fn game_loop(self: Arc<Self>) -> Result<(), GameError> {

        while !self.state.players.is_empty() {
            sleep(Duration::from_millis(self.args.game_tick)).await;
        }

        Ok(())
    }

}
