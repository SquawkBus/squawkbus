use std::io::{Error, ErrorKind, Result};

use async_trait::async_trait;
use http_auth_basic::Credentials;
use ldap3::{LdapConnAsync, LdapConnSettings};

use crate::authentication::traits::Authenticator;

#[derive(Clone)]
pub struct LdapAuthenticator {
    url: String,
}

impl LdapAuthenticator {
    pub fn new(url: String) -> LdapAuthenticator {
        LdapAuthenticator { url }
    }
}

#[async_trait]
impl Authenticator for LdapAuthenticator {
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
                    "Failed to authenticate as \"{}\".",
                    credentials.user_id.as_str()
                );
                Err(Error::new(
                    ErrorKind::Other,
                    format!("invalid user \"{}\"", credentials.user_id),
                ))
            }
            false => {
                log::info!("Authenticated as \"{}\".", credentials.user_id.as_str());
                Ok(credentials.user_id)
            }
        }
    }

    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}
