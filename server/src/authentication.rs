use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

use htpasswd_verify::Htpasswd;

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

    pub async fn authenticate<R>(&self, reader: &mut BufReader<R>) -> Result<String>
    where
        R: AsyncRead + Unpin,
    {
        // Read the username
        let mut username = String::new();
        reader.read_line(&mut username).await?;
        username.truncate(username.len() - 1); // Must have at least a single '\n';

        // Read the password.
        let mut password = String::new();
        reader.read_line(&mut password).await?;
        password.truncate(password.len() - 1); // Must have at least a single '\n';

        let is_valid = self.check(username.as_str(), password.as_str());
        match is_valid {
            true => {
                log::info!("Authenticated as \"{}\"", username.as_str());
                Ok(username)
            }
            false => {
                log::info!("Failed to authenticate as \"{}\"", username.as_str());
                Err(Error::new(
                    ErrorKind::Other,
                    format!("invalid user \"{}\"", username),
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
pub struct AuthenticationManager {
    pub htpasswd: Option<HtpasswdAuthenticationManager>,
}

impl AuthenticationManager {
    pub fn new(pwfile: &Option<PathBuf>) -> Self {
        let htpasswd: Option<HtpasswdAuthenticationManager> = match &pwfile {
            Some(path) => {
                let authentication_manager = HtpasswdAuthenticationManager::new(path).unwrap();
                Some(authentication_manager)
            }
            None => None,
        };
        AuthenticationManager { htpasswd }
    }

    pub async fn authenticate<R: AsyncRead + Unpin>(
        &self,
        reader: &mut BufReader<R>,
    ) -> Result<String> {
        // Read the mode.
        let mut mode = String::new();
        reader.read_line(&mut mode).await?;
        mode.truncate(mode.len() - 1); // Must have at least a single '\n';

        if mode == "none" {
            log::debug!("Authenticating with \"none\"");
            return Ok(String::from("nobody"));
        }

        if mode == "htpasswd" {
            log::debug!("Authenticating with \"htpasswd\"");
            return match &self.htpasswd {
                Some(auth) => auth.authenticate(reader).await,
                None => Err(Error::new(ErrorKind::Other, "no htpasswd auth")),
            };
        }

        Err(Error::new(ErrorKind::Other, format!("invalid mode {mode}")))
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
