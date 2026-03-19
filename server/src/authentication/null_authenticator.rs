use std::io::Result;

use async_trait::async_trait;

use crate::authentication::authenticatable::Authenticatable;

#[derive(Clone)]
pub struct NullAuthenticationManager {}

#[async_trait]
impl Authenticatable for NullAuthenticationManager {
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
