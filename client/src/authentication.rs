use std::io::{self, Error, ErrorKind};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, WriteHalf};

pub async fn authenticate<S>(
    skt_write_half: &mut WriteHalf<S>,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite,
{
    // Mode
    skt_write_half.write(mode.as_bytes()).await?;
    skt_write_half.write("\n".as_bytes()).await?;

    if mode == "none" {
        log::info!("Authenticate with {}", mode.as_str());
    } else if mode == "htpasswd" {
        log::info!("Authenticate with {}", mode.as_str());
        let Some(username) = username else {
            return Err(Error::new(ErrorKind::Other, "missing username"));
        };
        let Some(password) = password else {
            return Err(Error::new(ErrorKind::Other, "missing password"));
        };
        // User
        skt_write_half.write(username.as_bytes()).await?;
        skt_write_half.write("\n".as_bytes()).await?;

        // Password
        skt_write_half.write(password.as_bytes()).await?;
        skt_write_half.write("\n".as_bytes()).await?;
    } else {
        log::error!("Invalid mode {}", mode.as_str());
        return Err(Error::new(ErrorKind::Other, "invalid mode"));
    }

    skt_write_half.flush().await?;

    Ok(())
}
