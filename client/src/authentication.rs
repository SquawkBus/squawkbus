use std::io::{self, Error, ErrorKind};

use common::{
    messages::{AuthenticationRequest, Message},
    MessageStream,
};
use http_auth_basic::Credentials;

pub async fn authenticate(
    stream: &mut impl MessageStream,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) -> io::Result<String> {
    let request = Message::AuthenticationRequest(match mode.as_str() {
        "none" => Ok(AuthenticationRequest {
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

            Ok(AuthenticationRequest {
                method: "none".into(),
                credentials: credentials.encode().into(),
            })
        }
        _ => Err(Error::new(ErrorKind::Other, "invalid method")),
    }?);
    stream.write(&request).await?;

    let response = stream.read().await?;

    match response {
        Message::AuthenticationResponse(msg) => Ok(msg.client_id.clone()),
        _ => Err(Error::new(ErrorKind::Other, "invalid message")),
    }
}
