use error_stack::Context;
use std::fmt;

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
