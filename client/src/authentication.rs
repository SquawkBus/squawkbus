use std::io::{self, Error, ErrorKind};

use common::{messages::Message, MessageStream};
use http_auth_basic::Credentials;

pub async fn authenticate(
    stream: &mut impl MessageStream,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) -> io::Result<String> {
    let request = match mode.as_str() {
        "none" => Ok(Message::AuthenticationRequest {
            method: "none".into(),
            credentials: Vec::new(),
        }),
        "basic" | "ldap" => {
            let Some(username) = username else {
                return Err(Error::new(ErrorKind::Other, "missing username"));
            };
            let Some(password) = password else {
                return Err(Error::new(ErrorKind::Other, "missing password"));
            };

            let credentials = Credentials::new(username, password);

            Ok(Message::AuthenticationRequest {
                method: "none".into(),
                credentials: credentials.encode().into(),
            })
        }
        _ => Err(Error::new(ErrorKind::Other, "invalid method")),
    }?;
    stream.write(&request).await?;

    let response = stream.read().await?;

    match response {
        Message::AuthenticationResponse { client_id } => Ok(client_id.clone()),
        _ => Err(Error::new(ErrorKind::Other, "invalid message")),
    }
}
