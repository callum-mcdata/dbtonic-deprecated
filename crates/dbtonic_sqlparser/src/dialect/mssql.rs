

use crate::dialect::Dialect;

#[derive(Debug)]
pub struct MsSqlDialect {}

impl Dialect for MsSqlDialect {
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"' || ch == '['
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        // See https://docs.microsoft.com/en-us/sql/relational-databases/databases/database-identifiers?view=sql-server-2017#rules-for-regular-identifiers
        ch.is_alphabetic() || ch == '_' || ch == '#' || ch == '@'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_alphabetic()
            || ch.is_ascii_digit()
            || ch == '@'
            || ch == '$'
            || ch == '#'
            || ch == '_'
    }
}
