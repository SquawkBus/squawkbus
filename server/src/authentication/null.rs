use std::io::Result;

use async_trait::async_trait;

use crate::authentication::traits::Authenticator;

#[derive(Clone)]
pub struct NullAuthenticator {}

#[async_trait]
impl Authenticator for NullAuthenticator {
    fn name(&self) -> &str {
        "none"
    }

    async fn authenticate(&self, _credentials: &[u8]) -> Result<String> {
        Ok(String::from("nobody"))
    }

    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}
