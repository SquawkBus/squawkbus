use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use async_trait::async_trait;
use htpasswd_verify::Htpasswd;
use http_auth_basic::Credentials;

use crate::authentication::authenticatable::Authenticatable;

#[derive(Clone)]
pub struct HtpasswdAuthenticationManager {
    path: PathBuf,
    data: HashMap<String, String>,
}

impl HtpasswdAuthenticationManager {
    pub fn new(path: &PathBuf) -> Result<Self> {
        Ok(HtpasswdAuthenticationManager {
            path: path.clone(),
            data: load_htpasswd(path)?,
        })
    }

    pub fn check(&self, username: &str, password: &str) -> bool {
        let Some(value) = self.data.get(username) else {
            return false;
        };
        let encoded = Htpasswd::from(value.as_str());
        return encoded.check(username, password);
    }
}

fn load_htpasswd(path: &PathBuf) -> Result<HashMap<String, String>> {
    let contents = read_to_string(path)?;

    let mut data = HashMap::new();

    for line in contents.lines() {
        let (username, _hash) = line
            .split_once(':')
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid_entry"))?;
        data.insert(username.to_string(), line.to_owned());
    }

    Ok(data)
}

#[async_trait]
impl Authenticatable for HtpasswdAuthenticationManager {
    fn name(&self) -> &str {
        "basic"
    }

    async fn authenticate(&self, credentials: &[u8]) -> Result<String> {
        let credentials = String::from_utf8(credentials.into())
            .map_err(|e| Error::new(ErrorKind::Other, format!("invalid credentials: {}", e)))?;
        let credentials = Credentials::decode(credentials)
            .map_err(|e| Error::new(ErrorKind::Other, format!("invalid credentials: {}", e)))?;

        let is_valid = self.check(credentials.user_id.as_str(), credentials.password.as_str());
        match is_valid {
            true => {
                log::info!("Authenticated as \"{}\"", credentials.user_id.as_str());
                Ok(credentials.user_id)
            }
            false => {
                log::info!(
                    "Failed to authenticate as \"{}\"",
                    credentials.user_id.as_str()
                );
                Err(Error::new(
                    ErrorKind::Other,
                    format!("invalid user \"{}\"", credentials.user_id),
                ))
            }
        }
    }

    async fn reset(&mut self) -> Result<()> {
        self.data = load_htpasswd(&self.path)?;
        Ok(())
    }
}
