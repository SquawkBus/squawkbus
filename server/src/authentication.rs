use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use htpasswd_verify::Htpasswd;
use http_auth_basic::Credentials;
use ldap3::{LdapConnAsync, LdapConnSettings};
use tokio::sync::Mutex;

use common::MessageStream;
use common::messages::Message;

use crate::options::AuthenticationOption;

#[async_trait]
pub trait Authenticatable {
    fn name(&self) -> &str;
    async fn authenticate(&self, credentials: &[u8]) -> Result<String>;
    async fn reset(&mut self) -> Result<()>;
}

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

#[derive(Clone)]
pub struct LdapAuthenticationManager {
    url: String,
}

impl LdapAuthenticationManager {
    pub fn new(url: String) -> LdapAuthenticationManager {
        LdapAuthenticationManager { url }
    }
}

#[async_trait]
impl Authenticatable for LdapAuthenticationManager {
    fn name(&self) -> &str {
        "ldap"
    }

    async fn authenticate(&self, credentials: &[u8]) -> Result<String> {
        let credentials = String::from_utf8(credentials.into())
            .map_err(|e| Error::new(ErrorKind::Other, format!("invalid credentials: {}", e)))?;
        let credentials = Credentials::decode(credentials)
            .map_err(|e| Error::new(ErrorKind::Other, format!("invalid credentials: {}", e)))?;

        let (conn, mut ldap) = LdapConnAsync::with_settings(
            LdapConnSettings::new()
                .set_starttls(true)
                .set_no_tls_verify(true),
            &self.url,
        )
        .await?;
        ldap3::drive!(conn);

        // Attempts a simple bind using the passed in values of username and Password
        let result = ldap
            .simple_bind(&credentials.user_id, &credentials.password)
            .await?
            .success();
        ldap.unbind().await?;

        match result.is_err() {
            true => {
                log::info!(
                    "Failed to authenticate as \"{}\"",
                    credentials.user_id.as_str()
                );
                Err(Error::new(
                    ErrorKind::Other,
                    format!("invalid user \"{}\"", credentials.user_id),
                ))
            }
            false => {
                log::info!("Authenticated as \"{}\"", credentials.user_id.as_str());
                Ok(credentials.user_id)
            }
        }
    }

    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct AuthenticationManager {
    pub auth: Arc<Mutex<dyn Authenticatable + Send>>,
}

impl AuthenticationManager {
    pub fn new(option: &AuthenticationOption) -> Result<Self> {
        Ok(match option {
            AuthenticationOption::None => AuthenticationManager {
                auth: Arc::new(Mutex::new(NullAuthenticationManager {})),
            },
            AuthenticationOption::Basic(path) => AuthenticationManager {
                auth: Arc::new(Mutex::new(HtpasswdAuthenticationManager::new(&path)?)),
            },
            AuthenticationOption::Ldap(url) => AuthenticationManager {
                auth: Arc::new(Mutex::new(LdapAuthenticationManager::new(url.clone()))),
            },
        })
    }

    pub async fn authenticate(&self, stream: &mut impl MessageStream) -> Result<String> {
        let message = stream.read().await?;
        let Message::AuthenticationRequest {
            method,
            credentials,
        } = message
        else {
            return Err(Error::new(
                ErrorKind::Other,
                "expected authentication request",
            ));
        };

        let auth = self.auth.clone();
        let auth = auth.lock().await;

        if method.as_str() != auth.name() {
            let msg = std::format!("invalid method {}", method.as_str());
            return Err(Error::new(ErrorKind::Other, msg));
        }

        auth.authenticate(&credentials).await
    }

    pub async fn reset(&mut self) -> Result<()> {
        let auth = self.auth.clone();
        let mut auth = auth.lock().await;
        auth.reset().await
    }
}
