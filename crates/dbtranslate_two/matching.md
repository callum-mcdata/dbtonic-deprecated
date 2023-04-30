I am currently matching the glot 11.6.2 version.


 // fn scan_number(&mut self) {
    //     if self.char.unwrap() == '0' {
    //         let peek = self.peek().unwrap().to_uppercase();
    //         if peek == "B" {
    //             return self.scan_bits();
    //         } else if peek == "X" {
    //             return self.scan_hex();
    //         }
    //     }

    //     let mut decimal = false;
    //     let mut scientific = 0;

    //     loop {
    //         if self.peek().unwrap().is_digit(10) {
    //             self.advance();
    //         } else if self.peek().unwrap() == '.' && !decimal {
    //             decimal = true;
    //             self.advance();
    //         } else if self.peek().unwrap() == '-' && scientific == 1
    //             || self.peek().unwrap() == '+' && scientific == 1
    //         {
    //             scientific += 1;
    //             self.advance();
    //         } else if self.peek().unwrap().to_uppercase() == "E" && scientific == 0 {
    //             scientific += 1;
    //             self.advance();
    //         } else if self.peek().unwrap().is_alphabetic() {
    //             let number_text = self.text();
    //             let mut literal = String::new();

    //             while self.peek().unwrap().is_whitespace() && !self.single_tokens.contains(&self.peek().unwrap().to_string()) {
    //                 literal.push(self.peek().unwrap().to_uppercase().next().unwrap());
    //                 self.advance();
    //             }

    //             let token_type = self.keywords.get(&self.numeric_literals.get(&literal).unwrap().clone());

    //             if let Some(token_type) = token_type {
    //                 self.add(TokenType::Number, number_text);
    //                 self.add(TokenType::Dcolon, "::");
    //                 return self.add(token_type.clone(), literal);
    //             } else if self.identifier_can_start_with_digit {
    //                 return self.add(TokenType::Var, String::new());
    //             }

    //             self.add(TokenType::Number, number_text);
    //             return self.advance_back(literal.len());
    //         } else {
    //             return self.add(TokenType::Number, String::new());
    //         }
    //     }
    // }


    // SCANNING OPERATIONS

    // This function processes the SQL string and generates tokens based on the 
    // characters it encounters. It takes the parameter until, which is 
    // a closure that returns a boolean. When the until function returns true, 
    // the scanning process stops.
    // The scan function iterates through the SQL string, starting at the current 
    // position, and advances through it one character at a time. If a character 
    // is not white space, it checks if it's a digit, an identifier, or a keyword, 
    // and calls the corresponding functions to handle each case 
    // (scan_number, scan_identifier, and scan_keywords).
    // If the until function is provided and returns true, the loop breaks, and
    //  scanning stops. Before returning, the function appends any comments 
    // collected during the scanning process to the last token in the tokens vector.
    // fn scan(&mut self, until: Option<Box<dyn Fn() -> bool>>) {
    //     while self.size != 0 && !self.end {
    //         self.start = self.current;
    //         self.advance();

    //         if let Some(ch) = self.char {
    //             if !self.white_space.contains_key(&ch.to_string()) {
    //                 if ch.is_digit(10) {
    //                     self.scan_number();
    //                 } else if let Some(token_type) = self.identifiers.get(&ch.to_string()) {
    //                     self.scan_identifier(token_type.clone());
    //                 } else {
    //                     self.scan_keywords();
    //                 }
    //             }
    //         } else {
    //             break;
    //         }

    //         if let Some(until_fn) = &until {
    //             if until_fn() {
    //                 break;
    //             }
    //         }
    //     }

    //     if let Some(last_token) = self.tokens.last_mut() {
    //         last_token.comments.extend(self.comments.clone());
    //     }
    // }

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