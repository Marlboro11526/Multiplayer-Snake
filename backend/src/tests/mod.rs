#[cfg(test)]
mod tests {
    use crate::server::{messages::ClientMessage, types::Direction};

    #[test]
    fn deserialization() {
        let msg = ClientMessage::Turn {
            direction: Direction::Up,
        };
        let output = serde_json::to_string(&msg).unwrap();
        println!("{}", output);
    }
    #[test]
    fn serialization() {
        let fake_message = r#"
        {
            "Turn" : {
                "direction" : "Down"
            }
        }
        "#;

        let msg: ClientMessage = serde_json::from_str(fake_message).unwrap();
        println! {"{:?}", msg};
    }
}
