use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, ErrorKind, Result},
};

use regex::Regex;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Role: u8 {
        const Subscriber = 0b00000001;
        const Notifier = 0b00000010;
        const Publisher = 0b00000100;
    }
}

pub type Entitlements = HashSet<i32>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Authorization {
    entitlements: HashSet<i32>,
    roles: Role,
}

pub type AuthorizationByTopic = HashMap<String, Authorization>;
pub type AuthorizationByUser = HashMap<String, AuthorizationByTopic>;

pub struct UserEntitlementsSpec {
    pub user_pattern: Regex,
    pub topic_pattern: Regex,
    pub entitlements: Entitlements,
    pub roles: Role,
}

impl UserEntitlementsSpec {
    pub fn new(
        user_pattern: Regex,
        topic_pattern: Regex,
        entitlements: Entitlements,
        roles: Role,
    ) -> Self {
        UserEntitlementsSpec {
            user_pattern,
            topic_pattern,
            entitlements,
            roles,
        }
    }
}

pub struct EntitlementsManager {
    specs: Vec<UserEntitlementsSpec>,
}

impl EntitlementsManager {
    pub fn new(specs: Vec<UserEntitlementsSpec>) -> Self {
        EntitlementsManager { specs }
    }

    pub fn entitlements(&self, user_name: &str, topic: &str, role: Role) -> Entitlements {
        let mut entitlements = Entitlements::new();

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

    pub fn reload(&mut self, specs: Vec<UserEntitlementsSpec>) {
        self.specs = specs;
    }

    pub fn load(path: &str) -> Result<Self> {
        let file = fs::File::open(path)?;
        let obj: AuthorizationByUser =
            serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))?;
        Self::from_obj(obj)
    }

    pub fn from_obj(contents: AuthorizationByUser) -> Result<Self> {
        let mut specs: Vec<UserEntitlementsSpec> = Vec::new();
        for (user_pattern, topic_authorization) in contents {
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
            UserEntitlementsSpec::new(
                Regex::new("joe").unwrap(),
                Regex::new(".*\\.LSE").unwrap(),
                HashSet::from([1, 2]),
                Role::Subscriber | Role::Notifier,
            ),
            UserEntitlementsSpec::new(
                Regex::new("joe").unwrap(),
                Regex::new(".*\\.NSE").unwrap(),
                HashSet::from([3, 4]),
                Role::Subscriber,
            ),
        ];
        let entitlements_manager = EntitlementsManager::new(user_entitlements_spec);

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
