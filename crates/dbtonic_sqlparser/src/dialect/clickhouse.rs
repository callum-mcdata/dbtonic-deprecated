

use crate::dialect::Dialect;

#[derive(Debug)]
pub struct ClickHouseDialect {}

impl Dialect for ClickHouseDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        // See https://clickhouse.com/docs/en/sql-reference/syntax/#syntax-identifiers
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ch.is_ascii_digit()
    }
}
