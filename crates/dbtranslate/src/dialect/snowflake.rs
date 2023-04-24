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

#[cfg(not(feature = "std"))]
use crate::alloc::string::ToString;
use crate::ast::Statement;
use crate::dialect::Dialect;
use crate::keywords::Keyword;
use crate::parser::{Parser, ParserError};
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Default)]
pub struct SnowflakeDialect;

impl Dialect for SnowflakeDialect {
    // see https://docs.snowflake.com/en/sql-reference/identifiers-syntax.html
    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_ascii_lowercase()
            || ch.is_ascii_uppercase()
            || ch.is_ascii_digit()
            || ch == '$'
            || ch == '_'
    }

    fn supports_within_after_array_aggregation(&self) -> bool {
        true
    }

    fn parse_statement(&self, parser: &mut Parser) -> Option<Result<Statement, ParserError>> {
        if parser.parse_keyword(Keyword::CREATE) {
            // possibly CREATE STAGE
            //[ OR  REPLACE ]
            let or_replace = parser.parse_keywords(&[Keyword::OR, Keyword::REPLACE]);
            //[ TEMPORARY ]
            let temporary = parser.parse_keyword(Keyword::TEMPORARY);

            let mut back = 1;
            if or_replace {
                back += 2
            }
            if temporary {
                back += 1
            }
            for _i in 0..back {
                parser.prev_token();
            }
        }
        None
    }
}