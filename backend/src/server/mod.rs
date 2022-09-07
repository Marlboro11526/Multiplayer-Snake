pub mod errors;
pub mod messages;
pub mod snake;
pub mod types;

use clap::Parser;
use dashmap::{DashMap, DashSet};
use error_stack::{IntoReport, Report, Result, ResultExt};
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use self::types::{Colour, Direction, FieldHeightT, FieldWidthT, PlayerData, Point, State};
use self::{
    errors::*,
    messages::{ClientMessage, ServerMessage},
    snake::Snake,
};

#[derive(Parser, Debug)]
#[clap(name = "Multiplayer Snake Game")]
#[clap(author = "Bartek Sadlej <sadlejbartek@gmail.com>")]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Port to use
    #[clap(short = 'p', value_parser, default_value_t = 43210)]
    port: u16,

    /// Maximum players count
    #[clap(short = 'c', value_parser, default_value_t = 25)]
    max_players_count: usize,

    /// Field width in blocks
    #[clap(short = 'w', value_parser, default_value_t = 15)]
    field_width: FieldWidthT,

    /// Field height in blocks
    #[clap(short = 'h', value_parser, default_value_t = 10)]
    field_height: FieldHeightT,

    /// Game tick in miliseconds
    #[clap(short = 't', value_parser, default_value_t = 500)]
    game_tick: u64,

    /// Food count on map
    #[clap(short = 'f', value_parser, default_value_t = 5)]
    food_count: usize,
}

pub struct Server {
    args: Args,
    state: Arc<State>,
}

impl State {
    fn new(args: &Args) -> Self {
        State {
            players: DashMap::with_capacity(args.max_players_count),
            map_state: DashMap::with_capacity(
                args.field_height as usize * args.field_width as usize,
            ),
            is_running: AtomicBool::new(false),
            food: DashSet::new(),
        }
    }
}

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
            Report::new(ServerError).attach_printable(format!("Unable to start server! {:?}", e))
        })?;

        info!("Server listening on {:?}", listener.local_addr());

        self.refill_food();

        while let Ok((stream, addr)) = listener.accept().await {
            debug!("New connection from {}", addr);

            let me = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(e) = me
                    .handle_connection(stream, addr)
                    .await
                    .attach_printable_lazy(|| format!("Connection lost from {}", addr))
                {
                    debug!("{e:?}");
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

        _ = self.player_loop(sink, stream, uuid, rx).await;
        self.clear_player_parts(&uuid);
        _ = self.state.players.remove(&uuid);

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
                        self.send_turn_message(&mut sink).await?
                    }
                    ws_msg = stream.next() => match ws_msg {
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
                                    ClientMessage::Register { name } => {
                                        debug!("New player name: {}", name);

                                        if self.state.is_running.compare_exchange(
                                            false,
                                            true,
                                            std::sync::atomic::Ordering::SeqCst,
                                            std::sync::atomic::Ordering::SeqCst,
                                        ) == Ok(false)
                                        {
                                            debug!("RUNNING PLAYERS!");
                                            let me = self.clone();
                                            tokio::spawn(async move { me.game_loop().await });
                                        }
                                    }
                                }
                            }
                            _ => {
                                return Err(ConnectionError)
                                    .report()
                                    .attach("Invalid message")
                            }
                        }
                        None => {
                            debug!("Connection ended");
                            return Ok(())
                        }
                    }
            }
        }
    }

    async fn send_turn_message(
        self: &Arc<Self>,
        sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> Result<(), ConnectionError> {
        let players: Vec<Snake> = self
            .state
            .players
            .iter()
            .map(|entry| entry.value().snake.clone())
            .collect();
        let food = self.state.food.iter().map(|entry| *entry.key()).collect();
        let msg = ServerMessage::Turn { players, food };
        Server::send_message(sink, &msg)
            .await
            .change_context(ConnectionError)
            .attach_printable("Could not send message to clinet")?;

        Ok(())
    }

    fn spawn_payer(self: &Arc<Self>) -> (Uuid, Receiver<()>) {
        let mut rng = ChaCha20Rng::from_entropy();

        let uuid = Uuid::new_v4();
        let colour: Colour = rng.gen();
        let direction: Direction = rng.gen();
        let (tx, rx) = channel::<()>(16);
        let starting_point: Point = self.random_free_point();
        let new_player = PlayerData::new(starting_point, colour, direction, tx);

        self.state.players.insert(uuid, new_player);

        (uuid, rx)
    }

    async fn game_loop(self: &Arc<Self>) -> Result<(), GameError> {
        while !self.state.players.is_empty() {
            sleep(Duration::from_millis(self.args.game_tick)).await;

            let mut killed_players = HashSet::<Uuid>::new();
            let mut new_heads = HashMap::<Point, Vec<Uuid>>::new();

            for mut player in self.state.players.iter_mut() {
                if let Some(direction) = player.last_move {
                    player.snake.set_direction(direction)
                }
                let (new_head, last) = player.value_mut().snake.do_move();
                new_heads.entry(new_head).or_default().push(*player.key());
                if self.state.food.remove(&new_head).is_some() {
                    player.value_mut().score += 1;
                } else {
                    self.state.map_state.remove(&last);
                    player.value_mut().snake.pop_last();
                }
            }

            for new_head in new_heads.iter() {
                if self.state.map_state.contains_key(new_head.0)
                    || !self.is_in_map(new_head.0)
                    || new_head.1.len() > 1
                {
                    killed_players.extend(new_head.1.clone());
                } else {
                    for player_uuid in new_head.1 {
                        self.state.map_state.insert(*new_head.0, *player_uuid);
                    }
                }
            }

            for killed_player in killed_players {
                let starting_point = self.random_free_point();
                let direction = self.random_direction();
                self.clear_player_parts(&killed_player);
                if let Some(mut player_data) = self.state.players.get_mut(&killed_player) {
                    player_data.killed_restart(starting_point, direction)
                }
                self.state.map_state.insert(starting_point, killed_player);
            }

            self.refill_food();

            for player in self.state.players.iter_mut() {
                _ = player.tx.send(()).await;
            }
        }
        debug!("NO PLAYERS STOP");
        self.state
            .is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }

    fn random_free_point(self: &Arc<Self>) -> Point {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut point: Point = rng.gen();
        while !self.is_in_map(&point)
            || self.state.map_state.contains_key(&point)
            || self.state.food.contains(&point)
        {
            point = rng.gen();
        }
        point
    }

    fn random_direction(self: &Arc<Self>) -> Direction {
        let mut rng = ChaCha20Rng::from_entropy();
        rng.gen()
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

    fn refill_food(self: &Arc<Self>) {
        let curr_food_count = self.state.food.len();
        for _ in curr_food_count..self.args.food_count {
            let food = self.random_free_point();
            self.state.food.insert(food);
        }
    }

    fn clear_player_parts(self: &Arc<Self>, uuid: &Uuid) {
        if let Some(entry) = self.state.players.get_mut(uuid) {
            for p in &entry.snake.parts {
                self.state.map_state.remove(p);
            }
        }
    }
}
