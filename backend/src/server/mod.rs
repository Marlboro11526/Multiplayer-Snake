pub mod messages;
pub mod snake;

use std::{fmt, net::SocketAddr, sync::{Arc}, collections::VecDeque, time::Duration, rc::Rc};
use futures_util::{SinkExt, StreamExt};
use dashmap::DashMap;
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use clap::Parser;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{net::{TcpListener, TcpStream}, time::sleep, io::AsyncWriteExt};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use log::{debug, error, info};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;
use rand::Rng;
use rand::distributions::{Distribution, Standard};


use self::{snake::Snake, messages::{ServerMessage, ClientMessage}};

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
    field_width: usize,
    
    /// Field height in blocks
    #[clap(short='h', value_parser, default_value_t = 10)]
    field_height: usize,

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

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Point {
    x: u8,
    y: u8,
}

#[derive(Debug)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
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
        Colour {
            r, g, b
        }
    }
}


pub struct Server {
    args: Args,
    state: Arc<State>,
}

struct PlayerData {
    snake: Snake,
    last_move: Option<Direction>,
    tx: Sender<()>
}

impl PlayerData {
    fn new(starting_point: Point, colour: Colour, direction: Direction, tx: Sender<()>) -> Self {
        PlayerData { 
            snake: Snake::new(
                VecDeque::from([starting_point]), 
                colour, 
                direction),
            last_move: None, 
            tx}
    }
}

struct State {
    players: DashMap<Uuid, PlayerData>,
    map_state: DashMap<Point, Option<&'static Uuid> >
}

impl State {
    fn new(args: &Args) -> Self {
        State {
            players: DashMap::with_capacity(args.max_players_count),
            map_state: DashMap::with_capacity(args.field_height * args.field_width)
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
            state: Arc::new(State::new(&args)),
            args,
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

        let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut sink, mut stream) = stream.split();
        let (uuid, mut rx) = self.spawn_payer();

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    // send turn message
                    todo!();
                }
                ws_msg = stream.next() => {
                    match ws_msg {
                        Some(msg) => match msg {
                            Ok(Message::Text(json_str)) => {
                                let message: ClientMessage = serde_json::from_str(&json_str)
                                    .report()
                                    .change_context(ConnectionError)
                                    .attach_printable_lazy(|| format!("Invalid message body, got {}", json_str))?;
                            }
                            _ => {
                                // invalid message, continue
                            }
                        }
                        None => {
                            // close connection
                        }
                    }
                }
                // todo!()
            }
            // sleep(Duration::from_secs(3)).await;
            // info!("{:?}", stream.next().await);
            // if let Err(e) = sink.send(Message::Text(serde_json::to_string(&ServerMessage::Register { name: "Bartek".into() }).unwrap())).await {
            //     error!("{}", e);
            // }
            // sleep(Duration::from_secs(100)).await;
        }
        Ok(())
    }

    fn spawn_payer(self: Arc<Self>) -> (Uuid, Receiver<()>) {
        let mut rng = rand::thread_rng();

        let uuid = Uuid::new_v4();
        let colour: Colour = rng.gen();
        let direction: Direction = rng.gen();
        let (tx, rx) = channel::<()>(1);
        let mut starting_point: Point = rng.gen();
        while self.state.map_state.contains_key(&starting_point) {
            starting_point = rng.gen();
        }
        let new_player = PlayerData::new(
            starting_point, 
            colour,
            direction,
            tx,
        );
        
        self.state.players.insert(uuid.clone(), new_player);
        
        (uuid, rx)
    }

    async fn game_loop(self: Arc<Self>) -> Result<(), GameError> {

        while !self.state.players.is_empty() {
            sleep(Duration::from_millis(self.args.game_tick)).await;
        }

        Ok(())
    }

}
