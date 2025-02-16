use std::future::Future;

use tokio::io::{self};

use crate::messages::Message;

pub trait MessageStream {
    fn read(&mut self) -> impl Future<Output = io::Result<Message>> + Send;
    fn write(&mut self, message: &Message) -> impl Future<Output = io::Result<()>> + Send;
}
