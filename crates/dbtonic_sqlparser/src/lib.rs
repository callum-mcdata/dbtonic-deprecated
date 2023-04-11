//! SQL Parser for Rust
//!
//! This crate provides an ANSI:SQL 2011 lexer and parser that can parse SQL
//! into an Abstract Syntax Tree (AST). See the [sqlparser crates.io page]
//! for more information.
//!
//! See [`Parser::parse_sql`](crate::parser::Parser::parse_sql) and
//! [`Parser::new`](crate::parser::Parser::new) for the Parsing API
//! and the [`ast`](crate::ast) crate for the AST structure.
//!
//! Example:
//!
//! ```
//! use sqlparser::dialect::GenericDialect;
//! use sqlparser::parser::Parser;
//!
//! let dialect = GenericDialect {}; // or AnsiDialect
//!
//! let sql = "SELECT a, b, 123, myfunc(b) \
//!            FROM table_1 \
//!            WHERE a > b AND b < 100 \
//!            ORDER BY a DESC, b";
//!
//! let ast = Parser::parse_sql(&dialect, sql).unwrap();
//!
//! println!("AST: {:?}", ast);
//! ```
//! [sqlparser crates.io page]: https://crates.io/crates/sqlparser

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::upper_case_acronyms)]

// Allow proc-macros to find this crate
extern crate self as sqlparser;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;

pub mod ast;
#[macro_use]
pub mod dialect;
pub mod keywords;
pub mod parser;
pub mod tokenizer;

#[doc(hidden)]
// This is required to make utilities accessible by both the crate-internal
// unit-tests and by the integration tests <https://stackoverflow.com/a/44541071/1026>
// External users are not supposed to rely on this module.
pub mod test_utils;
