use common::messages::Message;
use tokio::io::{self};

pub trait MessageStream {
    async fn read(&mut self) -> io::Result<Message>;
    async fn write(&mut self, message: &Message) -> io::Result<()>;
}
