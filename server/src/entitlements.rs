use std::{
    collections::HashSet,
    io::{self, ErrorKind, Result},
};

use regex::Regex;

use crate::config::{Config, Role};

pub struct UserEntitlementsSpec {
    pub user_pattern: Regex,
    pub topic_pattern: Regex,
    pub entitlements: HashSet<i32>,
    pub roles: Role,
}

pub struct EntitlementsManager {
    specs: Vec<UserEntitlementsSpec>,
}

impl EntitlementsManager {
    pub fn new(specs: Vec<UserEntitlementsSpec>) -> Self {
        EntitlementsManager { specs }
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

    pub fn from_config(config: Config) -> Result<EntitlementsManager> {
        let mut specs: Vec<UserEntitlementsSpec> = Vec::new();
        for (user_pattern, topic_authorization) in config.authorization {
            for (topic_pattern, authorization) in topic_authorization {
                let user_pattern = Regex::new(user_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let topic_pattern = Regex::new(topic_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let entitlements: HashSet<i32> = HashSet::from_iter(authorization.entitlements);
                let roles = authorization.roles;
                specs.push(UserEntitlementsSpec {
                    user_pattern,
                    topic_pattern,
                    entitlements,
                    roles,
                });
            }
        }
        Ok(EntitlementsManager::new(specs))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let user_entitlements_spec = vec![
            UserEntitlementsSpec {
                user_pattern: Regex::new(".*").unwrap(),
                topic_pattern: Regex::new("PUB\\..*").unwrap(),
                entitlements: HashSet::from([0]),
                roles: Role::Subscriber | Role::Notifier | Role::Publisher,
            },
            UserEntitlementsSpec {
                user_pattern: Regex::new("joe").unwrap(),
                topic_pattern: Regex::new(".*\\.LSE").unwrap(),
                entitlements: HashSet::from([1, 2]),
                roles: Role::Subscriber | Role::Notifier,
            },
            UserEntitlementsSpec {
                user_pattern: Regex::new("joe").unwrap(),
                topic_pattern: Regex::new(".*\\.NSE").unwrap(),
                entitlements: HashSet::from([3, 4]),
                roles: Role::Subscriber,
            },
        ];
        let entitlements_manager = EntitlementsManager::new(user_entitlements_spec);

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
