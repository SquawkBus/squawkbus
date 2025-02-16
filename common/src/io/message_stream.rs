use tokio::io::{self};

use crate::messages::Message;

pub trait MessageStream {
    async fn read(&mut self) -> io::Result<Message>;
    async fn write(&mut self, message: &Message) -> io::Result<()>;
}
