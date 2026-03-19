use std::io::Result;

use async_trait::async_trait;

#[async_trait]
pub trait Authenticator {
    fn name(&self) -> &str;
    async fn authenticate(&self, credentials: &[u8]) -> Result<String>;
    async fn reset(&mut self) -> Result<()>;
}
