// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! SQL Parser

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt;

use log::debug;

use IsLateral::*;
use IsOptional::*;

use crate::ast::*;
use crate::dialect::*;
use crate::keywords::{self, Keyword};
use crate::tokenizer::*;
use std::collections::HashMap;
use crate::parser::query::{DbtConfigValue,DbtConfig, JinjaVariable, JinjaValue};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    TokenizerError(String),
    ParserError(String),
    RecursionLimitExceeded,
}

// Use `Parser::expected` instead, if possible
macro_rules! parser_err {
    ($MSG:expr) => {
        Err(ParserError::ParserError($MSG.to_string()))
    };
}

// Returns a successful result if the optional expression is some
macro_rules! return_ok_if_some {
    ($e:expr) => {{
        if let Some(v) = $e {
            return Ok(v);
        }
    }};
}

#[cfg(feature = "std")]
/// Implementation [`RecursionCounter`] if std is available
mod recursion {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::rc::Rc;

    use super::ParserError;

    /// Tracks remaining recursion depth. This value is decremented on
    /// each call to `try_decrease()`, when it reaches 0 an error will
    /// be returned.
    ///
    /// Note: Uses an Rc and AtomicUsize in order to satisfy the Rust
    /// borrow checker so the automatic DepthGuard decrement a
    /// reference to the counter. The actual value is not modified
    /// concurrently
    pub(crate) struct RecursionCounter {
        remaining_depth: Rc<AtomicUsize>,
    }

    impl RecursionCounter {
        /// Creates a [`RecursionCounter`] with the specified maximum
        /// depth
        pub fn new(remaining_depth: usize) -> Self {
            Self {
                remaining_depth: Rc::new(remaining_depth.into()),
            }
        }

        /// Decreases the remaining depth by 1.
        ///
        /// Returns `Err` if the remaining depth falls to 0.
        ///
        /// Returns a [`DepthGuard`] which will adds 1 to the
        /// remaining depth upon drop;
        pub fn try_decrease(&self) -> Result<DepthGuard, ParserError> {
            let old_value = self.remaining_depth.fetch_sub(1, Ordering::SeqCst);
            // ran out of space
            if old_value == 0 {
                Err(ParserError::RecursionLimitExceeded)
            } else {
                Ok(DepthGuard::new(Rc::clone(&self.remaining_depth)))
            }
        }
    }

    /// Guard that increass the remaining depth by 1 on drop
    pub struct DepthGuard {
        remaining_depth: Rc<AtomicUsize>,
    }

    impl DepthGuard {
        fn new(remaining_depth: Rc<AtomicUsize>) -> Self {
            Self { remaining_depth }
        }
    }
    impl Drop for DepthGuard {
        fn drop(&mut self) {
            self.remaining_depth.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[cfg(not(feature = "std"))]
mod recursion {
    /// Implementation [`RecursionCounter`] if std is NOT available (and does not
    /// guard against stack overflow).
    ///
    /// Has the same API as the std RecursionCounter implementation
    /// but does not actually limit stack depth.
    pub(crate) struct RecursionCounter {}

    impl RecursionCounter {
        pub fn new(_remaining_depth: usize) -> Self {
            Self {}
        }
        pub fn try_decrease(&self) -> Result<DepthGuard, super::ParserError> {
            Ok(DepthGuard {})
        }
    }

    pub struct DepthGuard {}
}

use recursion::RecursionCounter;

#[derive(PartialEq, Eq)]
pub enum IsOptional {
    Optional,
    Mandatory,
}

pub enum IsLateral {
    Lateral,
    NotLateral,
}

pub enum WildcardExpr {
    Expr(Expr),
    QualifiedWildcard(ObjectName),
    Wildcard,
}

impl From<WildcardExpr> for FunctionArgExpr {
    fn from(wildcard_expr: WildcardExpr) -> Self {
        match wildcard_expr {
            WildcardExpr::Expr(expr) => Self::Expr(expr),
            WildcardExpr::QualifiedWildcard(prefix) => Self::QualifiedWildcard(prefix),
            WildcardExpr::Wildcard => Self::Wildcard,
        }
    }
}

impl From<TokenizerError> for ParserError {
    fn from(e: TokenizerError) -> Self {
        ParserError::TokenizerError(e.to_string())
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sql parser error: {}",
            match self {
                ParserError::TokenizerError(s) => s,
                ParserError::ParserError(s) => s,
                ParserError::RecursionLimitExceeded => "recursion limit exceeded",
            }
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParserError {}

// By default, allow expressions up to this deep before erroring
const DEFAULT_REMAINING_DEPTH: usize = 50;

#[derive(Default)]
pub struct ParserOptions {
    pub trailing_commas: bool,
}

pub struct Parser<'a> {
    tokens: Vec<TokenWithLocation>,
    /// The index of the first unprocessed token in `self.tokens`
    index: usize,
    /// The current dialect to use
    dialect: &'a dyn Dialect,
    /// Additional options that allow you to mix & match behavior otherwise
    /// constrained to certain dialects (e.g. trailing commas)
    options: ParserOptions,
    /// ensure the stack does not overflow by limiting recursion depth
    recursion_counter: RecursionCounter,
}

impl<'a> Parser<'a> {
    /// Create a parser for a [`Dialect`]
    ///
    /// See also [`Parser::parse_sql`]
    ///
    /// Example:
    /// ```
    /// # use dbtranslate::{parser::{Parser, ParserError}, dialect::GenericDialect};
    /// # fn main() -> Result<(), ParserError> {
    /// let dialect = GenericDialect{};
    /// let statements = Parser::new(&dialect)
    ///   .try_with_sql("SELECT * FROM foo")?
    ///   .parse_statements()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(dialect: &'a dyn Dialect) -> Self {
        Self {
            tokens: vec![],
            index: 0,
            dialect,
            recursion_counter: RecursionCounter::new(DEFAULT_REMAINING_DEPTH),
            options: ParserOptions::default(),
        }
    }

    /// Specify the maximum recursion limit while parsing.
    ///
    ///
    /// [`Parser`] prevents stack overflows by returning
    /// [`ParserError::RecursionLimitExceeded`] if the parser exceeds
    /// this depth while processing the query.
    ///
    /// Example:
    /// ```
    /// # use dbtranslate::{parser::{Parser, ParserError}, dialect::GenericDialect};
    /// # fn main() -> Result<(), ParserError> {
    /// let dialect = GenericDialect{};
    /// let result = Parser::new(&dialect)
    ///   .with_recursion_limit(1)
    ///   .try_with_sql("SELECT * FROM foo WHERE (a OR (b OR (c OR d)))")?
    ///   .parse_statements();
    ///   assert_eq!(result, Err(ParserError::RecursionLimitExceeded));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_recursion_limit(mut self, recursion_limit: usize) -> Self {
        self.recursion_counter = RecursionCounter::new(recursion_limit);
        self
    }

    /// Specify additional parser options
    ///
    ///
    /// [`Parser`] supports additional options ([`ParserOptions`]) that allow you to
    /// mix & match behavior otherwise constrained to certain dialects (e.g. trailing
    /// commas).
    ///
    /// Example:
    /// ```
    /// # use dbtranslate::{parser::{Parser, ParserError, ParserOptions}, dialect::GenericDialect};
    /// # fn main() -> Result<(), ParserError> {
    /// let dialect = GenericDialect{};
    /// let result = Parser::new(&dialect)
    ///   .with_options(ParserOptions { trailing_commas: true })
    ///   .try_with_sql("SELECT a, b, COUNT(*), FROM foo GROUP BY a, b,")?
    ///   .parse_statements();
    ///   assert!(matches!(result, Ok(_)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_options(mut self, options: ParserOptions) -> Self {
        self.options = options;
        self
    }

    /// Reset this parser to parse the specified token stream
    pub fn with_tokens_with_locations(mut self, tokens: Vec<TokenWithLocation>) -> Self {
        self.tokens = tokens;
        self.index = 0;
        self
    }

    /// Reset this parser state to parse the specified tokens
    pub fn with_tokens(self, tokens: Vec<Token>) -> Self {
        // Put in dummy locations
        let tokens_with_locations: Vec<TokenWithLocation> = tokens
            .into_iter()
            .map(|token| TokenWithLocation {
                token,
                location: Location { line: 0, column: 0 },
            })
            .collect();
        self.with_tokens_with_locations(tokens_with_locations)
    }

    /// Tokenize the sql string and sets this [`Parser`]'s state to
    /// parse the resulting tokens
    ///
    /// Returns an error if there was an error tokenizing the SQL string.
    ///
    /// See example on [`Parser::new()`] for an example
    pub fn try_with_sql(self, sql: &str) -> Result<Self, ParserError> {
        debug!("Parsing sql '{}'...", sql);
        let mut tokenizer = Tokenizer::new(self.dialect, sql);
        let tokens = tokenizer.tokenize()?;
        Ok(self.with_tokens(tokens))
    }

    /// Parse potentially multiple statements
    ///
    /// Example
    /// ```
    /// # use dbtranslate::{parser::{Parser, ParserError}, dialect::GenericDialect};
    /// # fn main() -> Result<(), ParserError> {
    /// let dialect = GenericDialect{};
    /// let statements = Parser::new(&dialect)
    ///   // Parse a SQL string with 2 separate statements
    ///   .try_with_sql("SELECT * FROM foo; SELECT * FROM bar;")?
    ///   .parse_statements()?;
    /// assert_eq!(statements.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_statements(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut stmts = Vec::new();
        let mut expecting_statement_delimiter = false;
        loop {
            // ignore empty statements (between successive statement delimiters)
            while self.consume_token(&Token::SemiColon) {
                expecting_statement_delimiter = false;
            }

            if self.peek_token() == Token::EOF {
                break;
            }
            if expecting_statement_delimiter {
                return self.expected("end of statement", self.peek_token());
            }

            let statement = self.parse_statement()?;
            stmts.push(statement);
            expecting_statement_delimiter = true;
        }
        Ok(stmts)
    }

    /// Convenience method to parse a string with one or more SQL
    /// statements into produce an Abstract Syntax Tree (AST).
    ///
    /// Example
    /// ```
    /// # use dbtranslate::{parser::{Parser, ParserError}, dialect::GenericDialect};
    /// # fn main() -> Result<(), ParserError> {
    /// let dialect = GenericDialect{};
    /// let statements = Parser::parse_sql(
    ///   &dialect, "SELECT * FROM foo"
    /// )?;
    /// assert_eq!(statements.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_sql(dialect: &dyn Dialect, sql: &str) -> Result<Vec<Statement>, ParserError> {
        Parser::new(dialect).try_with_sql(sql)?.parse_statements()
    }

    /// Parse a single top-level statement (such as SELECT etc.),
    /// stopping before the statement separator, if any.
    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        let _guard = self.recursion_counter.try_decrease()?;

        // allow the dialect to override statement parsing
        if let Some(statement) = self.dialect.parse_statement(self) {
            return statement;
        }

        // TODO: Callum add support for DbtConfig
        // This is where we need to add dbt for config?.
        let next_token = self.next_token();
        match &next_token.token {
            Token::DoubleLBrace => {
                //TODO: Some parse_config function? After R brace it should run parse query
                // self.prev_token();
                let config = self.parse_config()?;
                Ok(Statement::Query(Box::new(self.parse_query(Some(config))?)))
            }
            // Token::LJinjaIterator => {
            //     let next_token: TokenWithLocation = self.next_token();
            //     match next_token.token {
            //         Token::Word(w) if w.value.eq_ignore_ascii_case("set") => {
            //             self.next_token(); // Consume the "set" word token
            //             let jinja_variables = self.parse_jinja_variables()?;
            //             // Do something with the jinja_variables or add it to a relevant struct
            //             Ok(Statement::Query(Box::new(self.parse_query(None)?)))
            //         }
            //         Token::Word(w) => {
            //             parser_err!(format!("Expected 'set', found '{}'", w.value))
            //         }
            //         _ => parser_err!("Expected 'set'"),
            //     }
            // }
            Token::Word(w) => match w.keyword {
                Keyword::KILL => {
                    parser_err!(format!("KILL is not supported by dbtranslate"))
                },
                Keyword::SELECT | Keyword::WITH | Keyword::VALUES => {
                    self.prev_token();
                    Ok(Statement::Query(Box::new(self.parse_query(None)?)))
                },
                Keyword::EXPLAIN => {
                    parser_err!(format!("EXPLAIN is not supported by dbtranslate"))
                },
                Keyword::ANALYZE => {
                    parser_err!(format!("ANALYZE is not supported by dbtranslate"))
                },
                Keyword::TRUNCATE => {
                    parser_err!(format!("TRUNCATE is not supported by dbtranslate"))
                },
                Keyword::MSCK => {
                    parser_err!(format!("MSCK is not supported by dbtranslate"))
                },
                Keyword::CREATE => {
                    parser_err!(format!("CREATE is not supported by dbtranslate"))
                },
                Keyword::CACHE => {
                    parser_err!(format!("CACHE is not supported by dbtranslate"))
                },
                Keyword::DROP => {
                    parser_err!(format!("DROP is not supported by dbtranslate"))
                },
                Keyword::DISCARD => {
                    parser_err!(format!("DISCARD is not supported by dbtranslate"))
                },
                Keyword::DECLARE => {
                    parser_err!(format!("DECLARE is not supported by dbtranslate"))
                },
                Keyword::FETCH => {
                    parser_err!(format!("FETCH is not supported by dbtranslate"))
                },
                Keyword::DELETE => {
                    parser_err!(format!("DELETE is not supported by dbtranslate"))
                },
                Keyword::INSERT => {
                    parser_err!(format!("INSERT is not supported by dbtranslate"))
                },
                Keyword::UNCACHE => {
                    parser_err!(format!("UNCACHE is not supported by dbtranslate"))
                },
                Keyword::UPDATE => {
                    parser_err!(format!("UPDATE is not supported by dbtranslate"))
                },
                Keyword::ALTER => {
                    parser_err!(format!("ALTER is not supported by dbtranslate"))
                },
                Keyword::COPY => {
                    parser_err!(format!("COPY is not supported by dbtranslate"))
                },
                Keyword::CLOSE => {
                    parser_err!(format!("CLOSE is not supported by dbtranslate"))
                },
                Keyword::SET => {
                    parser_err!(format!("SET is not supported by dbtranslate outside of jinja"))
                },
                Keyword::SHOW => {
                    parser_err!(format!("SHOW is not supported by dbtranslate"))
                },
                Keyword::USE => {
                    parser_err!(format!("USE is not supported by dbtranslate"))
                },
                Keyword::GRANT => {
                    parser_err!(format!("GRANT is not supported by dbtranslate"))
                },
                Keyword::REVOKE => {
                    parser_err!(format!("REVOKE is not supported by dbtranslate"))
                },
                Keyword::START => {
                    parser_err!(format!("START is not supported by dbtranslate"))
                },
                Keyword::BEGIN => {
                    parser_err!(format!("BEGIN is not supported by dbtranslate"))
                },
                Keyword::SAVEPOINT => {
                    parser_err!(format!("SAVEPOINT is not supported by dbtranslate"))
                },
                Keyword::COMMIT => {
                    parser_err!(format!("COMMIT is not supported by dbtranslate"))
                },
                Keyword::ROLLBACK => {
                    parser_err!(format!("ROLLBACK is not supported by dbtranslate"))
                },
                Keyword::ASSERT => {
                    parser_err!(format!("ASSERT is not supported by dbtranslate"))
                },
                // `PREPARE`, `EXECUTE` and `DEALLOCATE` are Postgres-specific
                // syntaxes. They are used for Postgres prepared statement.
                Keyword::DEALLOCATE => {
                    parser_err!(format!("DEALLOCATE is not supported by dbtranslate"))
                } ,
                Keyword::EXECUTE => {
                    parser_err!(format!("EXECUTE is not supported by dbtranslate"))
                },
                Keyword::COMMENT => {
                    parser_err!(format!("COMMENT is not supported by dbtranslate"))
                },
                Keyword::PREPARE => {
                    parser_err!(format!("PREPARE is not supported by dbtranslate"))   
                },
                Keyword::MERGE => {
                    parser_err!(format!("MERGE is not supported by dbtranslate"))
                },
                _ => self.expected("an SQL statement", next_token),
            },
            Token::LParen => {
                self.prev_token();
                Ok(Statement::Query(Box::new(self.parse_query(None)?)))
            }
            _ => self.expected("an SQL statement", next_token),
        }
    }

    /// Parse a new expression including wildcard & qualified wildcard
    pub fn parse_wildcard_expr(&mut self) -> Result<WildcardExpr, ParserError> {
        let index = self.index;

        let next_token = self.next_token();
        match next_token.token {
            Token::Word(w) if self.peek_token().token == Token::Period => {
                let mut id_parts: Vec<Ident> = vec![w.to_ident()];

                while self.consume_token(&Token::Period) {
                    let next_token = self.next_token();
                    match next_token.token {
                        Token::Word(w) => id_parts.push(w.to_ident()),
                        Token::Mul => {
                            return Ok(WildcardExpr::QualifiedWildcard(ObjectName(id_parts)));
                        }
                        _ => {
                            return self.expected("an identifier or a '*' after '.'", next_token);
                        }
                    }
                }
            }
            Token::Mul => {
                return Ok(WildcardExpr::Wildcard);
            }
            _ => (),
        };

        self.index = index;
        self.parse_expr().map(WildcardExpr::Expr)
    }

    /// Parse a new expression
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let _guard = self.recursion_counter.try_decrease()?;
        self.parse_subexpr(0)
    }

    /// Parse tokens until the precedence changes
    pub fn parse_subexpr(&mut self, precedence: u8) -> Result<Expr, ParserError> {
        debug!("parsing expr");
        let mut expr = self.parse_prefix()?;
        debug!("prefix: {:?}", expr);
        loop {
            let next_precedence = self.get_next_precedence()?;
            debug!("next precedence: {:?}", next_precedence);

            if precedence >= next_precedence {
                break;
            }

            expr = self.parse_infix(expr, next_precedence)?;
        }
        Ok(expr)
    }

    pub fn parse_interval_expr(&mut self) -> Result<Expr, ParserError> {
        let precedence = 0;
        let mut expr = self.parse_prefix()?;

        loop {
            let next_precedence = self.get_next_interval_precedence()?;

            if precedence >= next_precedence {
                break;
            }

            expr = self.parse_infix(expr, next_precedence)?;
        }

        Ok(expr)
    }

    /// Get the precedence of the next token
    /// With AND, OR, and XOR
    pub fn get_next_interval_precedence(&self) -> Result<u8, ParserError> {
        let token = self.peek_token();

        match token.token {
            Token::Word(w) if w.keyword == Keyword::AND => Ok(0),
            Token::Word(w) if w.keyword == Keyword::OR => Ok(0),
            Token::Word(w) if w.keyword == Keyword::XOR => Ok(0),
            _ => self.get_next_precedence(),
        }
    }

    /// Parse a ref function
    fn parse_ref(&mut self) -> Result<Ident, ParserError> {
        let model_name = self.parse_identifier()?;
        self.expect_token(&Token::RParen)?;
        Ok(model_name)
    }

    // Add a new method parse_source
    fn parse_source(&mut self) -> Result<(Ident, Ident), ParserError> {
        let source_name = self.parse_identifier()?;
        self.expect_token(&Token::Comma)?;
        let table_name = self.parse_identifier()?;
        self.expect_token(&Token::RParen)?;
    
        Ok((source_name, table_name))
    }

    // // Parse a jinja set expression
    // fn parse_jinja_variables(&mut self) -> Result<Vec<JinjaVariable>, ParserError> {
    //     let mut jinja_variables = Vec::new();
    
    //     loop {
    //         // Parse variable name
    //         let key = self.parse_identifier()?.value;
            
    //         // Consume the equal sign
    //         // TODO: Change this to any operator
    //         self.expect_token(&Token::Eq)?;
    
    //         // Parse Jinja value
    //         let value = self.parse_jinja_value()?;
    
    //         // Create JinjaVariable and push it to the list
    //         jinja_variables.push(JinjaVariable { key, value });
    
    //         // Check if there's another Jinja variable to parse
    //         // TODO: change this to and / or
    //         if !self.consume_token(&Token::Comma) {
    //             break;
    //         }
    //     }
    
    //     // Consume the closing Jinja delimiter
    //     self.expect_token(&Token::RJinjaIterator)?;
    
    //     Ok(jinja_variables)
    // }

    // fn parse_jinja_value(&mut self) -> Result<JinjaValue, ParserError> {
    //     match self.next_token().token {
    //         Token::DoubleQuotedString(_) => {
    //             self.prev_token();
    //             let value = self.parse_expr()?;
    //             Ok(JinjaValue::Str(value))
    //         }
    //         Token::LBracket => {
    //             let mut list = Vec::new();
    //             loop {
    //                 let current_token = self.peek_token();
    //                 match current_token.token {
    //                     Token::RBracket => {
    //                         self.next_token(); // Consume the RBracket
    //                         break;
    //                     }
    //                     None => return self.expected("a Jinja value or ']', found EOF", None),
    //                     _ => {
    //                         let value = self.parse_jinja_value()?;
    //                         list.push(value);
    //                         if !self.consume_token(&Token::Comma) {
    //                             self.expect_token(&Token::RBracket)?;
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //             Ok(JinjaValue::List(list))
    //         }
    //         _ => self.expected("a Jinja value", self.peek_token()),
    //     }
    // }
    

    /// Parse an expression prefix
    pub fn parse_prefix(&mut self) -> Result<Expr, ParserError> {
        // allow the dialect to override prefix parsing
        if let Some(prefix) = self.dialect.parse_prefix(self) {
            return prefix;
        }

        // PostgreSQL allows any string literal to be preceded by a type name, indicating that the
        // string literal represents a literal of that type. Some examples:
        //
        //      DATE '2020-05-20'
        //      TIMESTAMP WITH TIME ZONE '2020-05-20 7:43:54'
        //      BOOL 'true'
        //
        // The first two are standard SQL, while the latter is a PostgreSQL extension. Complicating
        // matters is the fact that INTERVAL string literals may optionally be followed by special
        // keywords, e.g.:
        //
        //      INTERVAL '7' DAY
        //
        // Note also that naively `SELECT date` looks like a syntax error because the `date` type
        // name is not followed by a string literal, but in fact in PostgreSQL it is a valid
        // expression that should parse as the column name "date".
        return_ok_if_some!(self.maybe_parse(|parser| {
            match parser.parse_data_type()? {
                DataType::Interval => parser.parse_interval(),
                // PostgreSQL allows almost any identifier to be used as custom data type name,
                // and we support that in `parse_data_type()`. But unlike Postgres we don't
                // have a list of globally reserved keywords (since they vary across dialects),
                // so given `NOT 'a' LIKE 'b'`, we'd accept `NOT` as a possible custom data type
                // name, resulting in `NOT 'a'` being recognized as a `TypedString` instead of
                // an unary negation `NOT ('a' LIKE 'b')`. To solve this, we don't accept the
                // `type 'string'` syntax for the custom data types at all.
                DataType::Custom(..) => parser_err!("dummy"),
                data_type => Ok(Expr::TypedString {
                    data_type,
                    value: parser.parse_literal_string()?,
                }),
            }
        }));

        let next_token = self.next_token();
        let expr = match next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::TRUE | Keyword::FALSE | Keyword::NULL => {
                    self.prev_token();
                    Ok(Expr::Value(self.parse_value()?))
                }
                Keyword::CURRENT_CATALOG
                | Keyword::CURRENT_USER
                | Keyword::SESSION_USER
                | Keyword::USER
                    if dialect_of!(self is PostgreSqlDialect | GenericDialect) =>
                {
                    Ok(Expr::Function(Function {
                        name: ObjectName(vec![w.to_ident()]),
                        args: vec![],
                        over: None,
                        distinct: false,
                        special: true,
                    }))
                }
                Keyword::CURRENT_TIMESTAMP
                | Keyword::CURRENT_TIME
                | Keyword::CURRENT_DATE
                | Keyword::LOCALTIME
                | Keyword::LOCALTIMESTAMP => {
                    self.parse_time_functions(ObjectName(vec![w.to_ident()]))
                }
                Keyword::CASE => self.parse_case_expr(),
                Keyword::CAST => self.parse_cast_expr(),
                Keyword::TRY_CAST => self.parse_try_cast_expr(),
                Keyword::SAFE_CAST => self.parse_safe_cast_expr(),
                Keyword::EXISTS => self.parse_exists_expr(false),
                Keyword::EXTRACT => self.parse_extract_expr(),
                Keyword::CEIL => self.parse_ceil_floor_expr(true),
                Keyword::FLOOR => self.parse_ceil_floor_expr(false),
                Keyword::POSITION => self.parse_position_expr(),
                Keyword::SUBSTRING => self.parse_substring_expr(),
                Keyword::OVERLAY => self.parse_overlay_expr(),
                Keyword::TRIM => self.parse_trim_expr(),
                Keyword::INTERVAL => self.parse_interval(),
                Keyword::LISTAGG => self.parse_listagg_expr(),
                // Treat ARRAY[1,2,3] as an array [1,2,3], otherwise try as subquery or a function call
                Keyword::ARRAY if self.peek_token() == Token::LBracket => {
                    self.expect_token(&Token::LBracket)?;
                    self.parse_array_expr(true)
                }
                Keyword::ARRAY
                    if self.peek_token() == Token::LParen =>
                {
                    self.expect_token(&Token::LParen)?;
                    self.parse_array_subquery()
                }
                Keyword::ARRAY_AGG => self.parse_array_agg_expr(),
                Keyword::NOT => self.parse_not(),
                Keyword::MATCH if dialect_of!(self is GenericDialect) => {
                    parser_err!(format!("MATCH is not supported by dbtranslate"))
                }
                // Here `w` is a word, check if it's a part of a multi-part
                // identifier, a function call, or a simple identifier:
                _ => match self.peek_token().token {
                    Token::LParen | Token::Period => {
                        let mut id_parts: Vec<Ident> = vec![w.to_ident()];
                        while self.consume_token(&Token::Period) {
                            let next_token = self.next_token();
                            match next_token.token {
                                Token::Word(w) => id_parts.push(w.to_ident()),
                                _ => {
                                    return self
                                        .expected("an identifier or a '*' after '.'", next_token);
                                }
                            }
                        }

                        if self.consume_token(&Token::LParen) {
                            self.prev_token();
                            self.parse_function(ObjectName(id_parts))
                        } else {
                            Ok(Expr::CompoundIdentifier(id_parts))
                        }
                    }
                    // string introducer https://dev.mysql.com/doc/refman/8.0/en/charset-introducer.html
                    Token::SingleQuotedString(_)
                    | Token::DoubleQuotedString(_)
                    | Token::HexStringLiteral(_)
                        if w.value.starts_with('_') =>
                    {
                        Ok(Expr::IntroducedString {
                            introducer: w.value,
                            value: self.parse_introduced_string_value()?,
                        })
                    }
                    _ => Ok(Expr::Identifier(w.to_ident())),
                },
            }, // End of Token::Word
            // array `[1, 2, 3]`
            Token::LBracket => self.parse_array_expr(false),
            tok @ Token::Minus | tok @ Token::Plus => {
                let op = if tok == Token::Plus {
                    UnaryOperator::Plus
                } else {
                    UnaryOperator::Minus
                };
                Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(self.parse_subexpr(Self::PLUS_MINUS_PREC)?),
                })
            }
            tok @ Token::DoubleExclamationMark
            | tok @ Token::PGSquareRoot
            | tok @ Token::PGCubeRoot
            | tok @ Token::AtSign
            | tok @ Token::Tilde
                if dialect_of!(self is PostgreSqlDialect) =>
            {
                let op = match tok {
                    Token::DoubleExclamationMark => UnaryOperator::PGPrefixFactorial,
                    Token::PGSquareRoot => UnaryOperator::PGSquareRoot,
                    Token::PGCubeRoot => UnaryOperator::PGCubeRoot,
                    Token::AtSign => UnaryOperator::PGAbs,
                    Token::Tilde => UnaryOperator::PGBitwiseNot,
                    _ => unreachable!(),
                };
                Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(self.parse_subexpr(Self::PLUS_MINUS_PREC)?),
                })
            }
            Token::EscapedStringLiteral(_) if dialect_of!(self is PostgreSqlDialect | GenericDialect) =>
            {
                self.prev_token();
                Ok(Expr::Value(self.parse_value()?))
            }
            Token::Number(_, _)
            | Token::SingleQuotedString(_)
            | Token::DoubleQuotedString(_)
            | Token::DollarQuotedString(_)
            | Token::SingleQuotedByteStringLiteral(_)
            | Token::DoubleQuotedByteStringLiteral(_)
            | Token::RawStringLiteral(_)
            | Token::NationalStringLiteral(_)
            | Token::HexStringLiteral(_) => {
                self.prev_token();
                Ok(Expr::Value(self.parse_value()?))
            }
            Token::LParen => {
                let expr =
                    if self.parse_keyword(Keyword::SELECT) || self.parse_keyword(Keyword::WITH) {
                        self.prev_token();
                        Expr::Subquery(Box::new(self.parse_query(None)?))
                    } else {
                        let exprs = self.parse_comma_separated(Parser::parse_expr)?;
                        match exprs.len() {
                            0 => unreachable!(), // parse_comma_separated ensures 1 or more
                            1 => Expr::Nested(Box::new(exprs.into_iter().next().unwrap())),
                            _ => Expr::Tuple(exprs),
                        }
                    };
                self.expect_token(&Token::RParen)?;
                if !self.consume_token(&Token::Period) {
                    Ok(expr)
                } else {
                    let tok = self.next_token();
                    let key = match tok.token {
                        Token::Word(word) => word.to_ident(),
                        _ => return parser_err!(format!("Expected identifier, found: {tok}")),
                    };
                    Ok(Expr::CompositeAccess {
                        expr: Box::new(expr),
                        key,
                    })
                }
            }
            Token::Placeholder(_) | Token::Colon | Token::AtSign => {
                self.prev_token();
                Ok(Expr::Value(self.parse_value()?))
            }
            _ => self.expected("an expression:", next_token),
        }?;

        if self.parse_keyword(Keyword::COLLATE) {
            Ok(Expr::Collate {
                expr: Box::new(expr),
                collation: self.parse_object_name()?,
            })
        } else {
            Ok(expr)
        }
    }

    pub fn parse_config(&mut self) -> Result<DbtConfig, ParserError> {
        let mut config_values = HashMap::new();
        
        self.expect_token(&Token::Word(Word {
            value: "config".to_string(),
            quote_style: None,
            keyword: Keyword::NoKeyword,
        }))?;
    
        self.expect_token(&Token::LParen)?;
    
        while self.peek_token() != Token::RParen {
            let key = self.parse_identifier()?.to_string();
            self.expect_token(&Token::Eq)?;
            let value = match self.next_token().token {
                Token::Word(w) => DbtConfigValue::String(w.value),
                Token::SingleQuotedString(s) => DbtConfigValue::String(s),
                Token::NationalStringLiteral(s) => DbtConfigValue::String(s),
                Token::HexStringLiteral(s) => DbtConfigValue::String(s),
                Token::LBracket => {
                    let mut values = Vec::new();
                    while self.peek_token() != Token::RBracket {
                        if let Token::Word(w) = self.next_token().token {
                            values.push(w.value);
                        } else {
                            return self.expected("a string value inside the list", self.peek_token());
                        }
                        if self.peek_token() != Token::RBracket {
                            self.expect_token(&Token::Comma)?;
                        }
                    }
                    self.expect_token(&Token::RBracket)?;
                    DbtConfigValue::List(values)
                }
                _ => return self.expected("a string value or a list", self.peek_token()),
            };
    
            config_values.insert(key, value);
    
            if self.peek_token() != Token::RParen {
                self.expect_token(&Token::Comma)?;
            }
        }
    
        self.expect_token(&Token::RParen)?;
        self.expect_token(&Token::DoubleRBrace)?;
    
        Ok(DbtConfig {
            values: config_values,
        })
    }
    
    pub fn parse_function(&mut self, name: ObjectName) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let distinct = self.parse_all_or_distinct()?;
        let args = self.parse_optional_args()?;
        let over = if self.parse_keyword(Keyword::OVER) {
            // TBD: support window names (`OVER mywin`) in place of inline specification
            self.expect_token(&Token::LParen)?;
            let partition_by = if self.parse_keywords(&[Keyword::PARTITION, Keyword::BY]) {
                // a list of possibly-qualified column names
                self.parse_comma_separated(Parser::parse_expr)?
            } else {
                vec![]
            };
            let order_by = if self.parse_keywords(&[Keyword::ORDER, Keyword::BY]) {
                self.parse_comma_separated(Parser::parse_order_by_expr)?
            } else {
                vec![]
            };
            let window_frame = if !self.consume_token(&Token::RParen) {
                let window_frame = self.parse_window_frame()?;
                self.expect_token(&Token::RParen)?;
                Some(window_frame)
            } else {
                None
            };

            Some(WindowSpec {
                partition_by,
                order_by,
                window_frame,
            })
        } else {
            None
        };
        Ok(Expr::Function(Function {
            name,
            args,
            over,
            distinct,
            special: false,
        }))
    }

    pub fn parse_time_functions(&mut self, name: ObjectName) -> Result<Expr, ParserError> {
        let args = if self.consume_token(&Token::LParen) {
            self.parse_optional_args()?
        } else {
            vec![]
        };
        Ok(Expr::Function(Function {
            name,
            args,
            over: None,
            distinct: false,
            special: false,
        }))
    }

    pub fn parse_window_frame_units(&mut self) -> Result<WindowFrameUnits, ParserError> {
        let next_token = self.next_token();
        match &next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::ROWS => Ok(WindowFrameUnits::Rows),
                Keyword::RANGE => Ok(WindowFrameUnits::Range),
                Keyword::GROUPS => Ok(WindowFrameUnits::Groups),
                _ => self.expected("ROWS, RANGE, GROUPS", next_token)?,
            },
            _ => self.expected("ROWS, RANGE, GROUPS", next_token),
        }
    }

    pub fn parse_window_frame(&mut self) -> Result<WindowFrame, ParserError> {
        let units = self.parse_window_frame_units()?;
        let (start_bound, end_bound) = if self.parse_keyword(Keyword::BETWEEN) {
            let start_bound = self.parse_window_frame_bound()?;
            self.expect_keyword(Keyword::AND)?;
            let end_bound = Some(self.parse_window_frame_bound()?);
            (start_bound, end_bound)
        } else {
            (self.parse_window_frame_bound()?, None)
        };
        Ok(WindowFrame {
            units,
            start_bound,
            end_bound,
        })
    }

    /// Parse `CURRENT ROW` or `{ <positive number> | UNBOUNDED } { PRECEDING | FOLLOWING }`
    pub fn parse_window_frame_bound(&mut self) -> Result<WindowFrameBound, ParserError> {
        if self.parse_keywords(&[Keyword::CURRENT, Keyword::ROW]) {
            Ok(WindowFrameBound::CurrentRow)
        } else {
            let rows = if self.parse_keyword(Keyword::UNBOUNDED) {
                None
            } else {
                Some(Box::new(match self.peek_token().token {
                    Token::SingleQuotedString(_) => self.parse_interval()?,
                    _ => self.parse_expr()?,
                }))
            };
            if self.parse_keyword(Keyword::PRECEDING) {
                Ok(WindowFrameBound::Preceding(rows))
            } else if self.parse_keyword(Keyword::FOLLOWING) {
                Ok(WindowFrameBound::Following(rows))
            } else {
                self.expected("PRECEDING or FOLLOWING", self.peek_token())
            }
        }
    }

    /// parse a group by expr. a group by expr can be one of group sets, roll up, cube, or simple
    /// expr.
    fn parse_group_by_expr(&mut self) -> Result<Expr, ParserError> {
        if dialect_of!(self is PostgreSqlDialect | GenericDialect) {
            if self.parse_keywords(&[Keyword::GROUPING, Keyword::SETS]) {
                self.expect_token(&Token::LParen)?;
                let result = self.parse_comma_separated(|p| p.parse_tuple(false, true))?;
                self.expect_token(&Token::RParen)?;
                Ok(Expr::GroupingSets(result))
            } else if self.parse_keyword(Keyword::CUBE) {
                self.expect_token(&Token::LParen)?;
                let result = self.parse_comma_separated(|p| p.parse_tuple(true, true))?;
                self.expect_token(&Token::RParen)?;
                Ok(Expr::Cube(result))
            } else if self.parse_keyword(Keyword::ROLLUP) {
                self.expect_token(&Token::LParen)?;
                let result = self.parse_comma_separated(|p| p.parse_tuple(true, true))?;
                self.expect_token(&Token::RParen)?;
                Ok(Expr::Rollup(result))
            } else {
                self.parse_expr()
            }
        } else {
            // TODO parse rollup for other dialects
            self.parse_expr()
        }
    }

    /// parse a tuple with `(` and `)`.
    /// If `lift_singleton` is true, then a singleton tuple is lifted to a tuple of length 1, otherwise it will fail.
    /// If `allow_empty` is true, then an empty tuple is allowed.
    fn parse_tuple(
        &mut self,
        lift_singleton: bool,
        allow_empty: bool,
    ) -> Result<Vec<Expr>, ParserError> {
        if lift_singleton {
            if self.consume_token(&Token::LParen) {
                let result = if allow_empty && self.consume_token(&Token::RParen) {
                    vec![]
                } else {
                    let result = self.parse_comma_separated(Parser::parse_expr)?;
                    self.expect_token(&Token::RParen)?;
                    result
                };
                Ok(result)
            } else {
                Ok(vec![self.parse_expr()?])
            }
        } else {
            self.expect_token(&Token::LParen)?;
            let result = if allow_empty && self.consume_token(&Token::RParen) {
                vec![]
            } else {
                let result = self.parse_comma_separated(Parser::parse_expr)?;
                self.expect_token(&Token::RParen)?;
                result
            };
            Ok(result)
        }
    }

    pub fn parse_case_expr(&mut self) -> Result<Expr, ParserError> {
        let mut operand = None;
        if !self.parse_keyword(Keyword::WHEN) {
            operand = Some(Box::new(self.parse_expr()?));
            self.expect_keyword(Keyword::WHEN)?;
        }
        let mut conditions = vec![];
        let mut results = vec![];
        loop {
            conditions.push(self.parse_expr()?);
            self.expect_keyword(Keyword::THEN)?;
            results.push(self.parse_expr()?);
            if !self.parse_keyword(Keyword::WHEN) {
                break;
            }
        }
        let else_result = if self.parse_keyword(Keyword::ELSE) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect_keyword(Keyword::END)?;
        Ok(Expr::Case {
            operand,
            conditions,
            results,
            else_result,
        })
    }

    /// Parse a SQL CAST function e.g. `CAST(expr AS FLOAT)`
    pub fn parse_cast_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect_keyword(Keyword::AS)?;
        let data_type = self.parse_data_type()?;
        self.expect_token(&Token::RParen)?;
        Ok(Expr::Cast {
            expr: Box::new(expr),
            data_type,
        })
    }

    /// Parse a SQL TRY_CAST function e.g. `TRY_CAST(expr AS FLOAT)`
    pub fn parse_try_cast_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect_keyword(Keyword::AS)?;
        let data_type = self.parse_data_type()?;
        self.expect_token(&Token::RParen)?;
        Ok(Expr::TryCast {
            expr: Box::new(expr),
            data_type,
        })
    }

    /// Parse a BigQuery SAFE_CAST function e.g. `SAFE_CAST(expr AS FLOAT64)`
    pub fn parse_safe_cast_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect_keyword(Keyword::AS)?;
        let data_type = self.parse_data_type()?;
        self.expect_token(&Token::RParen)?;
        Ok(Expr::SafeCast {
            expr: Box::new(expr),
            data_type,
        })
    }

    /// Parse a SQL EXISTS expression e.g. `WHERE EXISTS(SELECT ...)`.
    pub fn parse_exists_expr(&mut self, negated: bool) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let exists_node = Expr::Exists {
            negated,
            subquery: Box::new(self.parse_query(None)?),
        };
        self.expect_token(&Token::RParen)?;
        Ok(exists_node)
    }

    pub fn parse_extract_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let field = self.parse_date_time_field()?;
        self.expect_keyword(Keyword::FROM)?;
        let expr = self.parse_expr()?;
        self.expect_token(&Token::RParen)?;
        Ok(Expr::Extract {
            field,
            expr: Box::new(expr),
        })
    }

    pub fn parse_ceil_floor_expr(&mut self, is_ceil: bool) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        // Parse `CEIL/FLOOR(expr)`
        let mut field = DateTimeField::NoDateTime;
        let keyword_to = self.parse_keyword(Keyword::TO);
        if keyword_to {
            // Parse `CEIL/FLOOR(expr TO DateTimeField)`
            field = self.parse_date_time_field()?;
        }
        self.expect_token(&Token::RParen)?;
        if is_ceil {
            Ok(Expr::Ceil {
                expr: Box::new(expr),
                field,
            })
        } else {
            Ok(Expr::Floor {
                expr: Box::new(expr),
                field,
            })
        }
    }

    pub fn parse_position_expr(&mut self) -> Result<Expr, ParserError> {
        // PARSE SELECT POSITION('@' in field)
        self.expect_token(&Token::LParen)?;

        // Parse the subexpr till the IN keyword
        let expr = self.parse_subexpr(Self::BETWEEN_PREC)?;
        if self.parse_keyword(Keyword::IN) {
            let from = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            Ok(Expr::Position {
                expr: Box::new(expr),
                r#in: Box::new(from),
            })
        } else {
            parser_err!("Position function must include IN keyword".to_string())
        }
    }

    pub fn parse_substring_expr(&mut self) -> Result<Expr, ParserError> {
        // PARSE SUBSTRING (EXPR [FROM 1] [FOR 3])
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        let mut from_expr = None;
        if self.parse_keyword(Keyword::FROM) || self.consume_token(&Token::Comma) {
            from_expr = Some(self.parse_expr()?);
        }

        let mut to_expr = None;
        if self.parse_keyword(Keyword::FOR) || self.consume_token(&Token::Comma) {
            to_expr = Some(self.parse_expr()?);
        }
        self.expect_token(&Token::RParen)?;

        Ok(Expr::Substring {
            expr: Box::new(expr),
            substring_from: from_expr.map(Box::new),
            substring_for: to_expr.map(Box::new),
        })
    }

    pub fn parse_overlay_expr(&mut self) -> Result<Expr, ParserError> {
        // PARSE OVERLAY (EXPR PLACING EXPR FROM 1 [FOR 3])
        self.expect_token(&Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect_keyword(Keyword::PLACING)?;
        let what_expr = self.parse_expr()?;
        self.expect_keyword(Keyword::FROM)?;
        let from_expr = self.parse_expr()?;
        let mut for_expr = None;
        if self.parse_keyword(Keyword::FOR) {
            for_expr = Some(self.parse_expr()?);
        }
        self.expect_token(&Token::RParen)?;

        Ok(Expr::Overlay {
            expr: Box::new(expr),
            overlay_what: Box::new(what_expr),
            overlay_from: Box::new(from_expr),
            overlay_for: for_expr.map(Box::new),
        })
    }

    /// ```sql
    /// TRIM ([WHERE] ['text' FROM] 'text')
    /// TRIM ('text')
    /// ```
    pub fn parse_trim_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let mut trim_where = None;
        if let Token::Word(word) = self.peek_token().token {
            if [Keyword::BOTH, Keyword::LEADING, Keyword::TRAILING]
                .iter()
                .any(|d| word.keyword == *d)
            {
                trim_where = Some(self.parse_trim_where()?);
            }
        }
        let expr = self.parse_expr()?;
        if self.parse_keyword(Keyword::FROM) {
            let trim_what = Box::new(expr);
            let expr = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            Ok(Expr::Trim {
                expr: Box::new(expr),
                trim_where,
                trim_what: Some(trim_what),
            })
        } else {
            self.expect_token(&Token::RParen)?;
            Ok(Expr::Trim {
                expr: Box::new(expr),
                trim_where,
                trim_what: None,
            })
        }
    }

    pub fn parse_trim_where(&mut self) -> Result<TrimWhereField, ParserError> {
        let next_token = self.next_token();
        match &next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::BOTH => Ok(TrimWhereField::Both),
                Keyword::LEADING => Ok(TrimWhereField::Leading),
                Keyword::TRAILING => Ok(TrimWhereField::Trailing),
                _ => self.expected("trim_where field", next_token)?,
            },
            _ => self.expected("trim_where field", next_token),
        }
    }

    /// Parses an array expression `[ex1, ex2, ..]`
    /// if `named` is `true`, came from an expression like  `ARRAY[ex1, ex2]`
    pub fn parse_array_expr(&mut self, named: bool) -> Result<Expr, ParserError> {
        if self.peek_token().token == Token::RBracket {
            let _ = self.next_token(); // consume ]
            Ok(Expr::Array(Array {
                elem: vec![],
                named,
            }))
        } else {
            let exprs = self.parse_comma_separated(Parser::parse_expr)?;
            self.expect_token(&Token::RBracket)?;
            Ok(Expr::Array(Array { elem: exprs, named }))
        }
    }

    // Parses an array constructed from a subquery
    pub fn parse_array_subquery(&mut self) -> Result<Expr, ParserError> {
        let query = self.parse_query(None)?;
        self.expect_token(&Token::RParen)?;
        Ok(Expr::ArraySubquery(Box::new(query)))
    }

    /// Parse a SQL LISTAGG expression, e.g. `LISTAGG(...) WITHIN GROUP (ORDER BY ...)`.
    pub fn parse_listagg_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let distinct = self.parse_all_or_distinct()?;
        let expr = Box::new(self.parse_expr()?);
        // While ANSI SQL would would require the separator, Redshift makes this optional. Here we
        // choose to make the separator optional as this provides the more general implementation.
        let separator = if self.consume_token(&Token::Comma) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        let on_overflow = if self.parse_keywords(&[Keyword::ON, Keyword::OVERFLOW]) {
            if self.parse_keyword(Keyword::ERROR) {
                Some(ListAggOnOverflow::Error)
            } else {
                self.expect_keyword(Keyword::TRUNCATE)?;
                let filler = match self.peek_token().token {
                    Token::Word(w)
                        if w.keyword == Keyword::WITH || w.keyword == Keyword::WITHOUT =>
                    {
                        None
                    }
                    Token::SingleQuotedString(_)
                    | Token::EscapedStringLiteral(_)
                    | Token::NationalStringLiteral(_)
                    | Token::HexStringLiteral(_) => Some(Box::new(self.parse_expr()?)),
                    _ => self.expected(
                        "either filler, WITH, or WITHOUT in LISTAGG",
                        self.peek_token(),
                    )?,
                };
                let with_count = self.parse_keyword(Keyword::WITH);
                if !with_count && !self.parse_keyword(Keyword::WITHOUT) {
                    self.expected("either WITH or WITHOUT in LISTAGG", self.peek_token())?;
                }
                self.expect_keyword(Keyword::COUNT)?;
                Some(ListAggOnOverflow::Truncate { filler, with_count })
            }
        } else {
            None
        };
        self.expect_token(&Token::RParen)?;
        // Once again ANSI SQL requires WITHIN GROUP, but Redshift does not. Again we choose the
        // more general implementation.
        let within_group = if self.parse_keywords(&[Keyword::WITHIN, Keyword::GROUP]) {
            self.expect_token(&Token::LParen)?;
            self.expect_keywords(&[Keyword::ORDER, Keyword::BY])?;
            let order_by_expr = self.parse_comma_separated(Parser::parse_order_by_expr)?;
            self.expect_token(&Token::RParen)?;
            order_by_expr
        } else {
            vec![]
        };
        Ok(Expr::ListAgg(ListAgg {
            distinct,
            expr,
            separator,
            on_overflow,
            within_group,
        }))
    }

    pub fn parse_array_agg_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LParen)?;
        let distinct = self.parse_keyword(Keyword::DISTINCT);
        let expr = Box::new(self.parse_expr()?);
        // ANSI SQL and BigQuery define ORDER BY inside function.
        if !self.dialect.supports_within_after_array_aggregation() {
            let order_by = if self.parse_keywords(&[Keyword::ORDER, Keyword::BY]) {
                let order_by_expr = self.parse_order_by_expr()?;
                Some(Box::new(order_by_expr))
            } else {
                None
            };
            let limit = if self.parse_keyword(Keyword::LIMIT) {
                self.parse_limit()?.map(Box::new)
            } else {
                None
            };
            self.expect_token(&Token::RParen)?;
            return Ok(Expr::ArrayAgg(ArrayAgg {
                distinct,
                expr,
                order_by,
                limit,
                within_group: false,
            }));
        }
        // Snowflake defines ORDERY BY in within group instead of inside the function like
        // ANSI SQL.
        self.expect_token(&Token::RParen)?;
        let within_group = if self.parse_keywords(&[Keyword::WITHIN, Keyword::GROUP]) {
            self.expect_token(&Token::LParen)?;
            self.expect_keywords(&[Keyword::ORDER, Keyword::BY])?;
            let order_by_expr = self.parse_order_by_expr()?;
            self.expect_token(&Token::RParen)?;
            Some(Box::new(order_by_expr))
        } else {
            None
        };

        Ok(Expr::ArrayAgg(ArrayAgg {
            distinct,
            expr,
            order_by: within_group,
            limit: None,
            within_group: true,
        }))
    }

    // This function parses date/time fields for the EXTRACT function-like
    // operator, interval qualifiers, and the ceil/floor operations.
    // EXTRACT supports a wider set of date/time fields than interval qualifiers,
    // so this function may need to be split in two.
    pub fn parse_date_time_field(&mut self) -> Result<DateTimeField, ParserError> {
        let next_token = self.next_token();
        match &next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::YEAR => Ok(DateTimeField::Year),
                Keyword::MONTH => Ok(DateTimeField::Month),
                Keyword::WEEK => Ok(DateTimeField::Week),
                Keyword::DAY => Ok(DateTimeField::Day),
                Keyword::DATE => Ok(DateTimeField::Date),
                Keyword::HOUR => Ok(DateTimeField::Hour),
                Keyword::MINUTE => Ok(DateTimeField::Minute),
                Keyword::SECOND => Ok(DateTimeField::Second),
                Keyword::CENTURY => Ok(DateTimeField::Century),
                Keyword::DECADE => Ok(DateTimeField::Decade),
                Keyword::DOY => Ok(DateTimeField::Doy),
                Keyword::DOW => Ok(DateTimeField::Dow),
                Keyword::EPOCH => Ok(DateTimeField::Epoch),
                Keyword::ISODOW => Ok(DateTimeField::Isodow),
                Keyword::ISOYEAR => Ok(DateTimeField::Isoyear),
                Keyword::JULIAN => Ok(DateTimeField::Julian),
                Keyword::MICROSECOND => Ok(DateTimeField::Microsecond),
                Keyword::MICROSECONDS => Ok(DateTimeField::Microseconds),
                Keyword::MILLENIUM => Ok(DateTimeField::Millenium),
                Keyword::MILLENNIUM => Ok(DateTimeField::Millennium),
                Keyword::MILLISECOND => Ok(DateTimeField::Millisecond),
                Keyword::MILLISECONDS => Ok(DateTimeField::Milliseconds),
                Keyword::NANOSECOND => Ok(DateTimeField::Nanosecond),
                Keyword::NANOSECONDS => Ok(DateTimeField::Nanoseconds),
                Keyword::QUARTER => Ok(DateTimeField::Quarter),
                Keyword::TIMEZONE => Ok(DateTimeField::Timezone),
                Keyword::TIMEZONE_HOUR => Ok(DateTimeField::TimezoneHour),
                Keyword::TIMEZONE_MINUTE => Ok(DateTimeField::TimezoneMinute),
                _ => self.expected("date/time field", next_token),
            },
            _ => self.expected("date/time field", next_token),
        }
    }

    pub fn parse_not(&mut self) -> Result<Expr, ParserError> {
        match self.peek_token().token {
            Token::Word(w) => match w.keyword {
                Keyword::EXISTS => {
                    let negated = true;
                    let _ = self.parse_keyword(Keyword::EXISTS);
                    self.parse_exists_expr(negated)
                }
                _ => Ok(Expr::UnaryOp {
                    op: UnaryOperator::Not,
                    expr: Box::new(self.parse_subexpr(Self::UNARY_NOT_PREC)?),
                }),
            },
            _ => Ok(Expr::UnaryOp {
                op: UnaryOperator::Not,
                expr: Box::new(self.parse_subexpr(Self::UNARY_NOT_PREC)?),
            }),
        }
    }

    /// Parse an INTERVAL expression.
    ///
    /// Some syntactically valid intervals:
    ///
    ///   1. `INTERVAL '1' DAY`
    ///   2. `INTERVAL '1-1' YEAR TO MONTH`
    ///   3. `INTERVAL '1' SECOND`
    ///   4. `INTERVAL '1:1:1.1' HOUR (5) TO SECOND (5)`
    ///   5. `INTERVAL '1.1' SECOND (2, 2)`
    ///   6. `INTERVAL '1:1' HOUR (5) TO MINUTE (5)`
    ///   7. (MySql and BigQuey only):`INTERVAL 1 DAY`
    ///
    /// Note that we do not currently attempt to parse the quoted value.
    pub fn parse_interval(&mut self) -> Result<Expr, ParserError> {
        // The SQL standard allows an optional sign before the value string, but
        // it is not clear if any implementations support that syntax, so we
        // don't currently try to parse it. (The sign can instead be included
        // inside the value string.)

        // The first token in an interval is a string literal which specifies
        // the duration of the interval.
        let value = self.parse_interval_expr()?;

        // Following the string literal is a qualifier which indicates the units
        // of the duration specified in the string literal.
        //
        // Note that PostgreSQL allows omitting the qualifier, so we provide
        // this more general implementation.
        let leading_field = match self.peek_token().token {
            Token::Word(kw)
                if [
                    Keyword::YEAR,
                    Keyword::MONTH,
                    Keyword::WEEK,
                    Keyword::DAY,
                    Keyword::HOUR,
                    Keyword::MINUTE,
                    Keyword::SECOND,
                    Keyword::CENTURY,
                    Keyword::DECADE,
                    Keyword::DOW,
                    Keyword::DOY,
                    Keyword::EPOCH,
                    Keyword::ISODOW,
                    Keyword::ISOYEAR,
                    Keyword::JULIAN,
                    Keyword::MICROSECOND,
                    Keyword::MICROSECONDS,
                    Keyword::MILLENIUM,
                    Keyword::MILLENNIUM,
                    Keyword::MILLISECOND,
                    Keyword::MILLISECONDS,
                    Keyword::NANOSECOND,
                    Keyword::NANOSECONDS,
                    Keyword::QUARTER,
                    Keyword::TIMEZONE,
                    Keyword::TIMEZONE_HOUR,
                    Keyword::TIMEZONE_MINUTE,
                ]
                .iter()
                .any(|d| kw.keyword == *d) =>
            {
                Some(self.parse_date_time_field()?)
            }
            _ => None,
        };

        let (leading_precision, last_field, fsec_precision) =
            if leading_field == Some(DateTimeField::Second) {
                // SQL mandates special syntax for `SECOND TO SECOND` literals.
                // Instead of
                //     `SECOND [(<leading precision>)] TO SECOND[(<fractional seconds precision>)]`
                // one must use the special format:
                //     `SECOND [( <leading precision> [ , <fractional seconds precision>] )]`
                let last_field = None;
                let (leading_precision, fsec_precision) = self.parse_optional_precision_scale()?;
                (leading_precision, last_field, fsec_precision)
            } else {
                let leading_precision = self.parse_optional_precision()?;
                if self.parse_keyword(Keyword::TO) {
                    let last_field = Some(self.parse_date_time_field()?);
                    let fsec_precision = if last_field == Some(DateTimeField::Second) {
                        self.parse_optional_precision()?
                    } else {
                        None
                    };
                    (leading_precision, last_field, fsec_precision)
                } else {
                    (leading_precision, None, None)
                }
            };

        Ok(Expr::Interval {
            value: Box::new(value),
            leading_field,
            leading_precision,
            last_field,
            fractional_seconds_precision: fsec_precision,
        })
    }

    /// Parse an operator following an expression
    pub fn parse_infix(&mut self, expr: Expr, precedence: u8) -> Result<Expr, ParserError> {
        // allow the dialect to override infix parsing
        if let Some(infix) = self.dialect.parse_infix(self, &expr, precedence) {
            return infix;
        }

        let tok = self.next_token();

        let regular_binary_operator = match &tok.token {
            Token::Spaceship => Some(BinaryOperator::Spaceship),
            Token::DoubleEq => Some(BinaryOperator::Eq),
            Token::Eq => Some(BinaryOperator::Eq),
            Token::Neq => Some(BinaryOperator::NotEq),
            Token::Gt => Some(BinaryOperator::Gt),
            Token::GtEq => Some(BinaryOperator::GtEq),
            Token::Lt => Some(BinaryOperator::Lt),
            Token::LtEq => Some(BinaryOperator::LtEq),
            Token::Plus => Some(BinaryOperator::Plus),
            Token::Minus => Some(BinaryOperator::Minus),
            Token::Mul => Some(BinaryOperator::Multiply),
            Token::Mod => Some(BinaryOperator::Modulo),
            Token::StringConcat => Some(BinaryOperator::StringConcat),
            Token::Pipe => Some(BinaryOperator::BitwiseOr),
            Token::Caret => {
                // In PostgreSQL, ^ stands for the exponentiation operation,
                // and # stands for XOR. See https://www.postgresql.org/docs/current/functions-math.html
                if dialect_of!(self is PostgreSqlDialect) {
                    Some(BinaryOperator::PGExp)
                } else {
                    Some(BinaryOperator::BitwiseXor)
                }
            }
            Token::Ampersand => Some(BinaryOperator::BitwiseAnd),
            Token::Div => Some(BinaryOperator::Divide),
            Token::ShiftLeft if dialect_of!(self is PostgreSqlDialect | GenericDialect) => {
                Some(BinaryOperator::PGBitwiseShiftLeft)
            }
            Token::ShiftRight if dialect_of!(self is PostgreSqlDialect | GenericDialect) => {
                Some(BinaryOperator::PGBitwiseShiftRight)
            }
            Token::Sharp if dialect_of!(self is PostgreSqlDialect) => {
                Some(BinaryOperator::PGBitwiseXor)
            }
            Token::Tilde => Some(BinaryOperator::PGRegexMatch),
            Token::TildeAsterisk => Some(BinaryOperator::PGRegexIMatch),
            Token::ExclamationMarkTilde => Some(BinaryOperator::PGRegexNotMatch),
            Token::ExclamationMarkTildeAsterisk => Some(BinaryOperator::PGRegexNotIMatch),
            Token::Word(w) => match w.keyword {
                Keyword::AND => Some(BinaryOperator::And),
                Keyword::OR => Some(BinaryOperator::Or),
                Keyword::XOR => Some(BinaryOperator::Xor),
                Keyword::OPERATOR if dialect_of!(self is PostgreSqlDialect | GenericDialect) => {
                    self.expect_token(&Token::LParen)?;
                    // there are special rules for operator names in
                    // postgres so we can not use 'parse_object'
                    // or similar.
                    // See https://www.postgresql.org/docs/current/sql-createoperator.html
                    let mut idents = vec![];
                    loop {
                        idents.push(self.next_token().to_string());
                        if !self.consume_token(&Token::Period) {
                            break;
                        }
                    }
                    self.expect_token(&Token::RParen)?;
                    Some(BinaryOperator::PGCustomBinaryOperator(idents))
                }
                _ => None,
            },
            _ => None,
        };

        if let Some(op) = regular_binary_operator {
            if let Some(keyword) = self.parse_one_of_keywords(&[Keyword::ANY, Keyword::ALL]) {
                self.expect_token(&Token::LParen)?;
                let right = self.parse_subexpr(precedence)?;
                self.expect_token(&Token::RParen)?;

                let right = match keyword {
                    Keyword::ALL => Box::new(Expr::AllOp(Box::new(right))),
                    Keyword::ANY => Box::new(Expr::AnyOp(Box::new(right))),
                    _ => unreachable!(),
                };

                Ok(Expr::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right,
                })
            } else {
                Ok(Expr::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(self.parse_subexpr(precedence)?),
                })
            }
        } else if let Token::Word(w) = &tok.token {
            match w.keyword {
                Keyword::IS => {
                    if self.parse_keyword(Keyword::NULL) {
                        Ok(Expr::IsNull(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::NOT, Keyword::NULL]) {
                        Ok(Expr::IsNotNull(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::TRUE]) {
                        Ok(Expr::IsTrue(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::NOT, Keyword::TRUE]) {
                        Ok(Expr::IsNotTrue(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::FALSE]) {
                        Ok(Expr::IsFalse(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::NOT, Keyword::FALSE]) {
                        Ok(Expr::IsNotFalse(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::UNKNOWN]) {
                        Ok(Expr::IsUnknown(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::NOT, Keyword::UNKNOWN]) {
                        Ok(Expr::IsNotUnknown(Box::new(expr)))
                    } else if self.parse_keywords(&[Keyword::DISTINCT, Keyword::FROM]) {
                        let expr2 = self.parse_expr()?;
                        Ok(Expr::IsDistinctFrom(Box::new(expr), Box::new(expr2)))
                    } else if self.parse_keywords(&[Keyword::NOT, Keyword::DISTINCT, Keyword::FROM])
                    {
                        let expr2 = self.parse_expr()?;
                        Ok(Expr::IsNotDistinctFrom(Box::new(expr), Box::new(expr2)))
                    } else {
                        self.expected(
                            "[NOT] NULL or TRUE|FALSE or [NOT] DISTINCT FROM after IS",
                            self.peek_token(),
                        )
                    }
                }
                Keyword::AT => {
                    // if self.parse_keyword(Keyword::TIME) {
                    //     self.expect_keyword(Keyword::ZONE)?;
                    if self.parse_keywords(&[Keyword::TIME, Keyword::ZONE]) {
                        let time_zone = self.next_token();
                        match time_zone.token {
                            Token::SingleQuotedString(time_zone) => {
                                log::trace!("Peek token: {:?}", self.peek_token());
                                Ok(Expr::AtTimeZone {
                                    timestamp: Box::new(expr),
                                    time_zone,
                                })
                            }
                            _ => self.expected(
                                "Expected Token::SingleQuotedString after AT TIME ZONE",
                                time_zone,
                            ),
                        }
                    } else {
                        self.expected("Expected Token::Word after AT", tok)
                    }
                }
                Keyword::NOT
                | Keyword::IN
                | Keyword::BETWEEN
                | Keyword::LIKE
                | Keyword::ILIKE
                | Keyword::SIMILAR => {
                    self.prev_token();
                    let negated = self.parse_keyword(Keyword::NOT);
                    if self.parse_keyword(Keyword::IN) {
                        self.parse_in(expr, negated)
                    } else if self.parse_keyword(Keyword::BETWEEN) {
                        self.parse_between(expr, negated)
                    } else if self.parse_keyword(Keyword::LIKE) {
                        Ok(Expr::Like {
                            negated,
                            expr: Box::new(expr),
                            pattern: Box::new(self.parse_subexpr(Self::LIKE_PREC)?),
                            escape_char: self.parse_escape_char()?,
                        })
                    } else if self.parse_keyword(Keyword::ILIKE) {
                        Ok(Expr::ILike {
                            negated,
                            expr: Box::new(expr),
                            pattern: Box::new(self.parse_subexpr(Self::LIKE_PREC)?),
                            escape_char: self.parse_escape_char()?,
                        })
                    } else if self.parse_keywords(&[Keyword::SIMILAR, Keyword::TO]) {
                        Ok(Expr::SimilarTo {
                            negated,
                            expr: Box::new(expr),
                            pattern: Box::new(self.parse_subexpr(Self::LIKE_PREC)?),
                            escape_char: self.parse_escape_char()?,
                        })
                    } else {
                        self.expected("IN or BETWEEN after NOT", self.peek_token())
                    }
                }
                // Can only happen if `get_next_precedence` got out of sync with this function
                _ => parser_err!(format!("No infix parser for token {:?}", tok.token)),
            }
        } else if Token::DoubleColon == tok {
            self.parse_pg_cast(expr)
        } else if Token::ExclamationMark == tok {
            // PostgreSQL factorial operation
            Ok(Expr::UnaryOp {
                op: UnaryOperator::PGPostfixFactorial,
                expr: Box::new(expr),
            })
        } else if Token::LBracket == tok {
            if dialect_of!(self is PostgreSqlDialect | GenericDialect) {
                // parse index
                return self.parse_array_index(expr);
            }
            self.parse_map_access(expr)
        } else if Token::Colon == tok {
            Ok(Expr::JsonAccess {
                left: Box::new(expr),
                operator: JsonOperator::Colon,
                right: Box::new(Expr::Value(self.parse_value()?)),
            })
        } else if Token::Arrow == tok
            || Token::LongArrow == tok
            || Token::HashArrow == tok
            || Token::HashLongArrow == tok
            || Token::AtArrow == tok
            || Token::ArrowAt == tok
            || Token::HashMinus == tok
            || Token::AtQuestion == tok
            || Token::AtAt == tok
        {
            let operator = match tok.token {
                Token::Arrow => JsonOperator::Arrow,
                Token::LongArrow => JsonOperator::LongArrow,
                Token::HashArrow => JsonOperator::HashArrow,
                Token::HashLongArrow => JsonOperator::HashLongArrow,
                Token::AtArrow => JsonOperator::AtArrow,
                Token::ArrowAt => JsonOperator::ArrowAt,
                Token::HashMinus => JsonOperator::HashMinus,
                Token::AtQuestion => JsonOperator::AtQuestion,
                Token::AtAt => JsonOperator::AtAt,
                _ => unreachable!(),
            };
            Ok(Expr::JsonAccess {
                left: Box::new(expr),
                operator,
                right: Box::new(self.parse_expr()?),
            })
        } else {
            // Can only happen if `get_next_precedence` got out of sync with this function
            parser_err!(format!("No infix parser for token {:?}", tok.token))
        }
    }

    /// parse the ESCAPE CHAR portion of LIKE, ILIKE, and SIMILAR TO
    pub fn parse_escape_char(&mut self) -> Result<Option<char>, ParserError> {
        if self.parse_keyword(Keyword::ESCAPE) {
            Ok(Some(self.parse_literal_char()?))
        } else {
            Ok(None)
        }
    }

    pub fn parse_array_index(&mut self, expr: Expr) -> Result<Expr, ParserError> {
        let index = self.parse_expr()?;
        self.expect_token(&Token::RBracket)?;
        let mut indexes: Vec<Expr> = vec![index];
        while self.consume_token(&Token::LBracket) {
            let index = self.parse_expr()?;
            self.expect_token(&Token::RBracket)?;
            indexes.push(index);
        }
        Ok(Expr::ArrayIndex {
            obj: Box::new(expr),
            indexes,
        })
    }

    pub fn parse_map_access(&mut self, expr: Expr) -> Result<Expr, ParserError> {
        let key = self.parse_map_key()?;
        let tok = self.consume_token(&Token::RBracket);
        debug!("Tok: {}", tok);
        let mut key_parts: Vec<Expr> = vec![key];
        while self.consume_token(&Token::LBracket) {
            let key = self.parse_map_key()?;
            let tok = self.consume_token(&Token::RBracket);
            debug!("Tok: {}", tok);
            key_parts.push(key);
        }
        match expr {
            e @ Expr::Identifier(_) | e @ Expr::CompoundIdentifier(_) => Ok(Expr::MapAccess {
                column: Box::new(e),
                keys: key_parts,
            }),
            _ => Ok(expr),
        }
    }

    /// Parses the parens following the `[ NOT ] IN` operator
    pub fn parse_in(&mut self, expr: Expr, negated: bool) -> Result<Expr, ParserError> {
        // BigQuery allows `IN UNNEST(array_expression)`
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators#in_operators
        if self.parse_keyword(Keyword::UNNEST) {
            self.expect_token(&Token::LParen)?;
            let array_expr = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            return Ok(Expr::InUnnest {
                expr: Box::new(expr),
                array_expr: Box::new(array_expr),
                negated,
            });
        }
        self.expect_token(&Token::LParen)?;
        let in_op = if self.parse_keyword(Keyword::SELECT) || self.parse_keyword(Keyword::WITH) {
            self.prev_token();
            Expr::InSubquery {
                expr: Box::new(expr),
                subquery: Box::new(self.parse_query(None)?),
                negated,
            }
        } else {
            Expr::InList {
                expr: Box::new(expr),
                list: self.parse_comma_separated(Parser::parse_expr)?,
                negated,
            }
        };
        self.expect_token(&Token::RParen)?;
        Ok(in_op)
    }

    /// Parses `BETWEEN <low> AND <high>`, assuming the `BETWEEN` keyword was already consumed
    pub fn parse_between(&mut self, expr: Expr, negated: bool) -> Result<Expr, ParserError> {
        // Stop parsing subexpressions for <low> and <high> on tokens with
        // precedence lower than that of `BETWEEN`, such as `AND`, `IS`, etc.
        let low = self.parse_subexpr(Self::BETWEEN_PREC)?;
        self.expect_keyword(Keyword::AND)?;
        let high = self.parse_subexpr(Self::BETWEEN_PREC)?;
        Ok(Expr::Between {
            expr: Box::new(expr),
            negated,
            low: Box::new(low),
            high: Box::new(high),
        })
    }

    /// Parse a postgresql casting style which is in the form of `expr::datatype`
    pub fn parse_pg_cast(&mut self, expr: Expr) -> Result<Expr, ParserError> {
        Ok(Expr::Cast {
            expr: Box::new(expr),
            data_type: self.parse_data_type()?,
        })
    }

    // use https://www.postgresql.org/docs/7.0/operators.htm#AEN2026 as a reference
    const PLUS_MINUS_PREC: u8 = 30;
    const XOR_PREC: u8 = 24;
    const TIME_ZONE_PREC: u8 = 20;
    const BETWEEN_PREC: u8 = 20;
    const LIKE_PREC: u8 = 19;
    const IS_PREC: u8 = 17;
    const UNARY_NOT_PREC: u8 = 15;
    const AND_PREC: u8 = 10;
    const OR_PREC: u8 = 5;

    /// Get the precedence of the next token
    pub fn get_next_precedence(&self) -> Result<u8, ParserError> {
        // allow the dialect to override precedence logic
        if let Some(precedence) = self.dialect.get_next_precedence(self) {
            return precedence;
        }

        let token = self.peek_token();
        debug!("get_next_precedence() {:?}", token);
        let token_0 = self.peek_nth_token(0);
        let token_1 = self.peek_nth_token(1);
        let token_2 = self.peek_nth_token(2);
        debug!("0: {token_0} 1: {token_1} 2: {token_2}");
        match token.token {
            Token::Word(w) if w.keyword == Keyword::OR => Ok(Self::OR_PREC),
            Token::Word(w) if w.keyword == Keyword::AND => Ok(Self::AND_PREC),
            Token::Word(w) if w.keyword == Keyword::XOR => Ok(Self::XOR_PREC),

            Token::Word(w) if w.keyword == Keyword::AT => {
                match (self.peek_nth_token(1).token, self.peek_nth_token(2).token) {
                    (Token::Word(w), Token::Word(w2))
                        if w.keyword == Keyword::TIME && w2.keyword == Keyword::ZONE =>
                    {
                        Ok(Self::TIME_ZONE_PREC)
                    }
                    _ => Ok(0),
                }
            }

            Token::Word(w) if w.keyword == Keyword::NOT => match self.peek_nth_token(1).token {
                // The precedence of NOT varies depending on keyword that
                // follows it. If it is followed by IN, BETWEEN, or LIKE,
                // it takes on the precedence of those tokens. Otherwise it
                // is not an infix operator, and therefore has zero
                // precedence.
                Token::Word(w) if w.keyword == Keyword::IN => Ok(Self::BETWEEN_PREC),
                Token::Word(w) if w.keyword == Keyword::BETWEEN => Ok(Self::BETWEEN_PREC),
                Token::Word(w) if w.keyword == Keyword::LIKE => Ok(Self::LIKE_PREC),
                Token::Word(w) if w.keyword == Keyword::ILIKE => Ok(Self::LIKE_PREC),
                Token::Word(w) if w.keyword == Keyword::SIMILAR => Ok(Self::LIKE_PREC),
                _ => Ok(0),
            },
            Token::Word(w) if w.keyword == Keyword::IS => Ok(Self::IS_PREC),
            Token::Word(w) if w.keyword == Keyword::IN => Ok(Self::BETWEEN_PREC),
            Token::Word(w) if w.keyword == Keyword::BETWEEN => Ok(Self::BETWEEN_PREC),
            Token::Word(w) if w.keyword == Keyword::LIKE => Ok(Self::LIKE_PREC),
            Token::Word(w) if w.keyword == Keyword::ILIKE => Ok(Self::LIKE_PREC),
            Token::Word(w) if w.keyword == Keyword::SIMILAR => Ok(Self::LIKE_PREC),
            Token::Word(w) if w.keyword == Keyword::OPERATOR => Ok(Self::BETWEEN_PREC),
            Token::Eq
            | Token::Lt
            | Token::LtEq
            | Token::Neq
            | Token::Gt
            | Token::GtEq
            | Token::DoubleEq
            | Token::Tilde
            | Token::TildeAsterisk
            | Token::ExclamationMarkTilde
            | Token::ExclamationMarkTildeAsterisk
            | Token::Spaceship => Ok(20),
            Token::Pipe => Ok(21),
            Token::Caret | Token::Sharp | Token::ShiftRight | Token::ShiftLeft => Ok(22),
            Token::Ampersand => Ok(23),
            Token::Plus | Token::Minus => Ok(Self::PLUS_MINUS_PREC),
            Token::Mul | Token::Div | Token::Mod | Token::StringConcat => Ok(40),
            Token::DoubleColon => Ok(50),
            Token::Colon => Ok(50),
            Token::ExclamationMark => Ok(50),
            Token::LBracket
            | Token::LongArrow
            | Token::Arrow
            | Token::HashArrow
            | Token::HashLongArrow
            | Token::AtArrow
            | Token::ArrowAt
            | Token::HashMinus
            | Token::AtQuestion
            | Token::AtAt => Ok(50),
            _ => Ok(0),
        }
    }

    /// Return the first non-whitespace token that has not yet been processed
    /// (or None if reached end-of-file)
    pub fn peek_token(&self) -> TokenWithLocation {
        self.peek_nth_token(0)
    }

    /// Return nth non-whitespace token that has not yet been processed
    pub fn peek_nth_token(&self, mut n: usize) -> TokenWithLocation {
        let mut index = self.index;
        loop {
            index += 1;
            match self.tokens.get(index - 1) {
                Some(TokenWithLocation {
                    token: Token::Whitespace(_),
                    location: _,
                }) => continue,
                non_whitespace => {
                    if n == 0 {
                        return non_whitespace.cloned().unwrap_or(TokenWithLocation {
                            token: Token::EOF,
                            location: Location { line: 0, column: 0 },
                        });
                    }
                    n -= 1;
                }
            }
        }
    }

    /// Return the first non-whitespace token that has not yet been processed
    /// (or None if reached end-of-file) and mark it as processed. OK to call
    /// repeatedly after reaching EOF.
    pub fn next_token(&mut self) -> TokenWithLocation {
        loop {
            self.index += 1;
            match self.tokens.get(self.index - 1) {
                Some(TokenWithLocation {
                    token: Token::Whitespace(_),
                    location: _,
                }) => continue,
                token => {
                    return token
                        .cloned()
                        .unwrap_or_else(|| TokenWithLocation::wrap(Token::EOF))
                }
            }
        }
    }

    /// Return the first unprocessed token, possibly whitespace.
    pub fn next_token_no_skip(&mut self) -> Option<&TokenWithLocation> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    /// Push back the last one non-whitespace token. Must be called after
    /// `next_token()`, otherwise might panic. OK to call after
    /// `next_token()` indicates an EOF.
    pub fn prev_token(&mut self) {
        loop {
            assert!(self.index > 0);
            self.index -= 1;
            if let Some(TokenWithLocation {
                token: Token::Whitespace(_),
                location: _,
            }) = self.tokens.get(self.index)
            {
                continue;
            }
            return;
        }
    }

    /// Report unexpected token
    pub fn expected<T>(&self, expected: &str, found: TokenWithLocation) -> Result<T, ParserError> {
        parser_err!(format!("Expected {expected}, found: {found}"))
    }

    /// Look for an expected keyword and consume it if it exists
    #[must_use]
    pub fn parse_keyword(&mut self, expected: Keyword) -> bool {
        match self.peek_token().token {
            Token::Word(w) if expected == w.keyword => {
                self.next_token();
                true
            }
            _ => false,
        }
    }

    /// Look for an expected sequence of keywords and consume them if they exist
    #[must_use]
    pub fn parse_keywords(&mut self, keywords: &[Keyword]) -> bool {
        let index = self.index;
        for &keyword in keywords {
            if !self.parse_keyword(keyword) {
                // println!("parse_keywords aborting .. did not find {:?}", keyword);
                // reset index and return immediately
                self.index = index;
                return false;
            }
        }
        true
    }

    /// Look for one of the given keywords and return the one that matches.
    #[must_use]
    pub fn parse_one_of_keywords(&mut self, keywords: &[Keyword]) -> Option<Keyword> {
        match self.peek_token().token {
            Token::Word(w) => {
                keywords
                    .iter()
                    .find(|keyword| **keyword == w.keyword)
                    .map(|keyword| {
                        self.next_token();
                        *keyword
                    })
            }
            _ => None,
        }
    }

    /// Bail out if the current token is not one of the expected keywords, or consume it if it is
    pub fn expect_one_of_keywords(&mut self, keywords: &[Keyword]) -> Result<Keyword, ParserError> {
        if let Some(keyword) = self.parse_one_of_keywords(keywords) {
            Ok(keyword)
        } else {
            let keywords: Vec<String> = keywords.iter().map(|x| format!("{x:?}")).collect();
            self.expected(
                &format!("one of {}", keywords.join(" or ")),
                self.peek_token(),
            )
        }
    }

    /// Bail out if the current token is not an expected keyword, or consume it if it is
    pub fn expect_keyword(&mut self, expected: Keyword) -> Result<(), ParserError> {
        if self.parse_keyword(expected) {
            Ok(())
        } else {
            self.expected(format!("{:?}", &expected).as_str(), self.peek_token())
        }
    }

    /// Bail out if the following tokens are not the expected sequence of
    /// keywords, or consume them if they are.
    pub fn expect_keywords(&mut self, expected: &[Keyword]) -> Result<(), ParserError> {
        for &kw in expected {
            self.expect_keyword(kw)?;
        }
        Ok(())
    }

    /// Consume the next token if it matches the expected token, otherwise return false
    #[must_use]
    pub fn consume_token(&mut self, expected: &Token) -> bool {
        if self.peek_token() == *expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Bail out if the current token is not an expected keyword, or consume it if it is
    pub fn expect_token(&mut self, expected: &Token) -> Result<(), ParserError> {
        if self.consume_token(expected) {
            Ok(())
        } else {
            self.expected(&expected.to_string(), self.peek_token())
        }
    }

    /// Parse a comma-separated list of 1+ SelectItem
    pub fn parse_projection(&mut self) -> Result<Vec<SelectItem>, ParserError> {
        // BigQuery allows trailing commas, but only in project lists
        // e.g. `SELECT 1, 2, FROM t`
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical#trailing_commas
        //
        // This pattern could be captured better with RAII type semantics, but it's quite a bit of
        // code to add for just one case, so we'll just do it manually here.
        let old_value = self.options.trailing_commas;
        self.options.trailing_commas |= dialect_of!(self is BigQueryDialect);

        let ret = self.parse_comma_separated(|p| p.parse_select_item());
        self.options.trailing_commas = old_value;

        ret
    }

    /// Parse a comma-separated list of 1+ items accepted by `F`
    pub fn parse_comma_separated<T, F>(&mut self, mut f: F) -> Result<Vec<T>, ParserError>
    where
        F: FnMut(&mut Parser<'a>) -> Result<T, ParserError>,
    {
        let mut values = vec![];
        loop {
            values.push(f(self)?);
            if !self.consume_token(&Token::Comma) {
                break;
            } else if self.options.trailing_commas {
                match self.peek_token().token {
                    Token::Word(kw)
                        if keywords::RESERVED_FOR_COLUMN_ALIAS
                            .iter()
                            .any(|d| kw.keyword == *d) =>
                    {
                        break;
                    }
                    Token::RParen
                    | Token::SemiColon
                    | Token::EOF
                    | Token::RBracket
                    | Token::RBrace => break,
                    _ => continue,
                }
            }
        }
        Ok(values)
    }

    /// Run a parser method `f`, reverting back to the current position
    /// if unsuccessful.
    #[must_use]
    fn maybe_parse<T, F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Parser) -> Result<T, ParserError>,
    {
        let index = self.index;
        if let Ok(t) = f(self) {
            Some(t)
        } else {
            self.index = index;
            None
        }
    }

    /// Parse either `ALL` or `DISTINCT`. Returns `true` if `DISTINCT` is parsed and results in a
    /// `ParserError` if both `ALL` and `DISTINCT` are fround.
    pub fn parse_all_or_distinct(&mut self) -> Result<bool, ParserError> {
        let all = self.parse_keyword(Keyword::ALL);
        let distinct = self.parse_keyword(Keyword::DISTINCT);
        if all && distinct {
            parser_err!("Cannot specify both ALL and DISTINCT".to_string())
        } else {
            Ok(distinct)
        }
    }

    /// Parse 'AS' before as query,such as `WITH XXX AS SELECT XXX` oer `CACHE TABLE AS SELECT XXX`
    pub fn parse_as_query(&mut self) -> Result<(bool, Query), ParserError> {
        match self.peek_token().token {
            Token::Word(word) => match word.keyword {
                Keyword::AS => {
                    self.next_token();
                    Ok((true, self.parse_query(None)?))
                }
                _ => Ok((false, self.parse_query(None)?)),
            },
            _ => self.expected("a QUERY statement", self.peek_token()),
        }
    }

    fn parse_literal_char(&mut self) -> Result<char, ParserError> {
        let s = self.parse_literal_string()?;
        if s.len() != 1 {
            return parser_err!(format!("Expect a char, found {s:?}"));
        }
        Ok(s.chars().next().unwrap())
    }

    /// Parse a literal value (numbers, strings, date/time, booleans)
    pub fn parse_value(&mut self) -> Result<Value, ParserError> {
        let next_token = self.next_token();
        let location = next_token.location;
        match next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::TRUE => Ok(Value::Boolean(true)),
                Keyword::FALSE => Ok(Value::Boolean(false)),
                Keyword::NULL => Ok(Value::Null),
                Keyword::NoKeyword if w.quote_style.is_some() => match w.quote_style {
                    Some('"') => Ok(Value::DoubleQuotedString(w.value)),
                    Some('\'') => Ok(Value::SingleQuotedString(w.value)),
                    _ => self.expected(
                        "A value?",
                        TokenWithLocation {
                            token: Token::Word(w),
                            location,
                        },
                    )?,
                },
                // Case when Snowflake Semi-structured data like key:value
                Keyword::NoKeyword | Keyword::LOCATION | Keyword::TYPE if dialect_of!(self is SnowflakeDialect | GenericDialect) => {
                    Ok(Value::UnQuotedString(w.value))
                }
                _ => self.expected(
                    "a concrete value",
                    TokenWithLocation {
                        token: Token::Word(w),
                        location,
                    },
                ),
            },
            // The call to n.parse() returns a bigdecimal when the
            // bigdecimal feature is enabled, and is otherwise a no-op
            // (i.e., it returns the input string).
            Token::Number(ref n, l) => match n.parse() {
                Ok(n) => Ok(Value::Number(n, l)),
                Err(e) => parser_err!(format!("Could not parse '{n}' as number: {e}")),
            },
            Token::SingleQuotedString(ref s) => Ok(Value::SingleQuotedString(s.to_string())),
            Token::DoubleQuotedString(ref s) => Ok(Value::DoubleQuotedString(s.to_string())),
            Token::DollarQuotedString(ref s) => Ok(Value::DollarQuotedString(s.clone())),
            Token::SingleQuotedByteStringLiteral(ref s) => {
                Ok(Value::SingleQuotedByteStringLiteral(s.clone()))
            }
            Token::DoubleQuotedByteStringLiteral(ref s) => {
                Ok(Value::DoubleQuotedByteStringLiteral(s.clone()))
            }
            Token::RawStringLiteral(ref s) => Ok(Value::RawStringLiteral(s.clone())),
            Token::NationalStringLiteral(ref s) => Ok(Value::NationalStringLiteral(s.to_string())),
            Token::EscapedStringLiteral(ref s) => Ok(Value::EscapedStringLiteral(s.to_string())),
            Token::HexStringLiteral(ref s) => Ok(Value::HexStringLiteral(s.to_string())),
            Token::Placeholder(ref s) => Ok(Value::Placeholder(s.to_string())),
            tok @ Token::Colon | tok @ Token::AtSign => {
                let ident = self.parse_identifier()?;
                let placeholder = tok.to_string() + &ident.value;
                Ok(Value::Placeholder(placeholder))
            }
            unexpected => self.expected(
                "a value",
                TokenWithLocation {
                    token: unexpected,
                    location,
                },
            ),
        }
    }

    pub fn parse_number_value(&mut self) -> Result<Value, ParserError> {
        match self.parse_value()? {
            v @ Value::Number(_, _) => Ok(v),
            v @ Value::Placeholder(_) => Ok(v),
            _ => {
                self.prev_token();
                self.expected("literal number", self.peek_token())
            }
        }
    }

    fn parse_introduced_string_value(&mut self) -> Result<Value, ParserError> {
        let next_token = self.next_token();
        let location = next_token.location;
        match next_token.token {
            Token::SingleQuotedString(ref s) => Ok(Value::SingleQuotedString(s.to_string())),
            Token::DoubleQuotedString(ref s) => Ok(Value::DoubleQuotedString(s.to_string())),
            Token::HexStringLiteral(ref s) => Ok(Value::HexStringLiteral(s.to_string())),
            unexpected => self.expected(
                "a string value",
                TokenWithLocation {
                    token: unexpected,
                    location,
                },
            ),
        }
    }

    /// Parse an unsigned literal integer/long
    pub fn parse_literal_uint(&mut self) -> Result<u64, ParserError> {
        let next_token = self.next_token();
        match next_token.token {
            Token::Number(s, _) => s.parse::<u64>().map_err(|e| {
                ParserError::ParserError(format!("Could not parse '{s}' as u64: {e}"))
            }),
            _ => self.expected("literal int", next_token),
        }
    }

    /// Parse a literal string
    pub fn parse_literal_string(&mut self) -> Result<String, ParserError> {
        let next_token = self.next_token();
        match next_token.token {
            Token::Word(Word { value, keyword, .. }) if keyword == Keyword::NoKeyword => Ok(value),
            Token::SingleQuotedString(s) => Ok(s),
            Token::DoubleQuotedString(s) => Ok(s),
            Token::EscapedStringLiteral(s) if dialect_of!(self is PostgreSqlDialect | GenericDialect) => {
                Ok(s)
            }
            _ => self.expected("literal string", next_token),
        }
    }

    /// Parse a map key string
    pub fn parse_map_key(&mut self) -> Result<Expr, ParserError> {
        let next_token = self.next_token();
        match next_token.token {
            // handle bigquery offset subscript operator which overlaps with OFFSET operator
            Token::Word(Word { value, keyword, .. })
                if (dialect_of!(self is BigQueryDialect) && keyword == Keyword::OFFSET) =>
            {
                self.parse_function(ObjectName(vec![Ident::new(value)]))
            }
            Token::Word(Word { value, keyword, .. }) if (keyword == Keyword::NoKeyword) => {
                if self.peek_token() == Token::LParen {
                    return self.parse_function(ObjectName(vec![Ident::new(value)]));
                }
                Ok(Expr::Value(Value::SingleQuotedString(value)))
            }
            Token::SingleQuotedString(s) => Ok(Expr::Value(Value::SingleQuotedString(s))),
            #[cfg(not(feature = "bigdecimal"))]
            Token::Number(s, _) => Ok(Expr::Value(Value::Number(s, false))),
            #[cfg(feature = "bigdecimal")]
            Token::Number(s, _) => Ok(Expr::Value(Value::Number(s.parse().unwrap(), false))),
            _ => self.expected("literal string, number or function", next_token),
        }
    }

    /// Parse a SQL datatype (in the context of a CREATE TABLE statement for example)
    pub fn parse_data_type(&mut self) -> Result<DataType, ParserError> {
        let next_token = self.next_token();
        let mut data = match next_token.token {
            Token::Word(w) => match w.keyword {
                Keyword::BOOLEAN => Ok(DataType::Boolean),
                Keyword::FLOAT => Ok(DataType::Float(self.parse_optional_precision()?)),
                Keyword::REAL => Ok(DataType::Real),
                Keyword::DOUBLE => {
                    if self.parse_keyword(Keyword::PRECISION) {
                        Ok(DataType::DoublePrecision)
                    } else {
                        Ok(DataType::Double)
                    }
                }
                Keyword::TINYINT => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedTinyInt(optional_precision?))
                    } else {
                        Ok(DataType::TinyInt(optional_precision?))
                    }
                }
                Keyword::SMALLINT => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedSmallInt(optional_precision?))
                    } else {
                        Ok(DataType::SmallInt(optional_precision?))
                    }
                }
                Keyword::MEDIUMINT => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedMediumInt(optional_precision?))
                    } else {
                        Ok(DataType::MediumInt(optional_precision?))
                    }
                }
                Keyword::INT => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedInt(optional_precision?))
                    } else {
                        Ok(DataType::Int(optional_precision?))
                    }
                }
                Keyword::INTEGER => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedInteger(optional_precision?))
                    } else {
                        Ok(DataType::Integer(optional_precision?))
                    }
                }
                Keyword::BIGINT => {
                    let optional_precision = self.parse_optional_precision();
                    if self.parse_keyword(Keyword::UNSIGNED) {
                        Ok(DataType::UnsignedBigInt(optional_precision?))
                    } else {
                        Ok(DataType::BigInt(optional_precision?))
                    }
                }
                Keyword::VARCHAR => Ok(DataType::Varchar(self.parse_optional_character_length()?)),
                Keyword::NVARCHAR => Ok(DataType::Nvarchar(self.parse_optional_precision()?)),
                Keyword::CHARACTER => {
                    if self.parse_keyword(Keyword::VARYING) {
                        Ok(DataType::CharacterVarying(
                            self.parse_optional_character_length()?,
                        ))
                    } else if self.parse_keywords(&[Keyword::LARGE, Keyword::OBJECT]) {
                        Ok(DataType::CharacterLargeObject(
                            self.parse_optional_precision()?,
                        ))
                    } else {
                        Ok(DataType::Character(self.parse_optional_character_length()?))
                    }
                }
                Keyword::CHAR => {
                    if self.parse_keyword(Keyword::VARYING) {
                        Ok(DataType::CharVarying(
                            self.parse_optional_character_length()?,
                        ))
                    } else if self.parse_keywords(&[Keyword::LARGE, Keyword::OBJECT]) {
                        Ok(DataType::CharLargeObject(self.parse_optional_precision()?))
                    } else {
                        Ok(DataType::Char(self.parse_optional_character_length()?))
                    }
                }
                Keyword::CLOB => Ok(DataType::Clob(self.parse_optional_precision()?)),
                Keyword::BINARY => Ok(DataType::Binary(self.parse_optional_precision()?)),
                Keyword::VARBINARY => Ok(DataType::Varbinary(self.parse_optional_precision()?)),
                Keyword::BLOB => Ok(DataType::Blob(self.parse_optional_precision()?)),
                Keyword::UUID => Ok(DataType::Uuid),
                Keyword::DATE => Ok(DataType::Date),
                Keyword::DATETIME => Ok(DataType::Datetime(self.parse_optional_precision()?)),
                Keyword::TIMESTAMP => {
                    let precision = self.parse_optional_precision()?;
                    let tz = if self.parse_keyword(Keyword::WITH) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                        TimezoneInfo::WithTimeZone
                    } else if self.parse_keyword(Keyword::WITHOUT) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                        TimezoneInfo::WithoutTimeZone
                    } else {
                        TimezoneInfo::None
                    };
                    Ok(DataType::Timestamp(precision, tz))
                }
                Keyword::TIMESTAMPTZ => Ok(DataType::Timestamp(
                    self.parse_optional_precision()?,
                    TimezoneInfo::Tz,
                )),
                Keyword::TIME => {
                    let precision = self.parse_optional_precision()?;
                    let tz = if self.parse_keyword(Keyword::WITH) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                        TimezoneInfo::WithTimeZone
                    } else if self.parse_keyword(Keyword::WITHOUT) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                        TimezoneInfo::WithoutTimeZone
                    } else {
                        TimezoneInfo::None
                    };
                    Ok(DataType::Time(precision, tz))
                }
                Keyword::TIMETZ => Ok(DataType::Time(
                    self.parse_optional_precision()?,
                    TimezoneInfo::Tz,
                )),
                // Interval types can be followed by a complicated interval
                // qualifier that we don't currently support. See
                // parse_interval for a taste.
                Keyword::INTERVAL => Ok(DataType::Interval),
                Keyword::JSON => Ok(DataType::JSON),
                Keyword::REGCLASS => Ok(DataType::Regclass),
                Keyword::STRING => Ok(DataType::String),
                Keyword::TEXT => Ok(DataType::Text),
                Keyword::BYTEA => Ok(DataType::Bytea),
                Keyword::NUMERIC => Ok(DataType::Numeric(
                    self.parse_exact_number_optional_precision_scale()?,
                )),
                Keyword::DECIMAL => Ok(DataType::Decimal(
                    self.parse_exact_number_optional_precision_scale()?,
                )),
                Keyword::DEC => Ok(DataType::Dec(
                    self.parse_exact_number_optional_precision_scale()?,
                )),
                Keyword::BIGNUMERIC => Ok(DataType::BigNumeric(
                    self.parse_exact_number_optional_precision_scale()?,
                )),
                Keyword::BIGDECIMAL => Ok(DataType::BigDecimal(
                    self.parse_exact_number_optional_precision_scale()?,
                )),
                Keyword::ENUM => Ok(DataType::Enum(self.parse_string_values()?)),
                Keyword::SET => Ok(DataType::Set(self.parse_string_values()?)),
                Keyword::ARRAY => {
                    if dialect_of!(self is SnowflakeDialect) {
                        Ok(DataType::Array(None))
                    } else {
                        // Hive array syntax. Note that nesting arrays - or other Hive syntax
                        // that ends with > will fail due to "C++" problem - >> is parsed as
                        // Token::ShiftRight
                        self.expect_token(&Token::Lt)?;
                        let inside_type = self.parse_data_type()?;
                        self.expect_token(&Token::Gt)?;
                        Ok(DataType::Array(Some(Box::new(inside_type))))
                    }
                }
                _ => {
                    self.prev_token();
                    let type_name = self.parse_object_name()?;
                    if let Some(modifiers) = self.parse_optional_type_modifiers()? {
                        Ok(DataType::Custom(type_name, modifiers))
                    } else {
                        Ok(DataType::Custom(type_name, vec![]))
                    }
                }
            },
            _ => self.expected("a data type name", next_token),
        }?;

        // Parse array data types. Note: this is postgresql-specific and different from
        // Keyword::ARRAY syntax from above
        while self.consume_token(&Token::LBracket) {
            self.expect_token(&Token::RBracket)?;
            data = DataType::Array(Some(Box::new(data)))
        }
        Ok(data)
    }

    pub fn parse_string_values(&mut self) -> Result<Vec<String>, ParserError> {
        self.expect_token(&Token::LParen)?;
        let mut values = Vec::new();
        loop {
            let next_token = self.next_token();
            match next_token.token {
                Token::SingleQuotedString(value) => values.push(value),
                _ => self.expected("a string", next_token)?,
            }
            let next_token = self.next_token();
            match next_token.token {
                Token::Comma => (),
                Token::RParen => break,
                _ => self.expected(", or }", next_token)?,
            }
        }
        Ok(values)
    }

    /// Strictly parse `identifier AS identifier`
    pub fn parse_identifier_with_alias(&mut self) -> Result<IdentWithAlias, ParserError> {
        let ident = self.parse_identifier()?;
        self.expect_keyword(Keyword::AS)?;
        let alias = self.parse_identifier()?;
        Ok(IdentWithAlias { ident, alias })
    }

    /// Parse `AS identifier` (or simply `identifier` if it's not a reserved keyword)
    /// Some examples with aliases: `SELECT 1 foo`, `SELECT COUNT(*) AS cnt`,
    /// `SELECT ... FROM t1 foo, t2 bar`, `SELECT ... FROM (...) AS bar`
    pub fn parse_optional_alias(
        &mut self,
        reserved_kwds: &[Keyword],
    ) -> Result<Option<Ident>, ParserError> {
        let after_as = self.parse_keyword(Keyword::AS);
        let next_token = self.next_token();
        match next_token.token {
            // Accept any identifier after `AS` (though many dialects have restrictions on
            // keywords that may appear here). If there's no `AS`: don't parse keywords,
            // which may start a construct allowed in this position, to be parsed as aliases.
            // (For example, in `FROM t1 JOIN` the `JOIN` will always be parsed as a keyword,
            // not an alias.)
            Token::Word(w) if after_as || !reserved_kwds.contains(&w.keyword) => {
                Ok(Some(w.to_ident()))
            }
            // MSSQL supports single-quoted strings as aliases for columns
            // We accept them as table aliases too, although MSSQL does not.
            //
            // Note, that this conflicts with an obscure rule from the SQL
            // standard, which we don't implement:
            // https://crate.io/docs/sql-99/en/latest/chapters/07.html#character-string-literal-s
            //    "[Obscure Rule] SQL allows you to break a long <character
            //    string literal> up into two or more smaller <character string
            //    literal>s, split by a <separator> that includes a newline
            //    character. When it sees such a <literal>, your DBMS will
            //    ignore the <separator> and treat the multiple strings as
            //    a single <literal>."
            Token::SingleQuotedString(s) => Ok(Some(Ident::with_quote('\'', s))),
            // Support for MySql dialect double quoted string, `AS "HOUR"` for example
            Token::DoubleQuotedString(s) => Ok(Some(Ident::with_quote('\"', s))),
            _ => {
                if after_as {
                    return self.expected("an identifier after AS", next_token);
                }
                self.prev_token();
                Ok(None) // no alias found
            }
        }
    }

    /// Parse `AS identifier` when the AS is describing a table-valued object,
    /// like in `... FROM generate_series(1, 10) AS t (col)`. In this case
    /// the alias is allowed to optionally name the columns in the table, in
    /// addition to the table itself.
    pub fn parse_optional_table_alias(
        &mut self,
        reserved_kwds: &[Keyword],
    ) -> Result<Option<TableAlias>, ParserError> {
        match self.parse_optional_alias(reserved_kwds)? {
            Some(name) => {
                let columns = self.parse_parenthesized_column_list(Optional, false)?;
                Ok(Some(TableAlias { name, columns }))
            }
            None => Ok(None),
        }
    }

    /// Parse a possibly qualified, possibly quoted identifier, e.g.
    /// `foo` or `myschema."table"
    pub fn parse_object_name(&mut self) -> Result<ObjectName, ParserError> {
        let mut idents = vec![];
        loop {
            idents.push(self.parse_identifier()?);
            if !self.consume_token(&Token::Period) {
                break;
            }
        }
        Ok(ObjectName(idents))
    }

    /// Parse identifiers
    pub fn parse_identifiers(&mut self) -> Result<Vec<Ident>, ParserError> {
        let mut idents = vec![];
        loop {
            match self.peek_token().token {
                Token::Word(w) => {
                    idents.push(w.to_ident());
                }
                Token::EOF | Token::Eq => break,
                _ => {}
            }
            self.next_token();
        }
        Ok(idents)
    }

    /// Parse a simple one-word identifier (possibly quoted, possibly a keyword)
    pub fn parse_identifier(&mut self) -> Result<Ident, ParserError> {
        let next_token = self.next_token();
        match next_token.token {
            Token::Word(w) => Ok(w.to_ident()),
            Token::SingleQuotedString(s) => Ok(Ident::with_quote('\'', s)),
            Token::DoubleQuotedString(s) => Ok(Ident::with_quote('\"', s)),
            _ => self.expected("identifier", next_token),
        }
    }

    /// Parse a parenthesized comma-separated list of unqualified, possibly quoted identifiers
    pub fn parse_parenthesized_column_list(
        &mut self,
        optional: IsOptional,
        allow_empty: bool,
    ) -> Result<Vec<Ident>, ParserError> {
        if self.consume_token(&Token::LParen) {
            if allow_empty && self.peek_token().token == Token::RParen {
                self.next_token();
                Ok(vec![])
            } else {
                let cols = self.parse_comma_separated(Parser::parse_identifier)?;
                self.expect_token(&Token::RParen)?;
                Ok(cols)
            }
        } else if optional == Optional {
            Ok(vec![])
        } else {
            self.expected("a list of columns in parentheses", self.peek_token())
        }
    }

    pub fn parse_precision(&mut self) -> Result<u64, ParserError> {
        self.expect_token(&Token::LParen)?;
        let n = self.parse_literal_uint()?;
        self.expect_token(&Token::RParen)?;
        Ok(n)
    }

    pub fn parse_optional_precision(&mut self) -> Result<Option<u64>, ParserError> {
        if self.consume_token(&Token::LParen) {
            let n = self.parse_literal_uint()?;
            self.expect_token(&Token::RParen)?;
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }

    pub fn parse_optional_character_length(
        &mut self,
    ) -> Result<Option<CharacterLength>, ParserError> {
        if self.consume_token(&Token::LParen) {
            let character_length = self.parse_character_length()?;
            self.expect_token(&Token::RParen)?;
            Ok(Some(character_length))
        } else {
            Ok(None)
        }
    }

    pub fn parse_character_length(&mut self) -> Result<CharacterLength, ParserError> {
        let length = self.parse_literal_uint()?;
        let unit = if self.parse_keyword(Keyword::CHARACTERS) {
            Some(CharLengthUnits::Characters)
        } else if self.parse_keyword(Keyword::OCTETS) {
            Some(CharLengthUnits::Octets)
        } else {
            None
        };

        Ok(CharacterLength { length, unit })
    }

    pub fn parse_optional_precision_scale(
        &mut self,
    ) -> Result<(Option<u64>, Option<u64>), ParserError> {
        if self.consume_token(&Token::LParen) {
            let n = self.parse_literal_uint()?;
            let scale = if self.consume_token(&Token::Comma) {
                Some(self.parse_literal_uint()?)
            } else {
                None
            };
            self.expect_token(&Token::RParen)?;
            Ok((Some(n), scale))
        } else {
            Ok((None, None))
        }
    }

    pub fn parse_exact_number_optional_precision_scale(
        &mut self,
    ) -> Result<ExactNumberInfo, ParserError> {
        if self.consume_token(&Token::LParen) {
            let precision = self.parse_literal_uint()?;
            let scale = if self.consume_token(&Token::Comma) {
                Some(self.parse_literal_uint()?)
            } else {
                None
            };

            self.expect_token(&Token::RParen)?;

            match scale {
                None => Ok(ExactNumberInfo::Precision(precision)),
                Some(scale) => Ok(ExactNumberInfo::PrecisionAndScale(precision, scale)),
            }
        } else {
            Ok(ExactNumberInfo::None)
        }
    }

    pub fn parse_optional_type_modifiers(&mut self) -> Result<Option<Vec<String>>, ParserError> {
        if self.consume_token(&Token::LParen) {
            let mut modifiers = Vec::new();
            loop {
                let next_token = self.next_token();
                match next_token.token {
                    Token::Word(w) => modifiers.push(w.to_string()),
                    Token::Number(n, _) => modifiers.push(n),
                    Token::SingleQuotedString(s) => modifiers.push(s),

                    Token::Comma => {
                        continue;
                    }
                    Token::RParen => {
                        break;
                    }
                    _ => self.expected("type modifiers", next_token)?,
                }
            }

            Ok(Some(modifiers))
        } else {
            Ok(None)
        }
    }

    // TODO: Maybe this is where DbtConfig needs to live?

    /// Parse a query expression, i.e. a `SELECT` statement optionally
    /// preceded with some `WITH` CTE declarations and optionally followed
    /// by `ORDER BY`. Unlike some other parse_... methods, this one doesn't
    /// expect the initial keyword to be already consumed
    pub fn parse_query(&mut self, config: Option<DbtConfig>) -> Result<Query, ParserError> {
        let _guard = self.recursion_counter.try_decrease()?;
        let with = if self.parse_keyword(Keyword::WITH) {
            Some(With {
                recursive: self.parse_keyword(Keyword::RECURSIVE),
                cte_tables: self.parse_comma_separated(Parser::parse_cte)?,
            })
        } else {
            None
        };

        let body = Box::new(self.parse_query_body(0)?);
        let order_by = if self.parse_keywords(&[Keyword::ORDER, Keyword::BY]) {
            self.parse_comma_separated(Parser::parse_order_by_expr)?
        } else {
            vec![]
        };

        let mut limit = None;
        let mut offset = None;

        for _x in 0..2 {
            if limit.is_none() && self.parse_keyword(Keyword::LIMIT) {
                limit = self.parse_limit()?
            }

            if offset.is_none() && self.parse_keyword(Keyword::OFFSET) {
                offset = Some(self.parse_offset()?)
            }

            if dialect_of!(self is GenericDialect)
                && limit.is_some()
                && offset.is_none()
                && self.consume_token(&Token::Comma)
            {
                // MySQL style LIMIT x,y => LIMIT y OFFSET x.
                // Check <https://dev.mysql.com/doc/refman/8.0/en/select.html> for more details.
                offset = Some(Offset {
                    value: limit.unwrap(),
                    rows: OffsetRows::None,
                });
                limit = Some(self.parse_expr()?);
            }
        }

        Ok(Query {
            config: config,
            with,
            body,
            order_by,
            limit,
            offset,
            jinja_variables: vec![],
        })
    }

    /// Parse a CTE (`alias [( col1, col2, ... )] AS (subquery)`)
    pub fn parse_cte(&mut self) -> Result<Cte, ParserError> {
        let name = self.parse_identifier()?;

        let mut cte = if self.parse_keyword(Keyword::AS) {
            self.expect_token(&Token::LParen)?;
            let query = Box::new(self.parse_query(None)?);
            self.expect_token(&Token::RParen)?;
            let alias = TableAlias {
                name,
                columns: vec![],
            };
            Cte {
                alias,
                query,
                from: None,
            }
        } else {
            let columns = self.parse_parenthesized_column_list(Optional, false)?;
            self.expect_keyword(Keyword::AS)?;
            self.expect_token(&Token::LParen)?;
            let query = Box::new(self.parse_query(None)?);
            self.expect_token(&Token::RParen)?;
            let alias = TableAlias { name, columns };
            Cte {
                alias,
                query,
                from: None,
            }
        };
        if self.parse_keyword(Keyword::FROM) {
            cte.from = Some(self.parse_identifier()?);
        }
        Ok(cte)
    }

    /// Parse a "query body", which is an expression with roughly the
    /// following grammar:
    /// ```sql
    ///   query_body ::= restricted_select | '(' subquery ')' | set_operation
    ///   restricted_select ::= 'SELECT' [expr_list] [ from ] [ where ] [ groupby_having ]
    ///   subquery ::= query_body [ order_by_limit ]
    ///   set_operation ::= query_body { 'UNION' | 'EXCEPT' | 'INTERSECT' } [ 'ALL' ] query_body
    /// ```
    pub fn parse_query_body(&mut self, precedence: u8) -> Result<SetExpr, ParserError> {
        // We parse the expression using a Pratt parser, as in `parse_expr()`.
        // Start by parsing a restricted SELECT or a `(subquery)`:
        let mut expr = if self.parse_keyword(Keyword::SELECT) {
            SetExpr::Select(Box::new(self.parse_select()?))
        } else if self.consume_token(&Token::LParen) {
            // CTEs are not allowed here, but the parser currently accepts them
            let subquery = self.parse_query(None)?;
            self.expect_token(&Token::RParen)?;
            SetExpr::Query(Box::new(subquery))
        } else if self.parse_keyword(Keyword::VALUES) {
            SetExpr::Values(self.parse_values(false)?)
        } else {
            return self.expected(
                "SELECT, VALUES, or a subquery in the query body",
                self.peek_token(),
            );
        };

        loop {
            // The query can be optionally followed by a set operator:
            let op = self.parse_set_operator(&self.peek_token().token);
            let next_precedence = match op {
                // UNION and EXCEPT have the same binding power and evaluate left-to-right
                Some(SetOperator::Union) | Some(SetOperator::Except) => 10,
                // INTERSECT has higher precedence than UNION/EXCEPT
                Some(SetOperator::Intersect) => 20,
                // Unexpected token or EOF => stop parsing the query body
                None => break,
            };
            if precedence >= next_precedence {
                break;
            }
            self.next_token(); // skip past the set operator
            let set_quantifier = self.parse_set_quantifier(&op);
            expr = SetExpr::SetOperation {
                left: Box::new(expr),
                op: op.unwrap(),
                set_quantifier,
                right: Box::new(self.parse_query_body(next_precedence)?),
            };
        }

        Ok(expr)
    }

    pub fn parse_set_operator(&mut self, token: &Token) -> Option<SetOperator> {
        match token {
            Token::Word(w) if w.keyword == Keyword::UNION => Some(SetOperator::Union),
            Token::Word(w) if w.keyword == Keyword::EXCEPT => Some(SetOperator::Except),
            Token::Word(w) if w.keyword == Keyword::INTERSECT => Some(SetOperator::Intersect),
            _ => None,
        }
    }

    pub fn parse_set_quantifier(&mut self, op: &Option<SetOperator>) -> SetQuantifier {
        match op {
            Some(SetOperator::Union) => {
                if self.parse_keyword(Keyword::ALL) {
                    SetQuantifier::All
                } else if self.parse_keyword(Keyword::DISTINCT) {
                    SetQuantifier::Distinct
                } else {
                    SetQuantifier::None
                }
            }
            Some(SetOperator::Except) | Some(SetOperator::Intersect) => {
                if self.parse_keyword(Keyword::ALL) {
                    SetQuantifier::All
                } else if self.parse_keyword(Keyword::DISTINCT) {
                    SetQuantifier::Distinct
                } else {
                    SetQuantifier::None
                }
            }
            _ => SetQuantifier::None,
        }
    }

    /// Parse a restricted `SELECT` statement (no CTEs / `UNION` / `ORDER BY`),
    /// assuming the initial `SELECT` was already consumed
    pub fn parse_select(&mut self) -> Result<Select, ParserError> {
        let distinct = self.parse_all_or_distinct()?;

        let top = if self.parse_keyword(Keyword::TOP) {
            Some(self.parse_top()?)
        } else {
            None
        };

        let projection = self.parse_projection()?;

        let into = if self.parse_keyword(Keyword::INTO) {
            let temporary = self
                .parse_one_of_keywords(&[Keyword::TEMP, Keyword::TEMPORARY])
                .is_some();
            let unlogged = self.parse_keyword(Keyword::UNLOGGED);
            let table = self.parse_keyword(Keyword::TABLE);
            let name = self.parse_object_name()?;
            Some(SelectInto {
                temporary,
                unlogged,
                table,
                name,
            })
        } else {
            None
        };

        // Note that for keywords to be properly handled here, they need to be
        // added to `RESERVED_FOR_COLUMN_ALIAS` / `RESERVED_FOR_TABLE_ALIAS`,
        // otherwise they may be parsed as an alias as part of the `projection`
        // or `from`.

        let from = if self.parse_keyword(Keyword::FROM) {
            self.parse_comma_separated(Parser::parse_table_and_joins)?
        } else {
            vec![]
        };

        let mut lateral_views = vec![];
        loop {
            if self.parse_keywords(&[Keyword::LATERAL, Keyword::VIEW]) {
                let outer = self.parse_keyword(Keyword::OUTER);
                let lateral_view = self.parse_expr()?;
                let lateral_view_name = self.parse_object_name()?;
                let lateral_col_alias = self
                    .parse_comma_separated(|parser| {
                        parser.parse_optional_alias(&[
                            Keyword::WHERE,
                            Keyword::GROUP,
                            Keyword::CLUSTER,
                            Keyword::HAVING,
                            Keyword::LATERAL,
                        ]) // This couldn't possibly be a bad idea
                    })?
                    .into_iter()
                    .flatten()
                    .collect();

                lateral_views.push(LateralView {
                    lateral_view,
                    lateral_view_name,
                    lateral_col_alias,
                    outer,
                });
            } else {
                break;
            }
        }

        let selection = if self.parse_keyword(Keyword::WHERE) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let group_by = if self.parse_keywords(&[Keyword::GROUP, Keyword::BY]) {
            self.parse_comma_separated(Parser::parse_group_by_expr)?
        } else {
            vec![]
        };

        let cluster_by = if self.parse_keywords(&[Keyword::CLUSTER, Keyword::BY]) {
            self.parse_comma_separated(Parser::parse_expr)?
        } else {
            vec![]
        };

        let distribute_by = if self.parse_keywords(&[Keyword::DISTRIBUTE, Keyword::BY]) {
            self.parse_comma_separated(Parser::parse_expr)?
        } else {
            vec![]
        };

        let sort_by = if self.parse_keywords(&[Keyword::SORT, Keyword::BY]) {
            self.parse_comma_separated(Parser::parse_expr)?
        } else {
            vec![]
        };

        let having = if self.parse_keyword(Keyword::HAVING) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let qualify = if self.parse_keyword(Keyword::QUALIFY) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Select {
            distinct,
            top,
            projection,
            into,
            from,
            lateral_views,
            selection,
            group_by,
            cluster_by,
            distribute_by,
            sort_by,
            having,
            qualify,
        })
    }

    pub fn parse_table_and_joins(&mut self) -> Result<TableWithJoins, ParserError> {
        let relation = self.parse_table_factor()?;
        // Note that for keywords to be properly handled here, they need to be
        // added to `RESERVED_FOR_TABLE_ALIAS`, otherwise they may be parsed as
        // a table alias.
        let mut joins = vec![];
        loop {
            let join = if self.parse_keyword(Keyword::CROSS) {
                let join_operator = if self.parse_keyword(Keyword::JOIN) {
                    JoinOperator::CrossJoin
                } else if self.parse_keyword(Keyword::APPLY) {
                    // MSSQL extension, similar to CROSS JOIN LATERAL
                    JoinOperator::CrossApply
                } else {
                    return self.expected("JOIN or APPLY after CROSS", self.peek_token());
                };
                Join {
                    relation: self.parse_table_factor()?,
                    join_operator,
                }
            } else if self.parse_keyword(Keyword::OUTER) {
                // MSSQL extension, similar to LEFT JOIN LATERAL .. ON 1=1
                self.expect_keyword(Keyword::APPLY)?;
                Join {
                    relation: self.parse_table_factor()?,
                    join_operator: JoinOperator::OuterApply,
                }
            } else {
                let natural = self.parse_keyword(Keyword::NATURAL);
                let peek_keyword = if let Token::Word(w) = self.peek_token().token {
                    w.keyword
                } else {
                    Keyword::NoKeyword
                };

                let join_operator_type = match peek_keyword {
                    Keyword::INNER | Keyword::JOIN => {
                        let _ = self.parse_keyword(Keyword::INNER); // [ INNER ]
                        self.expect_keyword(Keyword::JOIN)?;
                        JoinOperator::Inner
                    }
                    kw @ Keyword::LEFT | kw @ Keyword::RIGHT => {
                        let _ = self.next_token(); // consume LEFT/RIGHT
                        let is_left = kw == Keyword::LEFT;
                        let join_type = self.parse_one_of_keywords(&[
                            Keyword::OUTER,
                            Keyword::SEMI,
                            Keyword::ANTI,
                            Keyword::JOIN,
                        ]);
                        match join_type {
                            Some(Keyword::OUTER) => {
                                self.expect_keyword(Keyword::JOIN)?;
                                if is_left {
                                    JoinOperator::LeftOuter
                                } else {
                                    JoinOperator::RightOuter
                                }
                            }
                            Some(Keyword::SEMI) => {
                                self.expect_keyword(Keyword::JOIN)?;
                                if is_left {
                                    JoinOperator::LeftSemi
                                } else {
                                    JoinOperator::RightSemi
                                }
                            }
                            Some(Keyword::ANTI) => {
                                self.expect_keyword(Keyword::JOIN)?;
                                if is_left {
                                    JoinOperator::LeftAnti
                                } else {
                                    JoinOperator::RightAnti
                                }
                            }
                            Some(Keyword::JOIN) => {
                                if is_left {
                                    JoinOperator::LeftOuter
                                } else {
                                    JoinOperator::RightOuter
                                }
                            }
                            _ => {
                                return Err(ParserError::ParserError(format!(
                                    "expected OUTER, SEMI, ANTI or JOIN after {kw:?}"
                                )))
                            }
                        }
                    }
                    Keyword::FULL => {
                        let _ = self.next_token(); // consume FULL
                        let _ = self.parse_keyword(Keyword::OUTER); // [ OUTER ]
                        self.expect_keyword(Keyword::JOIN)?;
                        JoinOperator::FullOuter
                    }
                    Keyword::OUTER => {
                        return self.expected("LEFT, RIGHT, or FULL", self.peek_token());
                    }
                    _ if natural => {
                        return self.expected("a join type after NATURAL", self.peek_token());
                    }
                    _ => break,
                };
                let relation = self.parse_table_factor()?;
                let join_constraint = self.parse_join_constraint(natural)?;
                Join {
                    relation,
                    join_operator: join_operator_type(join_constraint),
                }
            };
            joins.push(join);
        }
        Ok(TableWithJoins { relation, joins })
    }

    /// A table name or a parenthesized subquery, followed by optional `[AS] alias`
    pub fn parse_table_factor(&mut self) -> Result<TableFactor, ParserError> {
        if self.parse_keyword(Keyword::LATERAL) {
            // LATERAL must always be followed by a subquery.
            if !self.consume_token(&Token::LParen) {
                self.expected("subquery after LATERAL", self.peek_token())?;
            }
            self.parse_derived_table_factor(Lateral)
        } else if self.parse_keyword(Keyword::TABLE) {
            // parse table function (SELECT * FROM TABLE (<expr>) [ AS <alias> ])
            self.expect_token(&Token::LParen)?;
            let expr = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
            Ok(TableFactor::TableFunction { expr, alias })
        } else if self.consume_token(&Token::DoubleLBrace) {
            // parse dbt functions like (SELECT * FROM {{ ref('model') }} [ AS <alias> ])
            // I think I need to add some parse_ref function?
            let next_token = self.peek_token();
            match &next_token.token {
                Token::Word(w) if w.value.to_lowercase() == "ref" => {
                    self.next_token(); // Consume the "ref" keyword
                    self.expect_token(&Token::LParen)?;
                    let model_name = self.parse_ref()?;
                    self.expect_token(&Token::DoubleRBrace)?;
                    let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
                    return Ok(TableFactor::DbtRef { model_name, alias });
                }
                Token::Word(w) if w.value.to_lowercase() == "source" => {
                    self.next_token(); // Consume the "source" keyword
                    self.expect_token(&Token::LParen)?;
                    let (source_name, table_name) = self.parse_source()?;
                    self.expect_token(&Token::DoubleRBrace)?;
                    let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;

                    return Ok(TableFactor::DbtSource {
                        source_name,
                        table_name,
                        alias,
                    });
                }
                _ => return Err(ParserError::ParserError(format!(
                    "Expected `ref` or `source` keyword after '{{', found: {}",
                    next_token.token
                ))),
            }
            // let model_name = self.parse_ref()?;
            // let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
            // return Ok(TableFactor::DbtRef { model_name, alias });
        } else if self.consume_token(&Token::LParen) {
            // A left paren introduces either a derived table (i.e., a subquery)
            // or a nested join. It's nearly impossible to determine ahead of
            // time which it is... so we just try to parse both.
            //
            // Here's an example that demonstrates the complexity:
            //                     /-------------------------------------------------------\
            //                     | /-----------------------------------\                 |
            //     SELECT * FROM ( ( ( (SELECT 1) UNION (SELECT 2) ) AS t1 NATURAL JOIN t2 ) )
            //                   ^ ^ ^ ^
            //                   | | | |
            //                   | | | |
            //                   | | | (4) belongs to a SetExpr::Query inside the subquery
            //                   | | (3) starts a derived table (subquery)
            //                   | (2) starts a nested join
            //                   (1) an additional set of parens around a nested join
            //

            // If the recently consumed '(' starts a derived table, the call to
            // `parse_derived_table_factor` below will return success after parsing the
            // subquery, followed by the closing ')', and the alias of the derived table.
            // In the example above this is case (3).
            return_ok_if_some!(
                self.maybe_parse(|parser| parser.parse_derived_table_factor(NotLateral))
            );
            // A parsing error from `parse_derived_table_factor` indicates that the '(' we've
            // recently consumed does not start a derived table (cases 1, 2, or 4).
            // `maybe_parse` will ignore such an error and rewind to be after the opening '('.

            // Inside the parentheses we expect to find an (A) table factor
            // followed by some joins or (B) another level of nesting.
            let mut table_and_joins = self.parse_table_and_joins()?;

            #[allow(clippy::if_same_then_else)]
            if !table_and_joins.joins.is_empty() {
                self.expect_token(&Token::RParen)?;
                let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
                Ok(TableFactor::NestedJoin {
                    table_with_joins: Box::new(table_and_joins),
                    alias,
                }) // (A)
            } else if let TableFactor::NestedJoin {
                table_with_joins: _,
                alias: _,
            } = &table_and_joins.relation
            {
                // (B): `table_and_joins` (what we found inside the parentheses)
                // is a nested join `(foo JOIN bar)`, not followed by other joins.
                self.expect_token(&Token::RParen)?;
                let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
                Ok(TableFactor::NestedJoin {
                    table_with_joins: Box::new(table_and_joins),
                    alias,
                })
            } else if dialect_of!(self is SnowflakeDialect | GenericDialect) {
                // Dialect-specific behavior: Snowflake diverges from the
                // standard and from most of the other implementations by
                // allowing extra parentheses not only around a join (B), but
                // around lone table names (e.g. `FROM (mytable [AS alias])`)
                // and around derived tables (e.g. `FROM ((SELECT ...)
                // [AS alias])`) as well.
                self.expect_token(&Token::RParen)?;

                if let Some(outer_alias) =
                    self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?
                {
                    // Snowflake also allows specifying an alias *after* parens
                    // e.g. `FROM (mytable) AS alias`
                    match &mut table_and_joins.relation {
                        TableFactor::Derived { alias, .. }
                        | TableFactor::Table { alias, .. }
                        | TableFactor::DbtRef { alias, .. }
                        | TableFactor::DbtSource { alias, .. }
                        | TableFactor::UNNEST { alias, .. }
                        | TableFactor::TableFunction { alias, .. }
                        | TableFactor::Pivot {
                            pivot_alias: alias, ..
                        }
                        | TableFactor::NestedJoin { alias, .. } => {
                            // but not `FROM (mytable AS alias1) AS alias2`.
                            if let Some(inner_alias) = alias {
                                return Err(ParserError::ParserError(format!(
                                    "duplicate alias {inner_alias}"
                                )));
                            }
                            // Act as if the alias was specified normally next
                            // to the table name: `(mytable) AS alias` ->
                            // `(mytable AS alias)`
                            alias.replace(outer_alias);
                        }
                    };
                }
                // Do not store the extra set of parens in the AST
                Ok(table_and_joins.relation)
            } else {
                // The SQL spec prohibits derived tables and bare tables from
                // appearing alone in parentheses (e.g. `FROM (mytable)`)
                self.expected("joined table", self.peek_token())
            }
        } else if dialect_of!(self is BigQueryDialect | GenericDialect)
            && self.parse_keyword(Keyword::UNNEST)
        {
            self.expect_token(&Token::LParen)?;
            let expr = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;

            let alias = match self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS) {
                Ok(Some(alias)) => Some(alias),
                Ok(None) => None,
                Err(e) => return Err(e),
            };

            let with_offset = match self.expect_keywords(&[Keyword::WITH, Keyword::OFFSET]) {
                Ok(()) => true,
                Err(_) => false,
            };

            let with_offset_alias = if with_offset {
                match self.parse_optional_alias(keywords::RESERVED_FOR_COLUMN_ALIAS) {
                    Ok(Some(alias)) => Some(alias),
                    Ok(None) => None,
                    Err(e) => return Err(e),
                }
            } else {
                None
            };

            Ok(TableFactor::UNNEST {
                alias,
                array_expr: Box::new(expr),
                with_offset,
                with_offset_alias,
            })
        } else {
            let name = self.parse_object_name()?;

            // Postgres, MSSQL: table-valued functions:
            let args = if self.consume_token(&Token::LParen) {
                Some(self.parse_optional_args()?)
            } else {
                None
            };

            let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;

            // Pivot
            if self.parse_keyword(Keyword::PIVOT) {
                return self.parse_pivot_table_factor(name, alias);
            }

            // MSSQL-specific table hints:
            let mut with_hints = vec![];
            if self.parse_keyword(Keyword::WITH) {
                if self.consume_token(&Token::LParen) {
                    with_hints = self.parse_comma_separated(Parser::parse_expr)?;
                    self.expect_token(&Token::RParen)?;
                } else {
                    // rewind, as WITH may belong to the next statement's CTE
                    self.prev_token();
                }
            };
            Ok(TableFactor::Table {
                name,
                alias,
                args,
                with_hints,
            })
        }
    }

    pub fn parse_derived_table_factor(
        &mut self,
        lateral: IsLateral,
    ) -> Result<TableFactor, ParserError> {
        let subquery = Box::new(self.parse_query(None)?);
        self.expect_token(&Token::RParen)?;
        let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
        Ok(TableFactor::Derived {
            lateral: match lateral {
                Lateral => true,
                NotLateral => false,
            },
            subquery,
            alias,
        })
    }

    pub fn parse_pivot_table_factor(
        &mut self,
        name: ObjectName,
        table_alias: Option<TableAlias>,
    ) -> Result<TableFactor, ParserError> {
        self.expect_token(&Token::LParen)?;
        let function_name = match self.next_token().token {
            Token::Word(w) => Ok(w.value),
            _ => self.expected("an aggregate function name", self.peek_token()),
        }?;
        let function = self.parse_function(ObjectName(vec![Ident::new(function_name)]))?;
        self.expect_keyword(Keyword::FOR)?;
        let value_column = self.parse_object_name()?.0;
        self.expect_keyword(Keyword::IN)?;
        self.expect_token(&Token::LParen)?;
        let pivot_values = self.parse_comma_separated(Parser::parse_value)?;
        self.expect_token(&Token::RParen)?;
        self.expect_token(&Token::RParen)?;
        let alias = self.parse_optional_table_alias(keywords::RESERVED_FOR_TABLE_ALIAS)?;
        Ok(TableFactor::Pivot {
            name,
            table_alias,
            aggregate_function: function,
            value_column,
            pivot_values,
            pivot_alias: alias,
        })
    }

    pub fn parse_join_constraint(&mut self, natural: bool) -> Result<JoinConstraint, ParserError> {
        if natural {
            Ok(JoinConstraint::Natural)
        } else if self.parse_keyword(Keyword::ON) {
            let constraint = self.parse_expr()?;
            Ok(JoinConstraint::On(constraint))
        } else if self.parse_keyword(Keyword::USING) {
            let columns = self.parse_parenthesized_column_list(Mandatory, false)?;
            Ok(JoinConstraint::Using(columns))
        } else {
            Ok(JoinConstraint::None)
            //self.expected("ON, or USING after JOIN", self.peek_token())
        }
    }

    pub fn parse_function_args(&mut self) -> Result<FunctionArg, ParserError> {
        if self.peek_nth_token(1) == Token::RArrow {
            let name = self.parse_identifier()?;

            self.expect_token(&Token::RArrow)?;
            let arg = self.parse_wildcard_expr()?.into();

            Ok(FunctionArg::Named { name, arg })
        } else {
            Ok(FunctionArg::Unnamed(self.parse_wildcard_expr()?.into()))
        }
    }

    pub fn parse_optional_args(&mut self) -> Result<Vec<FunctionArg>, ParserError> {
        if self.consume_token(&Token::RParen) {
            Ok(vec![])
        } else {
            let args = self.parse_comma_separated(Parser::parse_function_args)?;
            self.expect_token(&Token::RParen)?;
            Ok(args)
        }
    }

    /// Parse a comma-delimited list of projections after SELECT
    pub fn parse_select_item(&mut self) -> Result<SelectItem, ParserError> {
        match self.parse_wildcard_expr()? {
            WildcardExpr::Expr(expr) => {
                let expr: Expr = if self.dialect.supports_filter_during_aggregation()
                    && self.parse_keyword(Keyword::FILTER)
                {
                    let i = self.index - 1;
                    if self.consume_token(&Token::LParen) && self.parse_keyword(Keyword::WHERE) {
                        let filter = self.parse_expr()?;
                        self.expect_token(&Token::RParen)?;
                        Expr::AggregateExpressionWithFilter {
                            expr: Box::new(expr),
                            filter: Box::new(filter),
                        }
                    } else {
                        self.index = i;
                        expr
                    }
                } else {
                    expr
                };
                self.parse_optional_alias(keywords::RESERVED_FOR_COLUMN_ALIAS)
                    .map(|alias| match alias {
                        Some(alias) => SelectItem::ExprWithAlias { expr, alias },
                        None => SelectItem::UnnamedExpr(expr),
                    })
            }
            WildcardExpr::QualifiedWildcard(prefix) => Ok(SelectItem::QualifiedWildcard(
                prefix,
                self.parse_wildcard_additional_options()?,
            )),
            WildcardExpr::Wildcard => Ok(SelectItem::Wildcard(
                self.parse_wildcard_additional_options()?,
            )),
        }
    }

    /// Parse an [`WildcardAdditionalOptions`](WildcardAdditionalOptions) information for wildcard select items.
    ///
    /// If it is not possible to parse it, will return an option.
    pub fn parse_wildcard_additional_options(
        &mut self,
    ) -> Result<WildcardAdditionalOptions, ParserError> {
        let opt_exclude = if dialect_of!(self is GenericDialect | SnowflakeDialect) {
            self.parse_optional_select_item_exclude()?
        } else {
            None
        };
        let opt_except = if dialect_of!(self is GenericDialect | BigQueryDialect) {
            self.parse_optional_select_item_except()?
        } else {
            None
        };
        let opt_rename = if dialect_of!(self is GenericDialect | SnowflakeDialect) {
            self.parse_optional_select_item_rename()?
        } else {
            None
        };

        let opt_replace = if dialect_of!(self is GenericDialect | BigQueryDialect) {
            self.parse_optional_select_item_replace()?
        } else {
            None
        };

        Ok(WildcardAdditionalOptions {
            opt_exclude,
            opt_except,
            opt_rename,
            opt_replace,
        })
    }

    /// Parse an [`Exclude`](ExcludeSelectItem) information for wildcard select items.
    ///
    /// If it is not possible to parse it, will return an option.
    pub fn parse_optional_select_item_exclude(
        &mut self,
    ) -> Result<Option<ExcludeSelectItem>, ParserError> {
        let opt_exclude = if self.parse_keyword(Keyword::EXCLUDE) {
            if self.consume_token(&Token::LParen) {
                let columns = self.parse_comma_separated(|parser| parser.parse_identifier())?;
                self.expect_token(&Token::RParen)?;
                Some(ExcludeSelectItem::Multiple(columns))
            } else {
                let column = self.parse_identifier()?;
                Some(ExcludeSelectItem::Single(column))
            }
        } else {
            None
        };

        Ok(opt_exclude)
    }

    /// Parse an [`Except`](ExceptSelectItem) information for wildcard select items.
    ///
    /// If it is not possible to parse it, will return an option.
    pub fn parse_optional_select_item_except(
        &mut self,
    ) -> Result<Option<ExceptSelectItem>, ParserError> {
        let opt_except = if self.parse_keyword(Keyword::EXCEPT) {
            let idents = self.parse_parenthesized_column_list(Mandatory, false)?;
            match &idents[..] {
                [] => {
                    return self.expected(
                        "at least one column should be parsed by the expect clause",
                        self.peek_token(),
                    )?;
                }
                [first, idents @ ..] => Some(ExceptSelectItem {
                    first_element: first.clone(),
                    additional_elements: idents.to_vec(),
                }),
            }
        } else {
            None
        };

        Ok(opt_except)
    }

    /// Parse a [`Rename`](RenameSelectItem) information for wildcard select items.
    pub fn parse_optional_select_item_rename(
        &mut self,
    ) -> Result<Option<RenameSelectItem>, ParserError> {
        let opt_rename = if self.parse_keyword(Keyword::RENAME) {
            if self.consume_token(&Token::LParen) {
                let idents =
                    self.parse_comma_separated(|parser| parser.parse_identifier_with_alias())?;
                self.expect_token(&Token::RParen)?;
                Some(RenameSelectItem::Multiple(idents))
            } else {
                let ident = self.parse_identifier_with_alias()?;
                Some(RenameSelectItem::Single(ident))
            }
        } else {
            None
        };

        Ok(opt_rename)
    }

    /// Parse a [`Replace`](ReplaceSelectItem) information for wildcard select items.
    pub fn parse_optional_select_item_replace(
        &mut self,
    ) -> Result<Option<ReplaceSelectItem>, ParserError> {
        let opt_replace = if self.parse_keyword(Keyword::REPLACE) {
            if self.consume_token(&Token::LParen) {
                let items = self.parse_comma_separated(|parser| {
                    Ok(Box::new(parser.parse_replace_elements()?))
                })?;
                self.expect_token(&Token::RParen)?;
                Some(ReplaceSelectItem { items })
            } else {
                let tok = self.next_token();
                return self.expected("( after REPLACE but", tok);
            }
        } else {
            None
        };

        Ok(opt_replace)
    }
    pub fn parse_replace_elements(&mut self) -> Result<ReplaceSelectElement, ParserError> {
        let expr = self.parse_expr()?;
        let as_keyword = self.parse_keyword(Keyword::AS);
        let ident = self.parse_identifier()?;
        Ok(ReplaceSelectElement {
            expr,
            column_name: ident,
            as_keyword,
        })
    }

    /// Parse an expression, optionally followed by ASC or DESC (used in ORDER BY)
    pub fn parse_order_by_expr(&mut self) -> Result<OrderByExpr, ParserError> {
        let expr = self.parse_expr()?;

        let asc = if self.parse_keyword(Keyword::ASC) {
            Some(true)
        } else if self.parse_keyword(Keyword::DESC) {
            Some(false)
        } else {
            None
        };

        let nulls_first = if self.parse_keywords(&[Keyword::NULLS, Keyword::FIRST]) {
            Some(true)
        } else if self.parse_keywords(&[Keyword::NULLS, Keyword::LAST]) {
            Some(false)
        } else {
            None
        };

        Ok(OrderByExpr {
            expr,
            asc,
            nulls_first,
        })
    }

    /// Parse a TOP clause, MSSQL equivalent of LIMIT,
    /// that follows after `SELECT [DISTINCT]`.
    pub fn parse_top(&mut self) -> Result<Top, ParserError> {
        let quantity = if self.consume_token(&Token::LParen) {
            let quantity = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            Some(quantity)
        } else {
            Some(Expr::Value(self.parse_number_value()?))
        };

        let percent = self.parse_keyword(Keyword::PERCENT);

        let with_ties = self.parse_keywords(&[Keyword::WITH, Keyword::TIES]);

        Ok(Top {
            with_ties,
            percent,
            quantity,
        })
    }

    /// Parse a LIMIT clause
    pub fn parse_limit(&mut self) -> Result<Option<Expr>, ParserError> {
        if self.parse_keyword(Keyword::ALL) {
            Ok(None)
        } else {
            Ok(Some(self.parse_expr()?))
        }
    }

    /// Parse an OFFSET clause
    pub fn parse_offset(&mut self) -> Result<Offset, ParserError> {
        let value = self.parse_expr()?;
        let rows = if self.parse_keyword(Keyword::ROW) {
            OffsetRows::Row
        } else if self.parse_keyword(Keyword::ROWS) {
            OffsetRows::Rows
        } else {
            OffsetRows::None
        };
        Ok(Offset { value, rows })
    }

    pub fn parse_values(&mut self, allow_empty: bool) -> Result<Values, ParserError> {
        let mut explicit_row = false;

        let rows = self.parse_comma_separated(|parser| {
            if parser.parse_keyword(Keyword::ROW) {
                explicit_row = true;
            }

            parser.expect_token(&Token::LParen)?;
            if allow_empty && parser.peek_token().token == Token::RParen {
                parser.next_token();
                Ok(vec![])
            } else {
                let exprs = parser.parse_comma_separated(Parser::parse_expr)?;
                parser.expect_token(&Token::RParen)?;
                Ok(exprs)
            }
        })?;
        Ok(Values { explicit_row, rows })
    }

    /// The index of the first unprocessed token.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Word {
    pub fn to_ident(&self) -> Ident {
        Ident {
            value: self.value.clone(),
            quote_style: self.quote_style,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{all_dialects, TestedDialects};

    use super::*;

    #[test]
    fn test_prev_index() {
        let sql = "SELECT version";
        all_dialects().run_parser_method(sql, |parser| {
            assert_eq!(parser.peek_token(), Token::make_keyword("SELECT"));
            assert_eq!(parser.next_token(), Token::make_keyword("SELECT"));
            parser.prev_token();
            assert_eq!(parser.next_token(), Token::make_keyword("SELECT"));
            assert_eq!(parser.next_token(), Token::make_word("version", None));
            parser.prev_token();
            assert_eq!(parser.peek_token(), Token::make_word("version", None));
            assert_eq!(parser.next_token(), Token::make_word("version", None));
            assert_eq!(parser.peek_token(), Token::EOF);
            parser.prev_token();
            assert_eq!(parser.next_token(), Token::make_word("version", None));
            assert_eq!(parser.next_token(), Token::EOF);
            assert_eq!(parser.next_token(), Token::EOF);
            parser.prev_token();
        });
    }

    #[test]
    fn test_parse_limit() {
        let sql = "SELECT * FROM user LIMIT 1";
        all_dialects().run_parser_method(sql, |parser| {
            let ast = parser.parse_query(None).unwrap();
            assert_eq!(ast.to_string(), sql.to_string());
        });

        let sql = "SELECT * FROM user LIMIT $1 OFFSET $2";
        let dialects = TestedDialects {
            dialects: vec![
                Box::new(PostgreSqlDialect {}),
                Box::new(GenericDialect {}),
                Box::new(SnowflakeDialect {}),
            ],
        };

        dialects.run_parser_method(sql, |parser| {
            let ast = parser.parse_query(None).unwrap();
            assert_eq!(ast.to_string(), sql.to_string());
        });

    }

    #[cfg(test)]
    mod test_parse_data_type {
        use crate::ast::{
            CharLengthUnits, CharacterLength, DataType, ExactNumberInfo, ObjectName, TimezoneInfo,
        };
        use crate::dialect::{AnsiDialect, GenericDialect};
        use crate::test_utils::TestedDialects;

        macro_rules! test_parse_data_type {
            ($dialect:expr, $input:expr, $expected_type:expr $(,)?) => {{
                $dialect.run_parser_method(&*$input, |parser| {
                    let data_type = parser.parse_data_type().unwrap();
                    assert_eq!($expected_type, data_type);
                    assert_eq!($input.to_string(), data_type.to_string());
                });
            }};
        }

        #[test]
        fn test_ansii_character_string_types() {
            // Character string types: <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#character-string-type>
            let dialect = TestedDialects {
                dialects: vec![Box::new(GenericDialect {}), Box::new(AnsiDialect {})],
            };

            test_parse_data_type!(dialect, "CHARACTER", DataType::Character(None));

            test_parse_data_type!(
                dialect,
                "CHARACTER(20)",
                DataType::Character(Some(CharacterLength {
                    length: 20,
                    unit: None
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHARACTER(20 CHARACTERS)",
                DataType::Character(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Characters)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHARACTER(20 OCTETS)",
                DataType::Character(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Octets)
                }))
            );

            test_parse_data_type!(dialect, "CHAR", DataType::Char(None));

            test_parse_data_type!(
                dialect,
                "CHAR(20)",
                DataType::Char(Some(CharacterLength {
                    length: 20,
                    unit: None
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHAR(20 CHARACTERS)",
                DataType::Char(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Characters)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHAR(20 OCTETS)",
                DataType::Char(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Octets)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHARACTER VARYING(20)",
                DataType::CharacterVarying(Some(CharacterLength {
                    length: 20,
                    unit: None
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHARACTER VARYING(20 CHARACTERS)",
                DataType::CharacterVarying(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Characters)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHARACTER VARYING(20 OCTETS)",
                DataType::CharacterVarying(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Octets)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHAR VARYING(20)",
                DataType::CharVarying(Some(CharacterLength {
                    length: 20,
                    unit: None
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHAR VARYING(20 CHARACTERS)",
                DataType::CharVarying(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Characters)
                }))
            );

            test_parse_data_type!(
                dialect,
                "CHAR VARYING(20 OCTETS)",
                DataType::CharVarying(Some(CharacterLength {
                    length: 20,
                    unit: Some(CharLengthUnits::Octets)
                }))
            );

            test_parse_data_type!(
                dialect,
                "VARCHAR(20)",
                DataType::Varchar(Some(CharacterLength {
                    length: 20,
                    unit: None
                }))
            );
        }

        #[test]
        fn test_ansii_character_large_object_types() {
            // Character large object types: <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#character-large-object-length>
            let dialect = TestedDialects {
                dialects: vec![Box::new(GenericDialect {}), Box::new(AnsiDialect {})],
            };

            test_parse_data_type!(
                dialect,
                "CHARACTER LARGE OBJECT",
                DataType::CharacterLargeObject(None)
            );
            test_parse_data_type!(
                dialect,
                "CHARACTER LARGE OBJECT(20)",
                DataType::CharacterLargeObject(Some(20))
            );

            test_parse_data_type!(
                dialect,
                "CHAR LARGE OBJECT",
                DataType::CharLargeObject(None)
            );
            test_parse_data_type!(
                dialect,
                "CHAR LARGE OBJECT(20)",
                DataType::CharLargeObject(Some(20))
            );

            test_parse_data_type!(dialect, "CLOB", DataType::Clob(None));
            test_parse_data_type!(dialect, "CLOB(20)", DataType::Clob(Some(20)));
        }

        #[test]
        fn test_parse_custom_types() {
            let dialect = TestedDialects {
                dialects: vec![Box::new(GenericDialect {}), Box::new(AnsiDialect {})],
            };
            test_parse_data_type!(
                dialect,
                "GEOMETRY",
                DataType::Custom(ObjectName(vec!["GEOMETRY".into()]), vec![])
            );

            test_parse_data_type!(
                dialect,
                "GEOMETRY(POINT)",
                DataType::Custom(
                    ObjectName(vec!["GEOMETRY".into()]),
                    vec!["POINT".to_string()]
                )
            );

            test_parse_data_type!(
                dialect,
                "GEOMETRY(POINT, 4326)",
                DataType::Custom(
                    ObjectName(vec!["GEOMETRY".into()]),
                    vec!["POINT".to_string(), "4326".to_string()]
                )
            );
        }

        #[test]
        fn test_ansii_exact_numeric_types() {
            // Exact numeric types: <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#exact-numeric-type>
            let dialect = TestedDialects {
                dialects: vec![Box::new(GenericDialect {}), Box::new(AnsiDialect {})],
            };

            test_parse_data_type!(dialect, "NUMERIC", DataType::Numeric(ExactNumberInfo::None));

            test_parse_data_type!(
                dialect,
                "NUMERIC(2)",
                DataType::Numeric(ExactNumberInfo::Precision(2))
            );

            test_parse_data_type!(
                dialect,
                "NUMERIC(2,10)",
                DataType::Numeric(ExactNumberInfo::PrecisionAndScale(2, 10))
            );

            test_parse_data_type!(dialect, "DECIMAL", DataType::Decimal(ExactNumberInfo::None));

            test_parse_data_type!(
                dialect,
                "DECIMAL(2)",
                DataType::Decimal(ExactNumberInfo::Precision(2))
            );

            test_parse_data_type!(
                dialect,
                "DECIMAL(2,10)",
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(2, 10))
            );

            test_parse_data_type!(dialect, "DEC", DataType::Dec(ExactNumberInfo::None));

            test_parse_data_type!(
                dialect,
                "DEC(2)",
                DataType::Dec(ExactNumberInfo::Precision(2))
            );

            test_parse_data_type!(
                dialect,
                "DEC(2,10)",
                DataType::Dec(ExactNumberInfo::PrecisionAndScale(2, 10))
            );
        }

        #[test]
        fn test_ansii_date_type() {
            // Datetime types: <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#datetime-type>
            let dialect = TestedDialects {
                dialects: vec![Box::new(GenericDialect {}), Box::new(AnsiDialect {})],
            };

            test_parse_data_type!(dialect, "DATE", DataType::Date);

            test_parse_data_type!(dialect, "TIME", DataType::Time(None, TimezoneInfo::None));

            test_parse_data_type!(
                dialect,
                "TIME(6)",
                DataType::Time(Some(6), TimezoneInfo::None)
            );

            test_parse_data_type!(
                dialect,
                "TIME WITH TIME ZONE",
                DataType::Time(None, TimezoneInfo::WithTimeZone)
            );

            test_parse_data_type!(
                dialect,
                "TIME(6) WITH TIME ZONE",
                DataType::Time(Some(6), TimezoneInfo::WithTimeZone)
            );

            test_parse_data_type!(
                dialect,
                "TIME WITHOUT TIME ZONE",
                DataType::Time(None, TimezoneInfo::WithoutTimeZone)
            );

            test_parse_data_type!(
                dialect,
                "TIME(6) WITHOUT TIME ZONE",
                DataType::Time(Some(6), TimezoneInfo::WithoutTimeZone)
            );

            test_parse_data_type!(
                dialect,
                "TIMESTAMP",
                DataType::Timestamp(None, TimezoneInfo::None)
            );

            test_parse_data_type!(
                dialect,
                "TIMESTAMP(22)",
                DataType::Timestamp(Some(22), TimezoneInfo::None)
            );

            test_parse_data_type!(
                dialect,
                "TIMESTAMP(22) WITH TIME ZONE",
                DataType::Timestamp(Some(22), TimezoneInfo::WithTimeZone)
            );

            test_parse_data_type!(
                dialect,
                "TIMESTAMP(33) WITHOUT TIME ZONE",
                DataType::Timestamp(Some(33), TimezoneInfo::WithoutTimeZone)
            );
        }
    }


    #[test]
    fn test_tokenizer_error_loc() {
        let sql = "foo '";
        let ast = Parser::parse_sql(&GenericDialect, sql);
        assert_eq!(
            ast,
            Err(ParserError::TokenizerError(
                "Unterminated string literal at Line: 1, Column 5".to_string()
            ))
        );
    }

    #[test]
    fn test_parser_error_loc() {
        // TODO: Once we thread token locations through the parser, we should update this
        // test to assert the locations of the referenced token
        let sql = "SELECT this is a syntax error";
        let ast = Parser::parse_sql(&GenericDialect, sql);
        assert_eq!(
            ast,
            Err(ParserError::ParserError(
                "Expected [NOT] NULL or TRUE|FALSE or [NOT] DISTINCT FROM after IS, found: a"
                    .to_string()
            ))
        );
    }

    #[test]
    fn parse_simple_jinja_ref() {
        let sql = "SELECT 1 FROM {{ ref('model') }}";
    
        let statements = Parser::parse_sql(&GenericDialect, sql).unwrap();
        assert_eq!(1, statements.len());

        let Statement::Query(query) = &statements[0];
        
        let is_dbt_ref_present = match &*query.body {
            SetExpr::Select(select) => select.from.iter().any(|table_with_joins| {
                matches!(
                    &table_with_joins.relation,
                    TableFactor::DbtRef {
                        model_name,
                        alias: None
                    } if model_name.value == "model" && model_name.quote_style == Some('\'')
                )
            }),
            _ => false,
        };
    
        assert!(is_dbt_ref_present, "DbtRef with model_name 'model' not found");
    }

    #[test]
    fn test_dbt_config_parsing() {
        let sql = r#"{{
  config(
    materialized = "table",
    sort = 'event_time',
    dist = 'event_id'
  )
}}
SELECT * FROM some_table;"#;


        let statements = Parser::parse_sql(&GenericDialect, sql).unwrap();
        assert_eq!(1, statements.len());

        let Statement::Query(query) = &statements[0];

        let config = query.config.as_ref().unwrap();
        assert_eq!(
            &DbtConfigValue::String("table".to_string()),
            config.values.get("materialized").unwrap()
        );
        assert_eq!(
            &DbtConfigValue::String("event_time".to_string()),
            config.values.get("sort").unwrap()
        );
        assert_eq!(
            &DbtConfigValue::String("event_id".to_string()),
            config.values.get("dist").unwrap()
        );

    }
}