// Command line parameters.

use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

use argh::FromArgs;
use regex::Regex;

use crate::authorization::{AuthorizationSpec, Role};

fn default_endpoint() -> String {
    String::from("0.0.0.0:8080")
}

impl FromStr for AuthorizationSpec {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let args: Vec<&str> = s.split(':').collect();
        if args.len() != 4 {
            return Err(format!("expected 4 parts, found {}", args.len()));
        }
        let user_pattern = Regex::new(args[0]).map_err(|e| format!("invalid regex: {}", e))?;
        let topic_pattern = Regex::new(args[1]).map_err(|e| format!("invalid regex: {}", e))?;
        let entitlements = args[2]
            .split(',')
            .map(|x| x.parse().map_err(|e| format!("invalid entitlement {}", e)))
            .collect::<std::result::Result<HashSet<i32>, String>>()?;
        let roles: Role =
            bitflags::parser::from_str(args[3]).map_err(|e| format!("invalid roles: {}", e))?;
        Ok(AuthorizationSpec {
            user_pattern,
            topic_pattern,
            entitlements,
            roles,
        })
    }
}

/// SquawkBus server.
#[derive(FromArgs)]
pub struct Options {
    /// an optional authorizations file.
    #[argh(option, short = 'f')]
    pub authorizations: Option<PathBuf>,

    /// endpoint - defaults to 0.0.0.0:8080
    #[argh(option, short = 'e', default = "default_endpoint()")]
    pub endpoint: String,

    /// use tls
    #[argh(switch, short = 't')]
    pub tls: bool,

    /// cert file
    #[argh(option, short = 'c')]
    pub certfile: Option<PathBuf>,

    /// key file
    #[argh(option, short = 'k')]
    pub keyfile: Option<PathBuf>,

    /// authorization
    #[argh(option, short = 'a')]
    pub authorization: Vec<AuthorizationSpec>,
}

impl Options {
    pub fn load() -> Self {
        argh::from_env()
    }
}

#[cfg(test)]
mod test {
    use crate::authorization::AuthorizationManager;

    use super::*;

    #[test]
    fn parse() {
        let spec: AuthorizationSpec =
            AuthorizationSpec::from_str(".*:PUB\\.*:1,2:Subscriber|Publisher").unwrap();
        let user_entitlements_spec = vec![spec];
        let entitlements_manager = AuthorizationManager::new(user_entitlements_spec);

        let actual = entitlements_manager.entitlements("nobody", "PUB.foo", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([1, 2]);
        assert_eq!(actual, expected);
    }
}
