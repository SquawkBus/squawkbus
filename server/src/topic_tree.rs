use std::collections::{HashMap, HashSet, VecDeque};

pub const LEVEL_SEPARATOR: &str = ".";
pub const MULTI_LEVEL_WILDCARD: &str = "*";
pub const SINGLE_LEVEL_WILDCARD: &str = "?";

struct LevelNode {
    subscribers: HashMap<String, u32>,
    children: HashMap<String, LevelNode>,
}

impl LevelNode {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

pub struct TopicTree {
    patterns: LevelNode,
}

impl TopicTree {
    pub fn new() -> Self {
        Self {
            patterns: LevelNode::new(),
        }
    }

    pub fn add(&mut self, pattern: &str, subscriber: String) -> Result<u32, String> {
        if pattern.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        let levels: Vec<&str> = pattern.split(LEVEL_SEPARATOR).collect();
        if levels[0..levels.len() - 1]
            .iter()
            .any(|&x| x == MULTI_LEVEL_WILDCARD)
        {
            return Err("Multi level wildcard must be last".to_string());
        }

        let mut tree = &mut self.patterns;
        for level in levels {
            if !tree.children.contains_key(level) {
                tree.children.insert(level.into(), LevelNode::new());
            }
            tree = tree.children.get_mut(level).unwrap();
        }
        if let Some(count) = tree.subscribers.get_mut(&subscriber) {
            *count += 1;
            Ok(*count)
        } else {
            tree.subscribers.insert(subscriber, 1);
            Ok(1)
        }
    }

    pub fn remove(&mut self, pattern: &str, subscriber: &str, remove_all: bool) -> Option<u32> {
        let mut tree = &mut self.patterns;
        for level in pattern.split(LEVEL_SEPARATOR) {
            let Some(child) = tree.children.get_mut(level) else {
                return None;
            };
            tree = child;
        }

        let Some(count) = tree.subscribers.get_mut(subscriber) else {
            return None;
        };

        if remove_all {
            *count = 0
        } else {
            *count -= 1;
        }

        if *count > 0 {
            Some(*count)
        } else {
            tree.subscribers.remove(subscriber);
            Some(0)
        }
    }

    // pub fn exists(&self, topic: &str, subscriber: &str) -> bool {
    //     let mut tree = &self.patterns;
    //     for level in topic.split(LEVEL_SEPARATOR) {
    //         let Some(child) = tree.children.get(level) else {
    //             return false;
    //         };
    //         tree = child;
    //     }
    //     tree.subscribers.contains_key(subscriber)
    // }

    pub fn subscribers(&self, topic: &str) -> Vec<&str> {
        let mut trees = vec![&self.patterns];
        let mut subscribers: Vec<&str> = Vec::new();
        for level in topic.split(LEVEL_SEPARATOR) {
            let mut match_trees: Vec<&LevelNode> = Vec::new();
            for tree in trees {
                if let Some(child) = tree.children.get(level) {
                    match_trees.push(child);
                }
                if let Some(child) = tree.children.get(SINGLE_LEVEL_WILDCARD) {
                    match_trees.push(child);
                }
                if let Some(child) = tree.children.get(MULTI_LEVEL_WILDCARD) {
                    subscribers.extend(child.subscribers.iter().map(|x| x.0.as_str()));
                }
            }
            trees = match_trees;
        }

        for tree in trees {
            subscribers.extend(tree.subscribers.iter().map(|x| x.0.as_str()));
        }

        subscribers
    }

    pub fn topics(&self, subscriber: &str) -> HashSet<String> {
        let mut subscribed_topics: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(&LevelNode, Vec<&str>)> = VecDeque::new();
        let mut visited: HashSet<Vec<&str>> = HashSet::new();

        for (key, node) in &self.patterns.children {
            queue.push_back((node, vec![key.as_str()]));
        }

        while !queue.is_empty() {
            let (current, levels) = queue.pop_front().unwrap();
            if current.subscribers.contains_key(subscriber) {
                subscribed_topics.insert(levels.join(LEVEL_SEPARATOR));
            }

            for (child_key, child_node) in &current.children {
                let mut child_level = levels.clone();
                child_level.push(child_key.as_str());
                if !visited.contains(&child_level) {
                    visited.insert(child_level.clone());
                    queue.push_back((child_node, child_level));
                }
            }
        }

        subscribed_topics
    }
}

#[cfg(test)]
mod test_topic_tree {

    use super::*;

    #[test]
    fn should_not_smoke() {
        let mut manager: TopicTree = TopicTree::new();

        manager.add("home.kitchen.temperature", "1".into()).unwrap();
        manager.add("home.kitchen.?", "2".into()).unwrap();
        manager.add("home.*", "3".into()).unwrap();
        manager.add("*", "4".into()).unwrap();

        let subscribers = manager.subscribers("home.kitchen.temperature");
        assert_eq!(subscribers, vec!["4", "3", "1", "2"]);

        let subscribers = manager.subscribers("home.kitchen.lighting");
        assert_eq!(subscribers, vec!["4", "3", "2"]);

        let subscribers = manager.subscribers("home.lounge.temperature");
        assert_eq!(subscribers, vec!["4", "3"]);
    }
}
