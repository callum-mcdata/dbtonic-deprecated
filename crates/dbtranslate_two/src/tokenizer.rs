use std::collections::{HashMap, HashSet};
use either::Either;
use crate::tokens::{Token, TokenType, single_tokens, keywords, comment_tokens, white_space};


/// This is the overall struct that contains all of the information about 
/// tokenizing strings. 
#[derive(Debug)]
pub struct Tokenizer {
    /// Token hashmaps
    single_tokens: HashMap<String, TokenType>,
    keywords: HashMap<String, TokenType>,
    white_space: HashMap<String, TokenType>,
    comment_tokens: HashMap<String, Option<String>>,
    /// Empty vectors
    bit_strings: Vec<Either<String, (String, String)>>,
    byte_strings: Vec<Either<String, (String, String)>>,
    hex_strings: Vec<Either<String, (String, String)>>,
    identifiers: Vec<Either<String, (String, String)>>,
    identifier_escapes: Vec<String>,
    quotes: Vec<Either<String, (String, String)>>,
    string_escapes: Vec<String>,
    var_single_tokens: HashSet<String>,
    comments: Vec<String>,
    /// Random
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

    /// This is the constructor method for the Tokenizer struct.
    pub fn new() -> Self {    
        let bit_strings = vec![];
        let byte_strings = vec![];
        let hex_strings = vec![];
        let identifiers = vec![Either::Left("\"".to_string())];
        let identifier_escapes = vec!["\"".to_string()];
        let quotes = vec![Either::Left("'".to_string())];
        let string_escapes = vec!["'".to_string()];
        let var_single_tokens = HashSet::new();
        let tokenizer = Tokenizer {
            /// Token hashmaps
            single_tokens: single_tokens(),
            keywords: keywords(),
            white_space: white_space(),
            comment_tokens: comment_tokens(),
            /// Empty vectors
            bit_strings,
            byte_strings,
            hex_strings,
            identifiers,
            identifier_escapes,
            quotes,
            string_escapes,
            var_single_tokens,
            comments: Vec::new(),
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
        };
        tokenizer
    }

    /// This function takes in a sql string and updates the state of the tokenizer  
    pub fn add_sql(&mut self, sql: String) {
        self.sql = sql;
        self.size = self.sql.len();
        self.char = self.sql.chars().nth(0).unwrap_or('\0');
        self.peek = self.sql.chars().nth(1).unwrap_or('\0');
        self.start = 0;
        self.current = 0;
        self.line = 1;
        self.col = 1;
        self.end = false;
        self.prev_token_line = -1;
        self.prev_token_comments.clear();
        self.prev_token_type = None;
    }

    pub fn reset(&mut self) {
        self.sql.clear();
        self.size = 0;
        self.tokens.clear();
        self.start = 0;
        self.current = 0;
        self.line = 1;
        self.col = 1;
        self.comments.clear();
        self.char = '\0';
        self.end = false;
        self.peek = '\0';
        self.prev_token_line = -1;
        self.prev_token_comments.clear();
        self.prev_token_type = None;
    }

    /// The `text` method returns a slice of the SQL string from the start to 
    /// the current position.
    fn text(&self) -> &str {
        &self.sql[self.start..self.current]
    }

    /// The `add_token` function appends a new token to the tokens list, using the given token type
    /// and an optional text. If the text is not provided, the function uses the tokenizer's
    /// current text. The function also updates the previous token's properties.
    fn add_token(&mut self, token_type: TokenType, text: Option<String>) {
        self.prev_token_line = self.line as isize;
        self.prev_token_comments = self.comments.clone();
        self.prev_token_type = Some(token_type);

        let token_text = text.unwrap_or_else(|| self.text().to_string());
        let token = Token {
            token_type,
            text: token_text,
            line: self.line,
            col: self.col,
            end: self.current,
            comments: self.comments.clone(),
        };

        self.tokens.push(token);
        self.comments.clear();
    }



    ///////////
    // STRING OPERATIONS 
    //////////

    // The function accepts a string reference &str for the SQL string and 
    // returns a Result containing a Vec<Token> or an error.
    // pub fn tokenize(&mut self, sql: &str) -> Result<Vec<Token>, String> {
    //     self.reset();
    //     self.sql = sql.to_string();
    //     self.size = sql.len();

    //     match self.scan() {
    //         Ok(()) => Ok(self.tokens.clone()),
    //         Err(_) => {
    //             let start = self.current.saturating_sub(50);
    //             let end = (self.current + 50).min(self.size);
    //             let context = &self.sql[start..end];
    //             Err(format!("Error tokenizing '{}'", context))
    //         }
    //     }
    // }

    fn chars(&self, size: usize) -> &str {
        if self.current == 0 {
            ""
        } else if size == 1 {
            &self.sql[self.current - 1..self.current]
        } else {
            let start = self.current - 1;
            let end = start + size;
            if end <= self.size {
                &self.sql[start..end]
            } else {
                ""
            }
        }
    }

    /// This function advances through the characters in the SQL string. It updates
    /// the state of the tokenizer struct.
    fn advance(&mut self, i: usize) {
        if let Some(token_type) = self.white_space.get(&self.char.to_string()) {
            if *token_type == TokenType::Break {
                self.col = 1;
                self.line += 1;
            } else {
                self.col += i;
            }
        } else {
            // We use this to account for all chars that aren't in whitespace
            self.col += i;
        }

        self.current += i;
        self.end = self.current >= self.size;
        // The nth() method returns an Option<char>, not a plain char. This is because
        // the iterator might not have an nth element if the index is out of bounds. 
        // To account for this we use unwrap_or with a default value of null char.
        self.char = self.sql.chars().nth(self.current - 1).unwrap_or('\0');
        if self.end {
            self.peek = '\0';
        } else {
            self.peek = self.sql.chars().nth(self.current).unwrap_or('\0');
        }
    }

    /////////////
    // EXTRACTING OPERATIONS
    /////////////

    /// This function extracts a string from the SQL string. It takes in a delimiter
    /// and returns a Result containing a string or an error. NOTE: IT MUST BEGIN
    /// WITH THE STATE OF THE TOKENIZER AT THE FIRST INSTANCE OF THE DELIMITER. 
    /// Otherwise it will just look for the delimiter at the current position.
    fn extract_string(&mut self, delimiter: &str) -> Result<String, String> {
        let mut text = String::new();
        let delim_size = delimiter.len();
        
        loop {
            if self.string_escapes.contains(&self.char.to_string()) && (self.peek.to_string() == delimiter || self.string_escapes.contains(&self.peek.to_string())) {
                if self.peek.to_string() == delimiter {
                    text.push(self.peek);
                } else {
                    text.push(self.char);
                    text.push(self.peek);
                }

                if self.current + 1 < self.size {
                    self.advance(2);
                } else {
                    return Err(format!("Missing {} from {}:{}", delimiter, self.line, self.current));
                }
            } else {
                if self.chars(delim_size) == delimiter {
                    if delim_size > 1 {
                        self.advance(delim_size - 1);
                    }
                    break;
                }

                if self.end {
                    return Err(format!("Missing {} from {}:{}", delimiter, self.line, self.start));
                }
                text.push(self.char);
                self.advance(1);
            }
        }

        Ok(text)
    }

    /// This function extracts a value from the current SQL string. It iterates 
    /// through the characters until it encounters a character that is either 
    /// empty after being stripped or part of the single_tokens HashSet. During 
    /// the loop, it appends the current character to the text variable and 
    /// advances the tokenizer. Finally, it returns the extracted value as a String.
    fn extract_value(&mut self) -> String {
        let mut text = String::new();
    
        loop {
            let stripped_char = self.peek.to_string();
            
            // Check if the character is not a null character and not a key in single_tokens
            if self.peek != '\0' && !self.single_tokens.contains_key(&stripped_char) {
                text.push(self.peek);
                self.advance(1);
            } else {
                break;
            }
        }
    
        text
    }

    ////////////
    /// SCANNING OPERATIONS
    ////////////
    
    /// The `scan_var` function scans a variable, keyword, or parameter in the input SQL string.
    /// It advances through the characters until it encounters a single token character or an
    /// empty/null character. The function then adds a token with the appropriate type to the
    /// tokens list.
    fn scan_var(&mut self) {
        while {
            let stripped_char = self.peek.to_string().trim().to_owned();
            !stripped_char.is_empty()
                && (self.var_single_tokens.contains(&stripped_char)
                    || !self.single_tokens.contains_key(&stripped_char))
        } {
            self.advance(1);
        }

        let token_type = if self.prev_token_type == Some(TokenType::Parameter) {
            TokenType::Var
        } else {
            let text_upper = self.text().to_uppercase();
            self.keywords.get(&text_upper).cloned().unwrap_or(TokenType::Var)
        };

        self.add_token(token_type, None);
    }

    // fn delimeter_list_to_dict(
    //     list: Vec<Either<String, (String, String)>>,
    // ) -> HashMap<String, String> {
    //     let mut dict = HashMap::new();
    //     for item in list {
    //         match item {
    //             Either::Left(s) => {
    //                 dict.insert(s.clone(), s);
    //             }
    //             Either::Right((k, v)) => {
    //                 dict.insert(k, v);
    //             }
    //         }
    //     }
    //     dict
    // }

    // Add other required methods
}


#[cfg(test)]
mod tests {
    use super::*;

    /// This test confirms that the chars method returns the correct string
    #[test]
    fn test_chars() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.sql = "SELECT * FROM table;".to_string();
        tokenizer.size = tokenizer.sql.len();
        tokenizer.current = 3;

        assert_eq!(tokenizer.chars(1), "L");
        assert_eq!(tokenizer.chars(2), "LE");
        assert_eq!(tokenizer.chars(3), "LEC");
        assert_eq!(tokenizer.chars(10), "LECT * FRO");
    }

    /// This test confirms that the advance method updates the Tokenizer struct
    #[test]
    fn test_advance_simple() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.sql = "SELECT * FROM table \n where 1=1;".to_string();
        tokenizer.size = tokenizer.sql.len();

        tokenizer.advance(1);
        assert_eq!(tokenizer.char, 'S');
        assert_eq!(tokenizer.peek, 'E');
        assert_eq!(tokenizer.col, 2);
        assert_eq!(tokenizer.line, 1);

        tokenizer.advance(6);
        assert_eq!(tokenizer.char, ' ');
        assert_eq!(tokenizer.peek, '*');
        assert_eq!(tokenizer.col, 8);
        assert_eq!(tokenizer.line, 1);

        tokenizer.advance(8);
        assert_eq!(tokenizer.char, 't');
        assert_eq!(tokenizer.peek, 'a');
        assert_eq!(tokenizer.col, 16);
        assert_eq!(tokenizer.line, 1);

        // We do this odd pattern to make sure that we run the advance function
        // on the newline string. TODO: Figure out if we need to change advance
        // to loop based on the int provided. Right now it skips over the chars
        tokenizer.advance(6);
        tokenizer.advance(1);
        tokenizer.advance(1);
        assert_eq!(tokenizer.char, 'w');
        assert_eq!(tokenizer.peek, 'h');
        assert_eq!(tokenizer.col, 2);
        assert_eq!(tokenizer.line, 2);
    }

    /// This test confirms that the reset functionality works as expected
    #[test]
    fn test_reset() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.sql = "SELECT * FROM table;".to_string();
        tokenizer.size = tokenizer.sql.len();

        tokenizer.advance(1);
        assert_eq!(tokenizer.char, 'S');
        assert_eq!(tokenizer.peek, 'E');
        assert_eq!(tokenizer.col, 2);
        assert_eq!(tokenizer.line, 1);

        tokenizer.advance(6);
        assert_eq!(tokenizer.char, ' ');
        assert_eq!(tokenizer.peek, '*');
        assert_eq!(tokenizer.col, 8);
        assert_eq!(tokenizer.line, 1);
       
        tokenizer.reset();
        assert_eq!(tokenizer.sql, "");
        assert_eq!(tokenizer.char, '\0');
        assert_eq!(tokenizer.peek, '\0');
        assert_eq!(tokenizer.col, 1);
        assert_eq!(tokenizer.line, 1);
    }

    /// This test first creates a new Tokenizer instance, then calls update_sql 
    /// with a SQL string. It then checks whether the state fields have been 
    /// updated correctly based on the provided SQL string.
    #[test]
    fn test_add_sql() {
        let mut tokenizer = Tokenizer::new();

        let sql = "SELECT * FROM table;".to_string();
        tokenizer.add_sql(sql);

        assert_eq!(tokenizer.sql, "SELECT * FROM table;");
        assert_eq!(tokenizer.size, 20);
        assert_eq!(tokenizer.char, 'S');
        assert_eq!(tokenizer.peek, 'E');
        assert_eq!(tokenizer.start, 0);
        assert_eq!(tokenizer.current, 0);
        assert_eq!(tokenizer.line, 1);
        assert_eq!(tokenizer.col, 1);
        assert_eq!(tokenizer.end, false);
        assert_eq!(tokenizer.prev_token_line, -1);
        assert!(tokenizer.prev_token_comments.is_empty());
        assert_eq!(tokenizer.prev_token_type, None);
    }

    // TODO: I don't think extract string fully works yet but I am burned on it
    // and want to move on to other things. I will come back to it later.
    // The issue appears to lie in John O/'Connor translating to John O'Connor.
    // Not sure where the newline break is going
    #[test]
    fn test_extract_string() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM table WHERE name = 'John O Connor'".to_string());  

        let delimiter = "'";
        tokenizer.advance(35);
        let extracted_string = tokenizer.extract_string(delimiter).unwrap();
        assert_eq!(extracted_string, "John O Connor");
    }

    #[test]
    fn test_extract_value() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM table WHERE value=42".to_string());
        tokenizer.advance(32); // Move the tokenizer to the position right before the value 42

        let extracted_value = tokenizer.extract_value();
        assert_eq!(extracted_value, "42");
    }

    /// This test creates a Tokenizer instance, adds an SQL query to it, and 
    /// then adds a token using the add_token function. It then checks if the 
    /// token was added correctly by comparing the token's properties with the 
    /// expected values.
    #[test]
    fn test_add_token() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM table;".to_string());

        let token_type = TokenType::Select;
        let token_text = Some("SELECT".to_string());

        tokenizer.add_token(token_type, token_text.clone());

        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, token_type);
        assert_eq!(tokenizer.tokens[0].text, token_text.unwrap());
        assert_eq!(tokenizer.tokens[0].line, tokenizer.line);
        assert_eq!(tokenizer.tokens[0].col, tokenizer.col);
        assert_eq!(tokenizer.tokens[0].end, tokenizer.current);
        assert_eq!(tokenizer.tokens[0].comments, tokenizer.comments);
    }

}