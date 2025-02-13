use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use common::messages::Message;
use http_auth_basic::Credentials;

use htpasswd_verify::Htpasswd;
use ldap3::{LdapConnAsync, LdapConnSettings};

use crate::message_stream::MessageStream;

#[derive(Clone)]
pub struct HtpasswdAuthenticationManager {
    data: HashMap<String, String>,
}

impl HtpasswdAuthenticationManager {
    pub fn new(path: &PathBuf) -> Result<Self> {
        Ok(HtpasswdAuthenticationManager {
            data: load_htpasswd(path)?,
        })
    }

    pub fn reset(&mut self, path: &PathBuf) -> Result<()> {
        self.data = load_htpasswd(path)?;
        Ok(())
    }

    pub fn check(&self, username: &str, password: &str) -> bool {
        let Some(value) = self.data.get(username) else {
            return false;
        };
        let encoded = Htpasswd::from(value.as_str());
        return encoded.check(username, password);
    }

    pub fn authenticate(&self, credentials: &[u8]) -> Result<String> {
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

#[derive(Clone)]
pub struct LdapAuthenticationManager {
    url: String,
}

impl LdapAuthenticationManager {
    pub fn new(url: String) -> LdapAuthenticationManager {
        LdapAuthenticationManager { url }
    }

    pub async fn authenticate(&self, credentials: &[u8]) -> Result<String> {
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
}

#[derive(Clone)]
pub struct AuthenticationManager {
    pub htpasswd: Option<HtpasswdAuthenticationManager>,
    pub ldap: Option<LdapAuthenticationManager>,
}

impl AuthenticationManager {
    pub fn new(pwfile: Option<PathBuf>, ldap_url: Option<String>) -> Self {
        let htpasswd: Option<HtpasswdAuthenticationManager> = match &pwfile {
            Some(path) => {
                // TODO: Fix unwrap
                let authentication_manager = HtpasswdAuthenticationManager::new(path).unwrap();
                Some(authentication_manager)
            }
            None => None,
        };
        let ldap = match ldap_url {
            Some(url) => Some(LdapAuthenticationManager::new(url)),
            None => None,
        };
        AuthenticationManager { htpasswd, ldap }
    }

    pub async fn authenticate(&self, stream: &mut impl MessageStream) -> Result<String> {
        let message = stream.read().await?;
        let Message::AuthenticationRequest(request) = message else {
            return Err(Error::new(
                ErrorKind::Other,
                "expected authentication request",
            ));
        };

        match request.method.as_str() {
            "none" => {
                log::debug!("Authenticating with \"none\"");
                return Ok("nobody".into());
            }
            "basic" => {
                log::debug!("Authenticating with \"basic\"");
                return match &self.htpasswd {
                    Some(auth) => auth.authenticate(&request.credentials),
                    None => Err(Error::new(ErrorKind::Other, "no htpasswd auth")),
                };
            }
            "ldap" => {
                log::debug!("Authenticating with \"ldap\"");
                return match &self.ldap {
                    Some(auth) => auth.authenticate(&request.credentials).await,
                    None => Err(Error::new(ErrorKind::Other, "no ldap auth")),
                };
            }
            method => Err(Error::new(
                ErrorKind::Other,
                format!("invalid mode {method}"),
            )),
        }
    }

    pub fn reset(&mut self, pwfile: &Option<PathBuf>) -> Result<()> {
        return match self.htpasswd {
            Some(ref mut auth) => match &pwfile {
                Some(path) => auth.reset(path),
                None => Ok(()),
            },
            None => Ok(()),
        };
    }
}
