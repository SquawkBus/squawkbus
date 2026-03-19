use async_trait::async_trait;
use tokio::io::{self};

use crate::messages::Message;

#[async_trait]
pub trait MessageStream {
    async fn read(&mut self) -> io::Result<Message>;
    async fn write(&mut self, message: &Message) -> io::Result<()>;
}
