use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, ErrorKind, Result},
};

use regex::Regex;

pub type Entitlements = HashSet<i32>;

pub struct UserEntitlementsSpec {
    pub user_pattern: Regex,
    pub topic_pattern: Regex,
    pub entitlements: Entitlements,
}

impl UserEntitlementsSpec {
    pub fn new(user_pattern: Regex, topic_pattern: Regex, entitlements: Entitlements) -> Self {
        UserEntitlementsSpec {
            user_pattern,
            topic_pattern,
            entitlements,
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

    pub fn user_entitlements(&self, user_name: &str, topic: &str) -> Entitlements {
        let mut entitlements = Entitlements::new();

        for spec in &self.specs {
            if spec.user_pattern.is_match(user_name) && spec.topic_pattern.is_match(topic) {
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
        let obj: HashMap<String, HashMap<String, Vec<i32>>> =
            serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))?;
        Self::from_obj(obj)
    }

    pub fn from_obj(contents: HashMap<String, HashMap<String, Vec<i32>>>) -> Result<Self> {
        let mut specs: Vec<UserEntitlementsSpec> = Vec::new();
        for (user_pattern, topic_entitlements) in contents {
            for (topic_pattern, entitlements) in topic_entitlements {
                let user_pattern = Regex::new(user_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let topic_pattern = Regex::new(topic_pattern.as_str())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
                let entitlements: HashSet<i32> = HashSet::from_iter(entitlements);
                specs.push(UserEntitlementsSpec {
                    user_pattern,
                    topic_pattern,
                    entitlements,
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
            ),
            UserEntitlementsSpec::new(
                Regex::new("joe").unwrap(),
                Regex::new(".*\\.NSE").unwrap(),
                HashSet::from([3, 4]),
            ),
        ];
        let entitlements_manager = EntitlementsManager::new(user_entitlements_spec);

        let actual = entitlements_manager.user_entitlements("joe", "TSCO.LSE");
        let expected: HashSet<i32> = HashSet::from([1, 2]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.user_entitlements("joe", "IBM.NSE");
        let expected: HashSet<i32> = HashSet::from([3, 4]);
        assert_eq!(actual, expected);

        let actual = entitlements_manager.user_entitlements("joe", "MSFT.NDAQ");
        let expected: HashSet<i32> = HashSet::from([]);
        assert_eq!(actual, expected);
    }
}
