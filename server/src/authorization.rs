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
        const Publisher = 0b00000010;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Authorization {
    pub entitlements: HashSet<i32>,
    pub roles: Role,
}

#[derive(Debug, Clone)]
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

    pub fn reset(&mut self, specs: Vec<AuthorizationSpec>) {
        self.specs = specs
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
}

pub fn load_authorizations<P>(
    path: &Option<P>,
    specs: &[AuthorizationSpec],
) -> Result<Vec<AuthorizationSpec>>
where
    P: AsRef<Path>,
{
    let mut specs: Vec<AuthorizationSpec> = specs.to_vec(); // specs.iter().map(|x| *x.clone()).collect();

    // Either load from a file, or provide useful defaults.
    match path {
        Some(path) => {
            let file = fs::File::open(path)?;
            let authorizations: HashMap<String, HashMap<String, Authorization>> =
                serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            for (user, topic_authorization) in authorizations {
                for (topic, authorization) in topic_authorization {
                    let user_pattern = Regex::new(user.as_str())
                        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                    let topic_pattern = Regex::new(topic.as_str())
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
        }
        None => {
            if specs.is_empty() {
                // Allow anyone to send anything
                let user = ".*";
                let topic = ".*";
                let entitlements = HashSet::from([0]);
                let roles = Role::Subscriber | Role::Publisher;

                let user_pattern =
                    Regex::new(user).map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let topic_pattern =
                    Regex::new(topic).map_err(|e| io::Error::new(ErrorKind::Other, e))?;

                let spec = AuthorizationSpec {
                    user_pattern,
                    topic_pattern,
                    entitlements,
                    roles,
                };
                specs.push(spec)
            }
        }
    };

    Ok(specs)
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
                roles: Role::Subscriber | Role::Publisher,
            },
            AuthorizationSpec {
                user_pattern: Regex::new("joe").unwrap(),
                topic_pattern: Regex::new(".*\\.LSE").unwrap(),
                entitlements: HashSet::from([1, 2]),
                roles: Role::Subscriber,
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

        let actual = entitlements_manager.entitlements("joe", "TSCO.LSE", Role::Subscriber);
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
