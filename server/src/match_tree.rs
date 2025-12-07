use std::{collections::HashMap, io};

use crate::constants::{LEVEL_SEPARATOR, MULTI_LEVEL_WILDCARD, SINGLE_LEVEL_WILDCARD};

#[derive(Debug, Clone)]
struct Node {
    children: HashMap<String, Node>, // level -> node
}

impl Node {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchTree {
    root_node: Node,
}

impl MatchTree {
    pub fn new() -> Self {
        Self {
            root_node: Node::new(),
        }
    }

    pub fn create(pattern: &str) -> io::Result<Self> {
        let mut tree = Self::new();
        tree.add(pattern)?;
        Ok(tree)
    }

    pub fn add(&mut self, pattern: &str) -> io::Result<()> {
        if pattern.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Pattern cannot be empty".to_string(),
            ));
        }

        let words: Vec<&str> = pattern.split(LEVEL_SEPARATOR).collect();
        if words[0..words.len() - 1]
            .iter()
            .any(|&x| x == MULTI_LEVEL_WILDCARD)
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Multi level wildcard must be last".to_string(),
            ));
        }

        let mut node = &mut self.root_node;
        for word in words {
            if !node.children.contains_key(word) {
                node.children.insert(word.into(), Node::new());
            }
            node = node.children.get_mut(word).unwrap();
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, pattern: &str) -> Option<()> {
        let mut tree = &mut self.root_node;
        let levels: Vec<&str> = pattern.split(LEVEL_SEPARATOR).collect();
        for index in 0..(levels.len() - 1) {
            let level = levels[index];
            let Some(child) = tree.children.get_mut(level) else {
                return None;
            };
            tree = child;
        }
        let level = levels[levels.len() - 1];
        tree.children.remove(level).map(|_| ())
    }

    pub fn is_match(&self, topic: &str) -> bool {
        let mut trees = vec![&self.root_node];
        for level in topic.split(LEVEL_SEPARATOR) {
            let mut match_trees: Vec<&Node> = Vec::new();
            for tree in trees {
                if let Some(child) = tree.children.get(level) {
                    match_trees.push(child);
                }
                if let Some(child) = tree.children.get(SINGLE_LEVEL_WILDCARD) {
                    match_trees.push(child);
                }
                if let Some(_child) = tree.children.get(MULTI_LEVEL_WILDCARD) {
                    return true;
                }
            }
            trees = match_trees;
        }

        if trees.len() > 0 {
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.root_node.children.clear();
    }
}

#[cfg(test)]
mod test_topic_tree {

    use super::*;

    #[test]
    fn should_not_smoke() {
        let mut match_tree: MatchTree = MatchTree::new();

        match_tree.add("LSE.?").unwrap();
        match_tree.add("NYSE.?").unwrap();
        assert!(match_tree.is_match("LSE.VOD"));
        assert!(match_tree.is_match("NYSE.GS"));
        assert!(!match_tree.is_match("NASDAQ.MSFT"));

        match_tree.remove("NYSE.?");
        assert!(!match_tree.is_match("NYSE.GS"));

        match_tree.clear();
        match_tree.add("X.*").unwrap();
        assert!(match_tree.is_match("X.a"));
        assert!(match_tree.is_match("X.a.b"));
        assert!(!match_tree.is_match("Y"));
        assert!(!match_tree.is_match("Y.a"));

        match_tree.clear();
        match_tree.add("*").unwrap();
        assert!(match_tree.is_match("X"));
        assert!(match_tree.is_match("Y"));
        assert!(match_tree.is_match("a.b.c"));
    }
}
