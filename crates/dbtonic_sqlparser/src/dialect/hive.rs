

use crate::dialect::Dialect;

#[derive(Debug)]
pub struct HiveDialect {}

impl Dialect for HiveDialect {
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        (ch == '"') || (ch == '`')
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '$'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_ascii_lowercase()
            || ch.is_ascii_uppercase()
            || ch.is_ascii_digit()
            || ch == '_'
            || ch == '$'
            || ch == '{'
            || ch == '}'
    }

    fn supports_filter_during_aggregation(&self) -> bool {
        true
    }
}
