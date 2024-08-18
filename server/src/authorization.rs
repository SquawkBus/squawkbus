use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Result};
use std::path::Path;

use bitflags::bitflags;
use regex::Regex;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Role: u8 {
        const Subscriber = 0b00000001;
        const Notifier = 0b00000010;
        const Publisher = 0b00000100;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Authorization {
    pub entitlements: HashSet<i32>,
    pub roles: Role,
}

pub struct AuthorizationSpec {
    pub user_pattern: Regex,
    pub topic_pattern: Regex,
    pub entitlements: HashSet<i32>,
    pub roles: Role,
}

pub struct AuthorizationManager {
    specs: Vec<AuthorizationSpec>,
}

impl AuthorizationManager {
    pub fn new(specs: Vec<AuthorizationSpec>) -> Self {
        AuthorizationManager { specs }
    }

    pub fn entitlements(&self, user_name: &str, topic: &str, role: Role) -> HashSet<i32> {
        let mut entitlements = HashSet::new();

        for spec in &self.specs {
            if spec.roles.contains(role)
                && spec.user_pattern.is_match(user_name)
                && spec.topic_pattern.is_match(topic)
            {
                entitlements.extend(spec.entitlements.iter());
            }
        }

        entitlements
    }

    pub fn from_config(
        authorizations: HashMap<String, HashMap<String, Authorization>>,
    ) -> Result<AuthorizationManager> {
        let mut specs: Vec<AuthorizationSpec> = Vec::new();
        for (user_pattern, topic_authorization) in authorizations {
            for (topic_pattern, authorization) in topic_authorization {
                let user_pattern = Regex::new(user_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let topic_pattern = Regex::new(topic_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let entitlements: HashSet<i32> = HashSet::from_iter(authorization.entitlements);
                let roles = authorization.roles;
                specs.push(AuthorizationSpec {
                    user_pattern,
                    topic_pattern,
                    entitlements,
                    roles,
                });
            }
        }
        Ok(AuthorizationManager::new(specs))
    }
}

fn default_authorization() -> HashMap<String, HashMap<String, Authorization>> {
    // The default is to authorize all users for all roles on "PUB.*".
    HashMap::from([(
        String::from(".*"),
        HashMap::from([(
            String::from("PUB\\..*"),
            Authorization {
                entitlements: HashSet::from([0]),
                roles: Role::Publisher | Role::Subscriber | Role::Notifier,
            },
        )]),
    )])
}

pub fn load_authorizations<P>(
    path: Option<P>,
) -> Result<HashMap<String, HashMap<String, Authorization>>>
where
    P: AsRef<Path>,
{
    match path {
        Some(path) => {
            let file = fs::File::open(path)?;
            serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))
        }
        None => Ok(default_authorization()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let user_entitlements_spec = vec![
            AuthorizationSpec {
                user_pattern: Regex::new(".*").unwrap(),
                topic_pattern: Regex::new("PUB\\..*").unwrap(),
                entitlements: HashSet::from([0]),
                roles: Role::Subscriber | Role::Notifier | Role::Publisher,
            },
            AuthorizationSpec {
                user_pattern: Regex::new("joe").unwrap(),
                topic_pattern: Regex::new(".*\\.LSE").unwrap(),
                entitlements: HashSet::from([1, 2]),
                roles: Role::Subscriber | Role::Notifier,
            },
            AuthorizationSpec {
                user_pattern: Regex::new("joe").unwrap(),
                topic_pattern: Regex::new(".*\\.NSE").unwrap(),
                entitlements: HashSet::from([3, 4]),
                roles: Role::Subscriber,
            },
        ];
        let entitlements_manager = AuthorizationManager::new(user_entitlements_spec);

        let actual = entitlements_manager.entitlements("nobody", "PUB.foo", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([0]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("nobody", "PUB.foo", Role::Publisher);
        let expected: HashSet<i32> = HashSet::from([0]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("nobody", "PUB.foo", Role::Notifier);
        let expected: HashSet<i32> = HashSet::from([0]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("joe", "TSCO.LSE", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([1, 2]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("joe", "TSCO.LSE", Role::Notifier);
        let expected: HashSet<i32> = HashSet::from([1, 2]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("joe", "TSCO.LSE", Role::Publisher);
        assert!(actual.is_empty());

        let actual = entitlements_manager.entitlements("joe", "IBM.NSE", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([3, 4]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.entitlements("joe", "MSFT.NDAQ", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([]);
        assert_eq!(actual, expected);
    }
}
