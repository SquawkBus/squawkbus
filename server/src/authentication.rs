use std::io;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub async fn authenticate<R: AsyncRead + Unpin>(reader: &mut BufReader<R>) -> io::Result<String> {
    // Read the mode.
    let mut mode = String::new();
    reader.read_line(&mut mode).await?;
    mode.truncate(mode.len() - 1); // Must have at least a single '\n';

    if mode == "none" {
        return Ok(String::from("nobody"));
    }

    if mode == "htpasswd" {
        // Read the username
        let mut user = String::new();
        reader.read_line(&mut user).await?;
        user.truncate(user.len() - 1); // Must have at least a single '\n';

        // Read the password.
        let mut password = String::new();
        reader.read_line(&mut password).await?;
        password.truncate(password.len() - 1); // Must have at least a single '\n';

        // TODO: Check the password
        return Ok(user);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("invalid mode {mode}"),
    ))
}
