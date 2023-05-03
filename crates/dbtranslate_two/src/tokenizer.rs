use std::collections::{HashMap, HashSet};
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
    bit_strings: HashMap<String, String>,
    byte_strings: HashMap<String, String>,
    hex_strings: HashMap<String, String>,
    identifiers: HashMap<String, String>,
    identifier_escapes: Vec<String>,
    quotes: HashMap<String, String>,
    string_escapes: Vec<String>,
    var_single_tokens: HashSet<String>,
    /// Random
    numeric_literals: HashMap<String, String>,
    identifier_can_start_with_digit: bool,
    /// State properties
    sql: String,
    size: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
    comments: Vec<String>,
    char: char,
    end: bool,
    peek: char,
    prev_token_line: usize,
    prev_token_comments: Vec<String>,
    prev_token_type: Option<TokenType>,
}

/// These are the implementation methods that are required for the Tokenizer struct.
impl Tokenizer {

    /// This is the constructor method for the Tokenizer struct.
    pub fn new() -> Self {    
        let bit_strings = HashMap::new();
        let byte_strings = HashMap::new();
        let hex_strings = HashMap::new();
        let identifiers = HashMap::new();
        let identifier_escapes = vec!["\"".to_string()];
        let quotes = HashMap::new();
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
            // ... add other field assignments
            numeric_literals: HashMap::new(),
            identifier_can_start_with_digit: false,
            /// State management
            sql: String::new(),
            size: 0,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            col: 1,
            comments: Vec::new(),
            char: '\0',
            end: false,
            peek: '\0',
            prev_token_line: 0,
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
        self.col = 0;
        self.end = false;
        self.prev_token_line = 0;
        self.prev_token_comments.clear();
        self.prev_token_type = None;
        // Pre-load all the things we need to tokenize
        // self.advance(1);
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
        self.prev_token_line = 0;
        self.prev_token_comments.clear();
        self.prev_token_type = None;
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
        self.char = self.sql.chars().nth(self.current-1).unwrap_or('\0');
        if self.end {
            self.peek = '\0';
        } else {
            self.peek = self.sql.chars().nth(self.current).unwrap_or('\0');
        }
    }

    /// This function retreats through the characters in the SQL string. It updates
    /// the state of the tokenizer struct.
    fn retreat(&mut self, i: usize) {
        // We should ensure that we don't retreat beyond the start of the input string
        if i > self.current {
            panic!("Cannot retreat beyond the start of the input string");
        }

        self.current -= i;
        self.end = self.current >= self.size;

        self.char = self.sql.chars().nth(self.current - 1).unwrap_or('\0');
        if self.end {
            self.peek = '\0';
        } else {
            self.peek = self.sql.chars().nth(self.current).unwrap_or('\0');
        }

        // We don't adjust the line and column positions in this function,
        // as it would require scanning backward to find previous line breaks.
        // If we need accurate line and column information when retreating,
        // we may need to implement additional logic to handle that.
    }

    /// Returns a list of tokens corresponding to the SQL string `sql`.
    pub fn tokenize(&mut self, sql: &str) -> Vec<Token> {
        self.reset();
        self.add_sql(sql.to_string());

        self.scan();

        self.tokens.clone()
    }

    /// This function scans the current character
    pub fn scan(&mut self) {
        while self.size > 0 && !self.end {
            self.start = self.current;
            self.advance(1);

            let current_char = self.char;

            if current_char == '\0' {
                break;
            }

            if let Some(token_type) = self.get_token_type_for_char(current_char) {
                match token_type.as_str() {
                    "Number" => self.scan_number(),
                    id => self.scan_identifier(id),
                };
            } else if !self.white_space.contains_key(&current_char.to_string()) {
                self.scan_keywords();
            }
        }

        if let Some(last_token) = self.tokens.last_mut() {
            last_token.comments.append(&mut self.comments);
        }
    }


    /// The `add_token` function appends a new token to the tokens list, using the given token type
    /// and an optional text. If the text is not provided, the function uses the tokenizer's
    /// current text. The function also updates the previous token's properties.
    fn add_token(&mut self, token_type: TokenType, text: Option<String>) {
        self.prev_token_line = self.line;
        self.prev_token_comments = self.comments.clone();
        self.prev_token_type = Some(token_type);

        let token_text = text.unwrap_or_else(|| self.get_text().to_string());
        let token_len = token_text.len();
        let token = Token {
            token_type,
            text: token_text,
            line: self.line,
            col: self.col,
            start: if self.current >= token_len { self.current - token_len } else { 0 },
            end: self.current,
            comments: self.comments.clone(),
        };

        self.tokens.push(token);
        self.comments.clear();
    }

    ///////////
    // STRING OPERATIONS 
    //////////

    fn chars(&mut self, size: usize) -> &str {
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

    /// The `text` method returns a slice of the SQL string from the start to 
    /// the current position.
    fn get_text(&self) -> &str {
        &self.sql[self.start..self.current]
    }

    /////////////
    // EXTRACTING OPERATIONS
    /////////////

    /// This function handles the logic of identifying the token type of the 
    /// current character, without modifying the Tokenizer struct. We can use 
    /// this new function in the scan function and pass the appropriate token 
    /// type to the corresponding scan functions.
    fn get_token_type_for_char(&self, ch: char) -> Option<String> {
        if ch.is_digit(10) {
            Some("Number".to_string())
        } else if let Some(identifier_value) = self.identifiers.get(&ch.to_string()) {
            Some(identifier_value.clone())
        } else {
            None
        }
    }

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
            // Check if the character is not a null character and not a key in single_tokens
            if self.peek != '\0' 
                && !self.single_tokens.contains_key(&self.peek.to_string()) 
                && !self.peek.is_whitespace() 
            {
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
    

    /// This function iterates through the characters in the input string to 
    /// form the word, then it checks if the word is in the keywords HashMap or 
    /// if the single character is in single_tokens. If it finds a match, it 
    /// adds the corresponding token to the list of tokens and updates the 
    /// position in the input string. If no keyword or single token is found, 
    /// it calls scan_var() to continue the tokenization process.
    fn scan_keywords(&mut self) -> bool {
        let mut size = 0;
        let mut word = None;
        let mut chars = self.get_text().to_string();
        let mut char = chars.clone();
        let mut prev_space = false;
        let mut skip = false;
        let mut single_token = self.single_tokens.contains_key(&char);
        
        while !chars.is_empty() {
            if skip {
                size += 1;
            } else {
                if let Some(token_type) = self.keywords.get(&char.to_uppercase()) {
                    word = Some(chars.clone());
                } else {
                    break;
                }
            }
    
            size += 1;
            let end = self.current - 1 + size;
    
            if end < self.size {
                char = self.sql.chars().nth(end).unwrap().to_string();
                single_token = single_token || self.single_tokens.contains_key(&char);
                let is_space = self.white_space.contains_key(&char);
    
                if !is_space || !prev_space {
                    if is_space {
                        char = " ".to_string();
                    }
                    chars.push_str(&char);
                    prev_space = is_space;
                    skip = false;
                } else {
                    skip = true;
                }
            } else {
                chars = " ".to_string();
            }
        }
    
        word = if single_token || !self.white_space.contains_key(&chars.chars().last().unwrap().to_string()) {
            None
        } else {
            word
        };
    
        if let Some(w) = word {
            if self.scan_string(&w) {
                return true;
            }
            if self.scan_formatted_string(&w) {
                return true;
            }
            if self.scan_comment(&w) {
                return true;
            }
    
            self.advance(size - 1);
            let w = w.to_uppercase();
            if let Some(token_type) = self.keywords.get(&w) {
                self.add_token(token_type.clone(), Some(w));
                return true;
            }
        } else {
            if let Some(token_type) = self.single_tokens.get(&self.char.to_string()) {
                self.add_token(token_type.clone(), Some(self.char.to_string()));
                return true;
            }
            self.scan_var();
            return true;
        }
    
        false
    }
    

    /// This function scans comments in the SQL string. It detects comments
    /// and appends them to the appropriate lists (comments, prev_token_comments).
    fn scan_comment(&mut self, comment_start: &str) -> bool {

        if !self.comment_tokens.contains_key(comment_start) {
            return false;
        }
    
        let comment_start_line = self.line;
        let comment_start_size = comment_start.len();
        let comment_end = match self.comment_tokens.get(&comment_start.to_string()) {
            Some(val) => val.clone().unwrap_or("".to_string()),
            None => {
                // Handle the case where comment_start is not found in comment_tokens
                return false;
            }
        };

        if !comment_end.is_empty() {
            self.advance(comment_start_size);
            let comment_end_size = comment_end.len();
    
            let mut current_chars = self.chars(comment_end_size);

            loop {
                if current_chars == comment_end {
                    break;
                }
                self.advance(1);
                if self.end {
                    break;
                }
                current_chars = self.chars(comment_end_size);
            }

            self.comments.push(self.get_text()[comment_start_size..self.current - comment_end_size + 1].to_string());
            self.advance(comment_end_size - 1);
        } else {
            while !self.end && !(self.white_space.get(&self.peek.to_string()) == Some(&TokenType::Break)) {
                self.advance(1);
            }
            self.comments.push(self.get_text()[comment_start_size..].to_string());
        }
    
        if comment_start_line == self.prev_token_line {
            self.prev_token_comments.extend_from_slice(&self.comments);
            self.comments.clear();
            self.prev_token_line = self.line;
        }
    
        true
    }

    /// This function takes a quote parameter and checks if it's a valid quote 
    /// start using _QUOTES. If it's not a valid quote, it returns False. Otherwise,
    /// it advances the tokenizer, extracts the string content until the quote end,
    /// and then adds a new token with the TokenType.NATIONAL or TokenType.STRING 
    /// type, depending on the quote type. Finally, it returns True to indicate 
    /// that a string has been scanned successfully.
    fn scan_string(&mut self, quote: &str) -> bool {

        // We use a block here to limit the scope of the immutable borrow.
        let (quote_end, quote_len) = {
            let quote_end = self.quotes.get(quote).map_or_else(|| quote.to_string(), |s| s.clone());
            let quote_len = quote.len();
            (quote_end, quote_len)
        };
    
        self.advance(quote_len);
        let result = self.extract_string(quote_end.as_str());
        match result {
            Ok(text) => {
                let token_type = if quote.chars().next().unwrap().is_uppercase() {
                    TokenType::National
                } else {
                    TokenType::String
                };
                self.add_token(token_type, Some(text));
                true
            }
            Err(_) => false,
        }
    }

    /// This function processes formatted strings such as hexadecimal strings, 
    /// bit strings, and byte strings. It checks if the string matches any of 
    /// the formats, extracts the string content, and then adds a token with 
    /// the appropriate type.
    fn scan_formatted_string(&mut self, string_start: &str) -> bool {

        let (delimiters, token_type, base) = if self.hex_strings.contains_key(string_start) {
            (&self.hex_strings, TokenType::HexString, Some(16))
        } else if self.bit_strings.contains_key(string_start) {
            (&self.bit_strings, TokenType::BitString, Some(2))
        } else if self.byte_strings.contains_key(string_start) {
            (&self.byte_strings, TokenType::ByteString, None)
        } else {
            return false;
        };
    
        let string_end = delimiters.get(string_start).cloned().unwrap_or_else(|| string_start.to_string());
        let string_start_len = string_start.len();

        self.advance(string_start_len);

        let text = self.extract_string(&string_end).unwrap();

        let final_text = if let Some(base) = base {
            match i64::from_str_radix(&text, base) {
                Ok(value) => value.to_string(),
                Err(_) => {
                    panic!(
                        "Numeric string contains invalid characters from {}:{}",
                        self.line, self.start
                    )
                }
            }
        } else {
            text
        };
    
        self.add_token(token_type, Some(final_text));
        true
    }    
    
    
    /// This function accepts an identifier_end parameter and processes the 
    /// input SQL accordingly. It builds an identifier token, handling 
    /// escape characters if needed, and adds it to the list of tokens.
    fn scan_identifier(&mut self, identifier_end: &str) -> bool {
        let mut text = String::new();
        let identifier_end_is_escape = self.identifier_escapes.contains(&identifier_end.to_string());
    
        loop {
            if self.end {
                panic!(
                    "Missing {} from {}:{}",
                    identifier_end, self.line, self.start
                );
            }
    
            self.advance(1);
    
            if self.char.to_string() == identifier_end {
                if identifier_end_is_escape && self.peek.to_string() == identifier_end {
                    text.push_str(identifier_end);
                    self.advance(1);
                    continue;
                }
    
                break;
            }
    
            text.push(self.char);
        }
    
        self.add_token(TokenType::Identifier, Some(text));
        true
    }
    
    

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
            let text_upper = self.get_text().to_uppercase();
            self.keywords.get(&text_upper).cloned().unwrap_or(TokenType::Var)
        };

        self.add_token(token_type, None);
    }

    /// This function scans a hex string and adds it as a token. The function 
    /// advances the tokenizer by one character, extracts the value of the hex 
    /// string, and attempts to convert the value to an integer using base 16. 
    /// If successful, the function adds a HEX_STRING token with the converted 
    /// value. If the conversion fails, it adds an IDENTIFIER token instead.
    fn scan_hex(&mut self) -> bool {
        self.advance(1);
        let value = self.extract_value();
    
        match i64::from_str_radix(&value, 16) {
            Ok(value) => {
                self.add_token(TokenType::HexString, Some(value.to_string()));
                true
            }
            Err(_) => {
                self.add_token(TokenType::Identifier, None);
                false
            }
        }
    }

    /// This function attempts to parse a binary string, advancing one character
    /// to skip the initial b or B character. Then, it extracts the value by 
    /// calling extract_value. It tries to convert the value to an integer using 
    /// base 2. If successful, it adds a TokenType::BitString token to the 
    /// tokens list with the integer value as its text. If the conversion fails, 
    /// it adds a TokenType::Identifier token instead.
    fn scan_bits(&mut self) -> bool {
        self.advance(1);
        let value = self.extract_value();
        match i64::from_str_radix(&value, 2) {
            Ok(parsed_value) => {
                self.add_token(TokenType::BitString, Some(parsed_value.to_string()));
                true
            }
            Err(_) => {
                self.add_token(TokenType::Identifier, None);
                false
            }
        }
    }

    // TODO: Fix - this one is

    /// This function attempts to parse a number. If the current character is '0',
    /// it checks if the next character is 'B' or 'X' for binary or hexadecimal 
    /// numbers, respectively, and calls the appropriate function. It then parses
    /// decimal and scientific notation numbers. If the number is followed by an 
    /// identifier, it adds the tokens accordingly, otherwise, it adds a 
    /// TokenType::Number token.
    fn scan_number(&mut self) -> bool {
        if self.char == '0' {
            let peek = self.peek.to_uppercase().to_string();
            if peek == "B" {
                return self.scan_bits();
            } else if peek == "X" {
                return self.scan_hex();
            }
        }
    
        let mut decimal = false;
        let mut scientific = 0;
    
        loop {
            match self.peek {
                c if c.is_digit(10) => {
                    self.advance(1)
                },
                '.' if !decimal => {
                    decimal = true;
                    self.advance(1);
                }
                '-' | '+' if scientific == 1 => {
                    scientific += 1;
                    self.advance(1);
                }
                c if c.to_uppercase().to_string() == "E" && scientific == 0 => {
                    scientific += 1;
                    self.advance(1);
                }
                c if c.is_alphanumeric() || c == '_' => {
                    let number_text = self.get_text().to_string();
                    let mut literal = String::new();
                    while !self.peek.is_whitespace() && !self.single_tokens.contains_key(&self.peek.to_string()) {
                        literal.push(self.peek.to_uppercase().next().unwrap());
                        self.advance(1);
                    }
                    let token_type = self
                        .numeric_literals
                        .get(&literal)
                        .and_then(|k| self.keywords.get(k).cloned());
                    if let Some(token_type) = token_type {
                        self.add_token(TokenType::Number, Some(number_text));
                        self.add_token(TokenType::DColon, Some("::".to_string()));
                        self.add_token(token_type.clone(), Some(literal));
                    } else if self.identifier_can_start_with_digit {
                        self.add_token(TokenType::Var, None);
                    } else {
                        self.add_token(TokenType::Number, Some(number_text));
                    }
                    // self.retreat(literal.len() as i64);
                }
                _ => {
                    let number_text = self.get_text().to_string();
                    self.add_token(TokenType::Number, Some(number_text));
                    break;
                },
            }
        }
    
        true
    }
    
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


    #[test]
    fn test_retreat() {
        let sql = "SELECT * FROM table";
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql(sql.to_string());

        // Advance 5 positions
        tokenizer.advance(5);
        assert_eq!(tokenizer.current, 6);
        assert_eq!(tokenizer.char, 'T');
        assert_eq!(tokenizer.peek, ' ');

        // Retreat 3 positions
        tokenizer.retreat(3);
        assert_eq!(tokenizer.current, 3);
        assert_eq!(tokenizer.char, 'L');
        assert_eq!(tokenizer.peek, 'E');

        // Retreat to the start of the input string
        tokenizer.retreat(2);
        assert_eq!(tokenizer.current, 1);
        assert_eq!(tokenizer.char, 'S');
        assert_eq!(tokenizer.peek, 'E');
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
        let mut tokenizer: Tokenizer = Tokenizer::new();

        let sql = "SELECT * FROM table;".to_string();
        tokenizer.add_sql(sql);

        assert_eq!(tokenizer.sql, "SELECT * FROM table;");
        assert_eq!(tokenizer.size, 20);
        assert_eq!(tokenizer.char, 'S');
        assert_eq!(tokenizer.peek, 'E');
        assert_eq!(tokenizer.start, 0);
        assert_eq!(tokenizer.current, 1);
        assert_eq!(tokenizer.line, 1);
        assert_eq!(tokenizer.col, 1);
        assert_eq!(tokenizer.end, false);
        assert_eq!(tokenizer.prev_token_line, 0);
        assert!(tokenizer.prev_token_comments.is_empty());
        assert_eq!(tokenizer.prev_token_type, None);
    }

    #[test]
    fn test_get_text() {
        let mut tokenizer: Tokenizer = Tokenizer::new();
        let sql = "SELECT * FROM table;".to_string();
        tokenizer.add_sql(sql);
        tokenizer.advance(5);
        assert_eq!(tokenizer.get_text(), "SELECT");
        tokenizer.advance(1);
        assert_eq!(tokenizer.get_text(), "SELECT ");
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
        tokenizer.advance(34);
        let extracted_string = tokenizer.extract_string(delimiter).unwrap();
        assert_eq!(extracted_string, "John O Connor");
    }

    #[test]
    fn test_extract_value() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM table WHERE value=42".to_string());
        tokenizer.advance(31); // Move the tokenizer to the position right before the value 42

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

    /// This test creates a Tokenizer instance, adds an SQL query to it, and 
    /// assumes that the tokenizer is at the position of the keyword "SELECT". 
    /// It then calls the scan_var function to tokenize the keyword and checks 
    /// if the token was added correctly by comparing the token's properties 
    /// with the expected values.
    #[test]
    fn test_scan_var() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM table;".to_string());

        // Assuming that the tokenizer is at the position of the keyword "SELECT"
        tokenizer.scan_var();

        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::Select);
        assert_eq!(tokenizer.tokens[0].text, "SELECT");

        tokenizer.advance(9);

        tokenizer.scan_var();
    }

    #[test]
    fn test_scan_identifier() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT * FROM database.schema.table".to_string());

        tokenizer.advance(13);

        tokenizer.scan_identifier(".");

        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokenizer.tokens[0].text, "database");
        assert_eq!(tokenizer.tokens[0].line, 1);
        assert_eq!(tokenizer.tokens[0].col, 23);
        assert_eq!(tokenizer.tokens[0].start, 15);
        assert_eq!(tokenizer.tokens[0].end, 23);

        tokenizer.scan_identifier(".");

        assert_eq!(tokenizer.tokens.len(), 2);
        assert_eq!(tokenizer.tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokenizer.tokens[1].text, "schema");
        assert_eq!(tokenizer.tokens[1].line, 1);
        assert_eq!(tokenizer.tokens[1].col, 30);
        assert_eq!(tokenizer.tokens[1].start, 24);
        assert_eq!(tokenizer.tokens[1].end, 30);

    }

    /// This unit test adds a SQL string with a single-quoted string and then 
    /// calls scan_string to ensure that the string is scanned correctly and that
    /// the result returns true.
    #[test]
    fn test_scan_string() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("SELECT 'Hello, World!'".to_string());
        tokenizer.advance(7);

        let result = tokenizer.scan_string("'");
        assert!(result);

        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::String);
        assert_eq!(tokenizer.tokens[0].text, "Hello, World!");
        assert_eq!(tokenizer.tokens[0].line, 1);
        assert_eq!(tokenizer.tokens[0].col, 22);
        assert_eq!(tokenizer.tokens[0].start, 9);
        assert_eq!(tokenizer.tokens[0].end, 22);
    }

    // TODO: Fix this as it is broken
    // I believe it is because the tokenizer is not recognizing the first "'" as 
    // being part of the string.
    // This implementation converts the formatted string to the appropriate type
    // and adds a token based on the extracted content. The unit test verifies 
    // the function for different formatted string types.
    #[test]
    fn test_scan_formatted_string() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.bit_strings.insert("b".to_string(), "'".to_string());
        tokenizer.byte_strings.insert("E".to_string(), "'".to_string());
        tokenizer.hex_strings.insert("X".to_string(), "'".to_string());

        tokenizer.tokenize("X'1A2B' b'1100' E'\\\\\\''");

        // assert!(tokenizer.scan_formatted_string("X"));
        // assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::HexString);
        assert_eq!(tokenizer.tokens[0].text, "6699");

        tokenizer.advance(4);

        assert!(tokenizer.scan_formatted_string("b"));
        assert_eq!(tokenizer.tokens.len(), 2);
        assert_eq!(tokenizer.tokens[1].token_type, TokenType::BitString);
        assert_eq!(tokenizer.tokens[1].text, "12");

        tokenizer.advance(4);

        assert!(tokenizer.scan_formatted_string("E"));
        assert_eq!(tokenizer.tokens.len(), 3);
        assert_eq!(tokenizer.tokens[2].token_type, TokenType::String);
        assert_eq!(tokenizer.tokens[2].text, "\\\\\\'");
    }

    /// This test checks whether the scan_hex function correctly identifies and 
    /// processes valid and invalid hex strings. The test adds a valid hex string 
    /// 0x1A2B and an invalid hex string 0xInvalid to the tokenizer. It checks 
    /// if the function adds a HEX_STRING token for the valid hex string and an 
    /// IDENTIFIER token for the invalid one.
    #[test]
    fn test_scan_hex() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("0x1A2B 0xInvalid".to_string());

        tokenizer.scan_hex();
        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::HexString);
        assert_eq!(tokenizer.tokens[0].text, "6699");

        tokenizer.advance(5);

        tokenizer.scan_hex();
        assert_eq!(tokenizer.tokens.len(), 2);
        assert_eq!(tokenizer.tokens[1].token_type, TokenType::Identifier);
    }

    /// This test case checks that the scan_bits function can successfully parse
    /// a valid binary string and also handles the case where the binary string 
    /// contains invalid characters.
    #[test]
    fn test_scan_bits() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("b'1010' b'invalid'".to_string());
    
        assert!(tokenizer.scan_bits());
        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::BitString);
        assert_eq!(tokenizer.tokens[0].text, "10");
    
        tokenizer.advance(5);
    
        assert!(!tokenizer.scan_bits());
        assert_eq!(tokenizer.tokens.len(), 2);
        assert_eq!(tokenizer.tokens[1].token_type, TokenType::Identifier);
    }
    
    // TODO: Fix this once I've got scan working
    /// This test checks various types of number inputs, including integers, 
    /// decimals, scientific notation, and numbers with numeric literals.
    #[test]
    fn test_scan_number() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql("1234 56.78 9.0e+1 0xEFF 0b1011 12::integer".to_string());
        // tokenizer.scan();
        dbg!(&tokenizer.tokens);

        assert!(tokenizer.scan_number());
        assert_eq!(tokenizer.tokens.len(), 1);
        assert_eq!(tokenizer.tokens[0].token_type, TokenType::Number);
        assert_eq!(tokenizer.tokens[0].text, "1234");

    }
    

    // THIS IS BROKEN BECAUSE START IS NOT UPDATING.
    // COME CHECK AGAIN WHEN WE INTRODUCE SCAN.

    /// This test checks whether the scan_comment function can correctly identify 
    /// and process single line and multiline comments in the given SQL string.
    /// We first advance the tokenizer to the position where the comments are 
    /// expected and then call scan_comment with the appropriate comment_start 
    /// value. The test then checks if the function returns true and if the 
    /// prev_token_comments vector contains the expected comments.
    #[test]
    fn test_scan_comment() {
        let sql = "SELECT * FROM users -- This is a single line comment
                   WHERE id = 42; /* This is a
                   multiline comment */";
        let mut tokenizer = Tokenizer::new();
        tokenizer.tokenize(sql);
        dbg!(&tokenizer);
        
        // Check for single line comment
        tokenizer.advance(20);
        assert_eq!(tokenizer.char, '-');
        assert_eq!(tokenizer.peek, '-');
        let comment_start = tokenizer.char.to_string() + &tokenizer.peek.to_string();
        let is_comment = tokenizer.scan_comment(comment_start.as_str());
        assert!(is_comment);
        assert_eq!(tokenizer.prev_token_comments, vec!["This is a single line comment"]);

        // Check for multiline comment
        tokenizer.advance(37);
        assert_eq!(tokenizer.char, '/');
        assert_eq!(tokenizer.peek, '*');
        let comment_start = tokenizer.char.to_string() + &tokenizer.peek.to_string();
        let is_comment = tokenizer.scan_comment(comment_start.as_str());
        assert!(is_comment);
        assert_eq!(tokenizer.prev_token_comments, vec!["This is a single line comment", "This is a\nmultiline comment"]);
    }

    #[test]
    fn test_scan_keywords() {
        let mut tokenizer: Tokenizer = Tokenizer::new();
        tokenizer.tokenize("SELECT * FROM users WHERE age >= 18 AND is_active = 1;");

        assert_eq!(tokenizer.tokens[0].token_type, TokenType::Select);
        assert_eq!(tokenizer.tokens[0].text, "SELECT");

        assert_eq!(tokenizer.tokens[1].token_type, TokenType::Star);
        assert_eq!(tokenizer.tokens[1].text, "*");

        assert_eq!(tokenizer.tokens[2].token_type, TokenType::From);
        assert_eq!(tokenizer.tokens[2].text, "FROM");

        assert_eq!(tokenizer.tokens[3].token_type, TokenType::Var);
        assert_eq!(tokenizer.tokens[3].text, "users");

        assert_eq!(tokenizer.tokens[4].token_type, TokenType::Where);
        assert_eq!(tokenizer.tokens[4].text, "WHERE");

        // ...continue testing the rest of the keywords in the input SQL
    }

    #[test]
    fn test_scan() {
        let sql = "SELECT * FROM users WHERE id = 42;";
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_sql(sql.to_string());

        tokenizer.scan();
        assert_eq!(
            tokenizer.tokens[0], 
            Token {
                token_type: TokenType::Select,
                text: "SELECT".to_string(),
                comments: Vec::new(),
                line: 1,
                col: 6,
                start: 0,
                end: 6,
            }
        );

        assert_eq!(
            tokenizer.tokens[1], 
            Token {
                token_type: TokenType::Star,
                text: "*".to_string(),
                comments: Vec::new(),
                line:1,
                col:8,
                start:7,
                end:8,
            },
        );

        assert_eq!(
            tokenizer.tokens[2], 
            Token {
                token_type: TokenType::From,
                text: "FROM".to_string(),
                comments: Vec::new(),
                line: 1,
                col:13,
                start:9,
                end:13,
            },
        );

        assert_eq!(
            tokenizer.tokens[3], 
            Token {
                token_type: TokenType::Var,
                text: "users".to_string(),
                comments: Vec::new(),
                line:1,
                col:19,
                start:14,
                end:19,
            },
        );

        assert_eq!(
            tokenizer.tokens[4], 
            Token {
                token_type: TokenType::Where,
                text: "WHERE".to_string(),
                comments: Vec::new(),
                line:1,
                col:25,
                start:20,
                end:25,
            },
        );

        assert_eq!(
            tokenizer.tokens[5], 
            Token {
                token_type: TokenType::Var,
                text: "id".to_string(),
                comments: Vec::new(),
                line:1,
                col:28,
                start:26,
                end:28,
            },
        );

        assert_eq!(
            tokenizer.tokens[6], 
            Token {
                token_type: TokenType::Eq,
                text: "=".to_string(),
                comments: Vec::new(),
                line:1,
                col:30,
                start:29,
                end:30,
            },
        );

        assert_eq!(
            tokenizer.tokens[7], 
            Token {
                token_type: TokenType::Number,
                text: "42".to_string(),
                comments: Vec::new(),
                line:1,
                col:33,
                start:31,
                end:33,
            },
        );

        assert_eq!(
            tokenizer.tokens[8], 
            Token {
                token_type: TokenType::Semicolon,
                text: ";".to_string(),
                comments: Vec::new(),
                line:1,
                col:34,
                start:33,
                end:34,
            },
        );

    }

}