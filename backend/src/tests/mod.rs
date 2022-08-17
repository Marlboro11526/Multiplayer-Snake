#[cfg(test)]
mod tests {
    use crate::server::messages::ClientMessage;
    use crate::server::Args;

    #[test]
    fn deserialization() {
        let msg = ClientMessage::Register { name: "Bartek".into() };
        let output = serde_json::to_string(&msg).unwrap();
        // let a : A
        println!("{}", output);
    }
    #[test]
    fn serialization() {
        let fake_message = r#"
        {
            "Register" : {
                "name" : "Bartek"
            }
        }
        "#;

        let msg : ClientMessage = serde_json::from_str(fake_message).unwrap();
        println!{"{:?}", msg};
    }
}