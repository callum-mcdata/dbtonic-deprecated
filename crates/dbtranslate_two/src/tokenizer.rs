use std::collections::{HashMap, HashSet};
use either::Either;
use crate::tokens::{Token, TokenType, single_tokens, keywords, comments, commands, white_space};


/// This is the overall struct that contains all of the information about 
/// tokenizing strings. 
#[derive(Debug)]
pub struct Tokenizer {
    /// Token hashmaps
    single_tokens: HashMap<String, TokenType>,
    keywords: HashMap<String, TokenType>,
    white_space: HashMap<String, TokenType>,
    comments: HashMap<String, Option<String>>,
    /// Empty vectors
    bit_strings: Vec<Either<String, (String, String)>>,
    byte_strings: Vec<Either<String, (String, String)>>,
    hex_strings: Vec<Either<String, (String, String)>>,
    identifiers: Vec<Either<String, (String, String)>>,
    identifier_escapes: Vec<String>,
    quotes: Vec<Either<String, (String, String)>>,
    string_escapes: Vec<String>,
    /// 
    numeric_literals: HashMap<String, String>,
    encode: Option<String>,
    identifier_can_start_with_digit: bool,
    /// State properties
    sql: String,
    size: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
    sql_comments: Vec<String>,
    char: char,
    end: bool,
    peek: char,
    prev_token_line: isize,
    prev_token_comments: Vec<String>,
    prev_token_type: Option<TokenType>,
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
            /// Token hashmaps
            single_tokens: single_tokens(),
            keywords: keywords(),
            white_space: white_space(),
            comments: comments(),
            /// Empty vectors
            bit_strings,
            byte_strings,
            hex_strings,
            identifiers,
            identifier_escapes,
            quotes,
            string_escapes,
            // ... add other field assignments
            numeric_literals: HashMap::new(),
            encode: None,
            identifier_can_start_with_digit: false,
            /// State management
            sql: String::new(),
            size: 0,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            col: 1,
            sql_comments: Vec::new(),
            char: '\0',
            end: false,
            peek: '\0',
            prev_token_line: -1,
            prev_token_comments: Vec::new(),
            prev_token_type: None,
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