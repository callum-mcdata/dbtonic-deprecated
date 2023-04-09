use std::fmt;
use std::collections::HashMap;

pub struct Trie {
    children: HashMap<char, Box<Trie>>,
    is_end_of_word: bool,
}

impl Trie {
    pub fn create() -> Self {
        Trie {
            children: HashMap::new(),
            is_end_of_word: false,
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut current = self;
        let mut chars = word.chars().peekable();
    
        while let Some(c) = chars.next() {
            if chars.peek().is_none() {
                let child = current.children.entry(c).or_insert_with(|| Box::new(Trie::create()));
                child.is_end_of_word = true;
                break;
            }
            current = current.children.entry(c).or_insert_with(|| Box::new(Trie::create()));
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        let mut current = self;

        for c in word.chars() {
            match current.children.get(&c) {
                Some(child) => current = child,
                None => return false,
            }
        }
        current.is_end_of_word
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        let mut current = self;

        for c in prefix.chars() {
            match current.children.get(&c) {
                Some(child) => current = child,
                None => return false,
            }
        }
        true
    }

    pub fn search_key(&self, key: &str) -> (u8, &Trie) {
        if key.is_empty() {
            return (0, self);
        }

        let mut current = self;

        for c in key.chars() {
            match current.children.get(&c) {
                Some(child) => current = child,
                None => return (0, current),
            }
        }

        if current.is_end_of_word {
            return (2, current);
        }
        return (1, current);
    }

}

impl fmt::Debug for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn build_debug_string(trie: &Trie, indent: usize) -> String {
            let mut result = String::new();
            let mut node_contents: Vec<String> = vec![];

            if trie.is_end_of_word {
                node_contents.push("0: true".to_string());
            }

            for (c, child) in trie.children.iter() {
                let child_string = build_debug_string(child, indent + 2);
                let formatted_child = format!("'{}': {}", c, child_string);
                node_contents.push(formatted_child);
            }

            result.push_str("{");
            result.push_str(&node_contents.join(", "));
            result.push_str("}");

            result
        }

        write!(f, "{}", build_debug_string(self, 0))
    }
}

impl fmt::Display for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut trie = Trie::create();
        trie.insert("hello");
        trie.insert("world");
        trie.insert("rust");

        assert!(trie.contains("hello"));
        assert!(trie.contains("world"));
        assert!(trie.contains("rust"));
        assert!(!trie.contains("python"));
    }

    #[test]
    fn test_insert_and_prefix_search() {
        let mut trie = Trie::create();
        trie.insert("apple");
        trie.insert("app");
        trie.insert("banana");
        trie.insert("bat");

        let prefixes = trie.starts_with("app");
        assert_eq!(prefixes, true);
    }

    #[test]
    fn test_empty_trie() {
        let trie = Trie::create();

        assert!(!trie.contains("hello"));
        assert_eq!(trie.starts_with("rust"), false);
    }

    #[test]
    fn test_single_char_keywords() {
        let mut trie = Trie::create();
        trie.insert("a");
        trie.insert("b");
        trie.insert("c");

        assert!(trie.contains("a"));
        assert!(trie.contains("b"));
        assert!(trie.contains("c"));
        assert!(!trie.contains("d"));
    }

    #[test]
    fn test_search_key() {
        let mut trie = Trie::create();
        trie.insert("cat");
        trie.insert("dog");
        trie.insert("cart");

        let (value, _) = trie.search_key("bob");
        assert_eq!(value, 0, "Expected 'bob' not to be in the trie");

        let (value, _) = trie.search_key("ca");
        assert_eq!(value, 1, "Expected 'ca' to be a prefix in the trie");

        let (value, _) = trie.search_key("cat");
        assert_eq!(value, 2, "Expected 'cat' to be in the trie");

        let (value, _) = trie.search_key("cart");
        assert_eq!(value, 2, "Expected 'cart' to be in the trie");

        let (value, _) = trie.search_key("car");
        assert_eq!(value, 1, "Expected 'car' to be a prefix in the trie");

        let (value, _) = trie.search_key("do");
        assert_eq!(value, 1, "Expected 'do' to be a prefix in the trie");

        let (value, _) = trie.search_key("dog");
        assert_eq!(value, 2, "Expected 'dog' to be in the trie");
    }

    #[test]
    fn test_trie_display() {
        let mut trie = Trie::create();
        trie.insert("bla");
        trie.insert("foo");
        trie.insert("blab");
    
        let trie_display = trie.to_string();
    
        // Check for specific substrings instead of comparing the entire output string
        assert!(trie_display.contains("'f': {'o': {'o': {0: true}}}"));
        assert!(trie_display.contains("'b': {'l': {'a': {0: true, 'b': {0: true}}}}"));
    }

}
