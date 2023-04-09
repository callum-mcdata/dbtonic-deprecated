use std::collections::HashMap;
use std::collections::HashSet;
use crate::tokenizer::trie::Trie;
use crate::tokenizer::tokens::TokenType;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    pub fn is_right(&self) -> bool {
        !self.is_left()
    }

    pub fn left(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    pub fn right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(r) => Some(r),
        }
    }
}

pub struct Tokenizer {
    pub single_tokens: HashMap<char, TokenType>,
    pub keywords: HashMap<String, TokenType>,
    pub quotes: HashMap<String, String>,
    pub bit_strings: HashMap<String, String>,
    pub hex_strings: HashMap<String, String>,
    pub byte_strings: HashMap<String, String>,
    pub identifiers: HashMap<String, String>,
    pub string_escapes: HashSet<String>,
    pub identifier_escapes: HashSet<String>,
    pub comments: HashMap<String, Option<String>>,
    pub keyword_trie: Trie,
}

impl Tokenizer {
    const SINGLE_TOKENS: [(char, TokenType); 30] = [
        ('(', TokenType::LParen),
        (')', TokenType::RParen),
        ('[', TokenType::LBracket),
        (']', TokenType::RBracket),
        ('{', TokenType::LBrace),
        ('}', TokenType::RBrace),
        ('&', TokenType::Amp),
        ('^', TokenType::Caret),
        (':', TokenType::Colon),
        (',', TokenType::Comma),
        ('.', TokenType::Dot),
        ('-', TokenType::Dash),
        ('=', TokenType::Eq),
        ('>', TokenType::Gt),
        ('<', TokenType::Lt),
        ('%', TokenType::Mod),
        ('!', TokenType::Not),
        ('|', TokenType::Pipe),
        ('+', TokenType::Plus),
        (';', TokenType::Semicolon),
        ('/', TokenType::Slash),
        ('\\', TokenType::Backslash),
        ('*', TokenType::Star),
        ('~', TokenType::Tilda),
        ('?', TokenType::Placeholder),
        ('@', TokenType::Parameter),
        ('\'', TokenType::Quote),
        ('`', TokenType::Identifier),
        ('"', TokenType::Identifier),
        ('#', TokenType::Hash),
    ];
    const QUOTES: [(&str, &str); 1] = [("''", "")];
    const BIT_STRINGS: [(&str, &str); 0] = [];
    const HEX_STRINGS: [(&str, &str); 0] = [];
    const BYTE_STRINGS: [(&str, &str); 0] = [];
    const IDENTIFIERS: [(&str, &str); 1] = [("\"", "")];
    const STRING_ESCAPES: [&str; 1] = ["'"];
    const _STRING_ESCAPES: std::collections::HashSet<&str> = std::array::IntoIter::new(&[]).collect();
    const IDENTIFIER_ESCAPES: [&str; 1] = ["\""];
    const _IDENTIFIER_ESCAPES: std::collections::HashSet<&str> = std::array::IntoIter::new(&[]).collect();

    // We use r# to tell rust that these can share words
    // that are reserved for Rust
    const KEYWORDS: HashMap<&'static str, TokenType> = [
        (r#"{{"#, TokenType::JinjaStart), 
        (r#"}}"#, TokenType::JinjaEnd), 
        (r#"{%"#, TokenType::JinjaIteratorStart), 
        (r#"%}"#, TokenType::JinjaIteratorEnd), 
        (r#"SOURCE"#, TokenType::Source), 
        (r#"REF"#, TokenType::Ref), 
        (r#"IF"#, TokenType::If), 
        (r#"ENDIF"#, TokenType::EndIf), 
        (r#"FOR"#, TokenType::For), 
        (r#"ENDFOR"#, TokenType::EndFor), 
        (r#"VAR"#, TokenType::Var), 
        (r#"SET"#, TokenType::Set), 
        (r#"/*+"#, TokenType::Hint),
        (r#"=="#, TokenType::Eq),
        (r#"::"#, TokenType::DColon),
        (r#"||"#, TokenType::DPipe),
        (r#">="#, TokenType::Gte),
        (r#"<="#, TokenType::Lte),
        (r#"<>"#, TokenType::Neq),
        (r#"!="#, TokenType::Neq),
        (r#"<=>"#, TokenType::NullsafeEq),
        (r#"->"#, TokenType::Arrow),
        (r#"->>"#, TokenType::DArrow),
        (r#"=>"#, TokenType::FArrow),
        (r#"#>"#, TokenType::HashArrow),
        (r#"#>>"#, TokenType::DHashArrow),
        (r#"<->"#, TokenType::LrArrow),
        (r#"&&"#, TokenType::Damp),
        (r#"ALL"#, TokenType::All),
        (r#"ALWAYS"#, TokenType::Always),
        (r#"AND"#, TokenType::And),
        (r#"ANTI"#, TokenType::Anti),
        (r#"ANY"#, TokenType::Any),
        (r#"ASC"#, TokenType::Asc),
        (r#"AS"#, TokenType::Alias),
        (r#"AT TIME ZONE"#, TokenType::AtTimeZone),
        (r#"AUTOINCREMENT"#, TokenType::AutoIncrement),
        (r#"AUTO_INCREMENT"#, TokenType::AutoIncrement),
        (r#"BEGIN"#, TokenType::Begin),
        (r#"BETWEEN"#, TokenType::Between),
        (r#"BOTH"#, TokenType::Both),
        (r#"BUCKET"#, TokenType::Bucket),
        (r#"BY DEFAULT"#, TokenType::ByDefault),
        (r#"CACHE"#, TokenType::Cache),
        (r#"UNCACHE"#, TokenType::Uncache),
        (r#"CASE"#, TokenType::Case),
        (r#"CASCADE"#, TokenType::Cascade),
        (r#"CHARACTER SET"#, TokenType::CharacterSet),
        (r#"CLUSTER BY"#, TokenType::ClusterBy),
        (r#"COLLATE"#, TokenType::Collate),
        (r#"COLUMN"#, TokenType::Column),
        (r#"COMMIT"#, TokenType::Commit),
        (r#"COMPOUND"#, TokenType::Compound),
        (r#"CONSTRAINT"#, TokenType::Constraint),
        (r#"CREATE"#, TokenType::Create),
        (r#"CROSS"#, TokenType::Cross),
        (r#"CUBE"#, TokenType::Cube),
        (r#"CURRENT_DATE"#, TokenType::CurrentDate),

        (r#"CurrentDatetime"#,TokenType::CurrentDatetime),
        (r#"CurrentRow"#,TokenType::CurrentRow),
        (r#"CurrentTime"#,TokenType::CurrentTime),
        (r#"CurrentTimestamp"#,TokenType::CurrentTimestamp),
        (r#"CurrentUser"#,TokenType::CurrentUser),
        (r#"Default"#,TokenType::Default),
        (r#"Delete"#,TokenType::Delete),
        (r#"Desc"#,TokenType::Desc),
        (r#"Describe"#,TokenType::Describe),
        (r#"Distinct"#,TokenType::Distinct),
        (r#"DistinctFrom"#,TokenType::DistinctFrom),
        (
            r#"DistributeBy"#,TokenType::DistributeBy),
        (
            r#"Div"#,TokenType::Div),
        (
            r#"Drop"#,TokenType::Drop),
        (
            r#"Else"#,TokenType::Else),
        (
            r#"End"#,TokenType::End),
        (
            r#"Escape"#,TokenType::Escape),
        (
            r#"Except"#,TokenType::Except),
        (
            r#"Execute"#,TokenType::Execute),
        (
            r#"Exists"#,TokenType::Exists),
        (
            r#"False"#,TokenType::False),
        (
            r#"Fetch"#,TokenType::Fetch),
        (
            r#"Filter"#,TokenType::Filter),
        (
            r#"Final"#,TokenType::Final),
        (
            r#"First"#,TokenType::First),
        (
            r#"Following"#,TokenType::Following),
        (
            r#"ForeignKey"#,TokenType::ForeignKey),
        (
            r#"Format"#,TokenType::Format),
        (
            r#"From"#,TokenType::From),
        (
            r#"Full"#,TokenType::Full),
        (
            r#"Function"#,TokenType::Function),
        (
            r#"Glob"#,TokenType::Glob),
        (
            r#"Global"#,TokenType::Global),
        (
            r#"GroupBy"#,TokenType::GroupBy),
        (
            r#"GroupingSets"#,TokenType::GroupingSets),
        (
            r#"Having"#,TokenType::Having),
        (
            r#"Hint"#,TokenType::Hint),
        (
            r#"IgnoreNulls"#,TokenType::IgnoreNulls),
        (
            r#"ILike"#,TokenType::ILike),
        (
            r#"ILikeAny"#,TokenType::ILikeAny),
        (
            r#"In"#,TokenType::In),
        (
            r#"Index"#,TokenType::Index),
        (
            r#"Inner"#,TokenType::Inner),
        (
            r#"Insert"#,TokenType::Insert),
        (
            r#"Intersect"#,TokenType::Intersect),
        (
            r#"Interval"#,TokenType::Interval),
        (
            r#"Into"#,TokenType::Into),
        (
            r#"Introducer"#,TokenType::Introducer),
        (
            r#"IRLike"#,TokenType::IRLike),
        (
            r#"Is"#,TokenType::Is),
        (
            r#"IsNull"#,TokenType::IsNull),
        (
            r#"Join"#,TokenType::Join),
        (
            r#"JoinMarker"#,TokenType::JoinMarker),
        (
            r#"Language"#,TokenType::Language),
        (
            r#"Lateral"#,TokenType::Lateral)


    ].iter().cloned().collect();


    fn delimeter_list_to_dict(list: Vec<Either<String, (String, String)>>) -> HashMap<String, String> {
        list.into_iter()
            .map(|item| match item {
                Either::Left(s) => (s.clone(), s),
                Either::Right((k, v)) => (k, v),
            })
            .collect()
    }

    pub fn create(
        quotes: Vec<Either<String, (String, String)>>,
        bit_strings: Vec<Either<String, (String, String)>>,
        hex_strings: Vec<Either<String, (String, String)>>,
        byte_strings: Vec<Either<String, (String, String)>>,
        identifiers: Vec<Either<String, (String, String)>>,
        string_escapes: Vec<String>,
        identifier_escapes: Vec<String>,
        comments: Vec<Either<String, (String, String)>>,
        keywords: HashMap<String, TokenType>,
        single_tokens: HashMap<char, TokenType>
    ) -> Self{
        let single_tokens_map: HashMap<char, TokenType> =Self::SINGLE_TOKENS.iter().cloned().collect();
        let quotes_map = Self::delimeter_list_to_dict(quotes);
        let bit_strings_map = Self::delimeter_list_to_dict(bit_strings);
        let hex_strings_map = Self::delimeter_list_to_dict(hex_strings);
        let byte_strings_map = Self::delimeter_list_to_dict(byte_strings);
        let identifiers_map = Self::delimeter_list_to_dict(identifiers);

        let string_escapes_set = string_escapes.into_iter().collect();
        let identifier_escapes_set = identifier_escapes.into_iter().collect();

        let comments_map = comments
            .into_iter()
            .map(|item| match item {
                Either::Left(s) => (s, None),
                Either::Right((k, v)) => (k, Some(v)),
            })
            .collect();

        let mut keyword_trie = Trie::create();
        for key in keywords.keys() {
            keyword_trie.insert(key);
        }


        // Add additional tokens to the trie based on the provided data

        Tokenizer {
            quotes: quotes_map,
            bit_strings: bit_strings_map,
            hex_strings: hex_strings_map,
            byte_strings: byte_strings_map,
            identifiers: identifiers_map,
            string_escapes: string_escapes_set,
            identifier_escapes: identifier_escapes_set,
            comments: comments_map,
            keyword_trie,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tokenizer, Either, TokenType};
    use std::collections::{HashMap};

    #[test]
    fn test_create_tokenizer() {
        let quotes = vec![
            Either::Left(String::from("'")),
            Either::Right((String::from("\""), String::from("\""))),
        ];
        let bit_strings = vec![Either::Left(String::from("B"))];
        let hex_strings = vec![Either::Left(String::from("X"))];
        let byte_strings = vec![Either::Left(String::from("b"))];
        let identifiers = vec![Either::Left(String::from("N"))];
        let string_escapes = vec![String::from("E")];
        let identifier_escapes = vec![String::from("I")];
        let comments = vec![Either::Left(String::from("--"))];

        let mut keywords = HashMap::new();
        keywords.insert(String::from("SELECT"), TokenType::Select);

        let tokenizer = _Tokenizer::create(
            quotes,
            bit_strings,
            hex_strings,
            byte_strings,
            identifiers,
            string_escapes,
            identifier_escapes,
            comments,
            keywords,);

        // Verify that the tokenizer data is stored correctly
        assert_eq!(tokenizer.quotes.len(), 2);
        assert_eq!(tokenizer.bit_strings.len(), 1);
        assert_eq!(tokenizer.hex_strings.len(), 1);
        assert_eq!(tokenizer.byte_strings.len(), 1);
        assert_eq!(tokenizer.identifiers.len(), 1);
        assert_eq!(tokenizer.string_escapes.len(), 1);
        assert_eq!(tokenizer.identifier_escapes.len(), 1);
        assert_eq!(tokenizer.comments.len(), 1);

        // Verify that the keyword trie contains the keyword
        assert!(tokenizer.keyword_trie.contains("SELECT"));
    }
}
