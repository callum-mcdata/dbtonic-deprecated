use std::collections::{HashMap, HashSet};
use either::Either;
use crate::tokens::{Token, TokenType, single_tokens, keywords, commands, white_space};


/// This is the overall struct that contains all of the information about 
/// tokenizing strings. 
#[derive(Debug)]
pub struct Tokenizer {
    single_tokens: HashMap<String, TokenType>,
    bit_strings: Vec<Either<String, (String, String)>>,
    byte_strings: Vec<Either<String, (String, String)>>,
    hex_strings: Vec<Either<String, (String, String)>>,
    identifiers: Vec<Either<String, (String, String)>>,
    identifier_escapes: Vec<String>,
    quotes: Vec<Either<String, (String, String)>>,
    string_escapes: Vec<String>,
    // ... add other fields
}

/// These are the implementation methods that are required for the Tokenizer struct.
impl Tokenizer {
    pub fn new() -> Self {    

        let bit_strings = vec![];
        let byte_strings = vec![];
        let hex_strings = vec![];
        let identifiers = vec![Either::Left("\"".to_string())];
        let identifier_escapes = vec!["\"".to_string()];
        let quotes = vec![Either::Left("'".to_string())];
        let string_escapes = vec!["'".to_string()];

        // ... add other initializations

        Tokenizer {
            single_tokens: single_tokens(),
            bit_strings,
            byte_strings,
            hex_strings,
            identifiers,
            identifier_escapes,
            quotes,
            string_escapes,
            // ... add other field assignments
        }
    }

    fn delimeter_list_to_dict(
        list: Vec<Either<String, (String, String)>>,
    ) -> HashMap<String, String> {
        let mut dict = HashMap::new();
        for item in list {
            match item {
                Either::Left(s) => {
                    dict.insert(s.clone(), s);
                }
                Either::Right((k, v)) => {
                    dict.insert(k, v);
                }
            }
        }
        dict
    }

    // Add other required methods
}