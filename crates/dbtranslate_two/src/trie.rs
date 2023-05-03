use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// WHAT IS A TRIE?
/// A trie is a tree-like data structure that stores a dynamic set of strings.
/// It is a tree whose nodes are associated with the letters of an alphabet.
/// It is used to store strings to allow their quick retrieval.
/// Each node stores a character and a map of children nodes.

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Trie {
    pub children: HashMap<char, Trie>,
    pub is_end_of_word: bool,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            children: HashMap::new(),
            is_end_of_word: false,
        }
    }

    /// This function takes a slice of string references &[&str] as its argument 
    /// and creates a new Trie from the given keywords. The function iterates 
    /// over the keywords and constructs the trie using nested HashMaps.
    pub fn from_keywords(keywords: &[&str]) -> Self {
        let mut trie = Trie::new();

        for key in keywords {
            let mut current = &mut trie;
    
            for ch in key.chars() {
                current = current.children.entry(ch).or_insert_with(Trie::new);
            }
            current.is_end_of_word = true;
        }
    
        trie
    }

    /// This function takes a reference to a Trie and a string reference &str as 
    /// its arguments. It checks if the key is in the trie and returns a tuple with
    /// the TrieResult and a reference to the sub-trie where the search stopped.
    /// LIFETIME USAGE:
    /// By adding the lifetime specifier 'a to the function, we're giving a name 
    /// to this relationship between the input and output lifetimes. Specifically, 
    /// the syntax <'a> introduces a lifetime parameter called 'a. Then, 
    /// by annotating the input reference trie with &'a Trie and the output 
    /// reference &'a TrieNode, we're telling the compiler that both the input 
    /// and output references share the same lifetime 'a.
    pub fn search<'a>(&'a self, key: &str) -> (TrieResult, &'a Trie) {
        if key.is_empty() {
            return (TrieResult::NotFound, self);
        }
    
        let mut current = self;
    
        for ch in key.chars() {
            match current.children.get(&ch) {
                None => return (TrieResult::NotFound, current),
                Some(node) => current = node,
            }
        }
    
        if current.is_end_of_word {
            (TrieResult::Found, current)
        } else {
            (TrieResult::Prefix, current)
        }

    }

}

/// This display implementation only shows the is_end_of_word property if it 
/// is true, otherwise it leaves it out. We still need the bool in the data 
/// structure though, even if we don't need to show it.
impl fmt::Display for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "children: {:?}", self.children)?;
        if self.is_end_of_word {
            write!(f, ", is_end_of_word: true")?;
        }
        Ok(())
    }
}

/// Same thing as above
impl fmt::Debug for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "children: {:?}", self.children)?;
        if self.is_end_of_word {
            write!(f, ", is_end_of_word: true")?;
        }
        Ok(())
    }
}


/// This enum is introduced with three variants, NotFound, Prefix, and Found, 
/// to represent the result of searching a key in the trie.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TrieResult {
    NotFound,
    Prefix,
    Found,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test checks if the trie is constructed correctly from a list of keywords
    /// It then looks for the end of word keyword to ensure it exists in the trie.
    #[test]
    fn test_new_trie() {
        let keywords = ["bla", "foo", "blab"];
        let trie = Trie::from_keywords(&keywords);

        assert_eq!(
            trie.children.get(&'b').unwrap()
                .children.get(&'l').unwrap()
                .children.get(&'a').unwrap()
                .is_end_of_word,
            true
        );
        assert_eq!(
            trie.children.get(&'b').unwrap()
                .children.get(&'l').unwrap()
                .children.get(&'a').unwrap()
                .children.get(&'b').unwrap()
                .is_end_of_word,
            true
        );
        assert_eq!(
            trie.children.get(&'f').unwrap()
                .children.get(&'o').unwrap()
                .children.get(&'o').unwrap()
                .is_end_of_word,
            true
        );
    }

    /// This test checks if the search function correctly identifies whether 
    /// a key is not found, is a prefix, or is found in the trie.
    #[test]
    fn test_in_trie() {
        let keywords = ["cat"];
        let trie = Trie::from_keywords(&keywords);

        let (result1, _) = Trie::search(&trie, "bob");
        assert_eq!(result1, TrieResult::NotFound);

        let (result2, _) = Trie::search(&trie, "ca");
        assert_eq!(result2, TrieResult::Prefix);

        let (result3, _) = Trie::search(&trie, "cat");
        assert_eq!(result3, TrieResult::Found);
    }
}
