pub mod messages;
pub mod snake;

use clap::Parser;
use dashmap::DashMap;
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::sync::atomic::AtomicBool;
use std::{collections::VecDeque, fmt, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use self::{
    messages::{ClientMessage, ServerMessage},
    snake::Snake,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Port to use
    #[clap(short = 'p', value_parser, default_value_t = 43210)]
    port: u16,

    /// Maximum players count
    #[clap(short = 'c', value_parser, default_value_t = 25)]
    max_players_count: usize,

    /// Field width in blocks
    #[clap(short = 'w', value_parser, default_value_t = 15)]
    field_width: i8,

    /// Field height in blocks
    #[clap(short = 'h', value_parser, default_value_t = 10)]
    field_height: i8,

    /// Game tick in miliseconds
    #[clap(short = 't', value_parser, default_value_t = 500)]
    game_tick: u64,
}

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
    x: i8,
    y: i8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        Colour { r, g, b }
    }
}

pub struct Server {
    args: Args,
    state: Arc<State>,
}

struct PlayerData {
    snake: Snake,
    last_move: Option<Direction>,
    tx: Sender<()>,
}

impl PlayerData {
    fn new(starting_point: Point, colour: Colour, direction: Direction, tx: Sender<()>) -> Self {
        PlayerData {
            snake: Snake::new(VecDeque::from([starting_point]), colour, direction),
            last_move: None,
            tx,
        }
    }

    fn killed_restart(&mut self, starting_point: Point) {
        self.snake.killed_restart(starting_point);
        self.last_move = None;
    }
}

struct State {
    players: DashMap<Uuid, PlayerData>,
    map_state: DashMap<Point, Uuid>,
    is_running: AtomicBool,
}

impl State {
    fn new(args: &Args) -> Self {
        State {
            players: DashMap::with_capacity(args.max_players_count),
            map_state: DashMap::with_capacity(
                args.field_height as usize * args.field_width as usize,
            ),
            is_running: AtomicBool::new(false),
        }
    }
}

#[derive(Debug)]
pub struct ServerError;

#[derive(Debug)]
pub struct ConnectionError;

#[derive(Debug)]
pub struct GameError;

#[derive(Debug)]
pub struct SendError;

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
impl fmt::Display for SendError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Send message error")
    }
}

impl Context for ServerError {}
impl Context for ConnectionError {}
impl Context for GameError {}
impl Context for SendError {}

impl Server {
    pub fn new(args: Args) -> Self {
        Server {
            state: Arc::new(State::new(&args)),
            args,
        }
    }

    pub async fn run(self: &Arc<Self>) -> Result<(), ServerError> {
        let addr = format!("127.0.0.1:{}", self.args.port).to_string();
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            Report::new(ServerError).attach_printable(format!("Unable to start server! {}", e))
        })?;

        info!("Server listening on {:?}", listener.local_addr());

        while let Ok((stream, addr)) = listener.accept().await {
            debug!("New connection from {}", addr);

            if self.state.is_running.compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            ) == Ok(false)
            {
                let me = self.clone();
                tokio::spawn(async move { me.game_loop().await });
            }

            let me = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(e) = me
                    .handle_connection(stream, addr)
                    .await
                    .attach_printable_lazy(|| format!("Error in connection {}", addr))
                {
                    info!("{}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        self: &Arc<Self>,
        mut stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), ConnectionError> {
        if self.state.players.len() == self.args.max_players_count {
            debug!("Connection limit reached. Disconnecting {}", addr);
            _ = stream.shutdown().await;
            return Ok(());
        }

        let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut sink, stream) = stream.split();
        let (uuid, rx) = self.spawn_payer();
        Server::send_message(
            &mut sink,
            &ServerMessage::Register {
                field_width: self.args.field_width,
                field_height: self.args.field_height,
            },
        )
        .await
        .change_context(ConnectionError)
        .attach_printable("Unable to send Register message")?;
        self.player_loop(sink, stream, uuid, rx).await?;

        self.state.players.remove(&uuid);
        Ok(())
    }

    async fn send_message(
        sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
        message: &ServerMessage,
    ) -> Result<(), SendError> {
        let encoded_message = serde_json::to_string(message)
            .report()
            .change_context(SendError)
            .attach_printable("Serde error while encoding!")?;

        sink.send(Message::Text(encoded_message))
            .await
            .report()
            .change_context(SendError)
            .attach_printable("Error while sending message!")?;

        Ok(())
    }

    async fn player_loop(
        self: &Arc<Self>,
        mut sink: SplitSink<WebSocketStream<TcpStream>, Message>,
        mut stream: SplitStream<WebSocketStream<TcpStream>>,
        uuid: Uuid,
        mut rx: Receiver<()>,
    ) -> Result<(), ConnectionError> {
        loop {
            tokio::select! {
                    _ = rx.recv() => {
                        let players : Vec<Snake> = self.state.players.iter().map(|entry| entry.value().snake.clone()).collect();
                        let msg = ServerMessage::Turn{players};
                        Server::send_message(&mut sink, &msg).await
                            .change_context(ConnectionError)
                            .attach_printable("Could not send message to clinet")?;
                    }
                    ws_msg = stream.next() => {
                        match ws_msg {
                            Some(msg) => match msg {
                                Ok(Message::Text(json_str)) => {
                                    let message: ClientMessage = serde_json::from_str(&json_str)
                                        .report()
                                        .change_context(ConnectionError)
                                        .attach_printable_lazy(|| format!("Invalid message body, got {}", json_str))?;

                                    match message {
                                        ClientMessage::Turn { direction } => {
                                            if let Some(mut player_state) = self.state.players.get_mut(&uuid) {
                                                player_state.last_move = Some(direction);
                                            } else {
                                                return Err(ConnectionError)
                                                    .report()
                                                    .attach("Game logic broken! Player not in players.")
                                            }
                                        },
                                    }
                                }
                                _ => {
                                    return Err(ConnectionError)
                                        .report()
                                        .attach("Invalid message")
                                }
                            }
                            None => {
                                return Ok(())
                            }
                        }
                    }
            }
        }
    }

    fn spawn_payer(self: &Arc<Self>) -> (Uuid, Receiver<()>) {
        let mut rng = rand::thread_rng();

        let uuid = Uuid::new_v4();
        let colour: Colour = rng.gen();
        let direction: Direction = rng.gen();
        let (tx, rx) = channel::<()>(16);
        let starting_point: Point = self.random_free_point();
        let new_player = PlayerData::new(starting_point, colour, direction, tx);

        self.state.players.insert(uuid.clone(), new_player);

        (uuid, rx)
    }

    async fn game_loop(self: &Arc<Self>) -> Result<(), GameError> {
        while !self.state.players.is_empty() {
            sleep(Duration::from_millis(self.args.game_tick)).await;

            let mut killed_players = HashSet::<Uuid>::new();
            let mut new_heads = HashMap::<Uuid, Point>::new();

            for mut player in self.state.players.iter_mut() {
                player
                    .last_move
                    .map(|direction| player.snake.set_direction(direction));
                let (new_head, last) = player.value_mut().snake.do_move();
                new_heads.insert(player.key().clone(), new_head);
                self.state.map_state.remove(&last);
            }

            for new_head in new_heads.iter() {
                if self.state.map_state.contains_key(&new_head.1) || !self.is_in_map(&new_head.1) {
                    killed_players.insert(*new_head.0);
                }
            }

            for new_head in new_heads.iter() {
                if !killed_players.contains(&new_head.0) {
                    self.state.map_state.insert(*new_head.1, *new_head.0);
                }
            }

            for killed_player in killed_players {
                let starting_point = self.random_free_point();
                self.state
                    .players
                    .get_mut(&killed_player)
                    .map(|mut player_data| player_data.killed_restart(starting_point));
                self.state.map_state.insert(starting_point, killed_player);
            }

            for player in self.state.players.iter_mut() {
                _ = player.tx.send(()).await;
            }
        }

        self.state
            .is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }

    fn random_free_point(self: &Arc<Self>) -> Point {
        let mut rng = rand::thread_rng();
        let mut point: Point = rng.gen();
        while self.state.map_state.contains_key(&point) || !self.is_in_map(&point) {
            point = rng.gen();
        }
        point
    }

    fn is_in_map(self: &Arc<Self>, point: &Point) -> bool {
        let field_width = self.args.field_width;
        let field_height = self.args.field_height;
        match point {
            Point { x, y } if *x < 0 || *y < 0 => false,
            Point { x, y: _ } if *x >= field_width => false,
            Point { x: _, y } if *y >= field_height => false,
            _ => true,
        }
    }
}
