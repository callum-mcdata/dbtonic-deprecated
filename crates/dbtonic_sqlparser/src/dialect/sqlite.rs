

use crate::ast::Statement;
use crate::dialect::Dialect;
use crate::keywords::Keyword;
use crate::parser::{Parser, ParserError};

#[derive(Debug)]
pub struct SQLiteDialect {}

impl Dialect for SQLiteDialect {
    // see https://www.sqlite.org/lang_keywords.html
    // parse `...`, [...] and "..." as identifier
    // TODO: support depending on the context tread '...' as identifier too.
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '`' || ch == '"' || ch == '['
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        // See https://www.sqlite.org/draft/tokenreq.html
        ch.is_ascii_lowercase()
            || ch.is_ascii_uppercase()
            || ch == '_'
            || ch == '$'
            || ('\u{007f}'..='\u{ffff}').contains(&ch)
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ch.is_ascii_digit()
    }

    fn parse_statement(&self, parser: &mut Parser) -> Option<Result<Statement, ParserError>> {
        if parser.parse_keyword(Keyword::REPLACE) {
            parser.prev_token();
            Some(parser.parse_insert())
        } else {
            None
        }
    }
}
