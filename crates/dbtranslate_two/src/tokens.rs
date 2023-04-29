use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// This is an enum that contains all of the different token types in dbtranslate.
/// Each token type represents a different type of token that can be parsed.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum TokenType {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Dash,
    Plus,
    Colon,
    DColon,
    Semicolon,
    Star,
    Backslash,
    Slash,
    Lt,
    Lte,
    Gt,
    Gte,
    Not,
    Eq,
    Neq,
    NullsafeEq,
    And,
    Or,
    Amp,
    DPipe,
    Pipe,
    Caret,
    Tilda,
    Arrow,
    DArrow,
    FArrow,
    Hash,
    HashArrow,
    DHashArrow,
    LrArrow,
    LtAt,
    AtGt,
    Dollar,
    Parameter,
    SessionParameter,
    National,
    Damp,

    /// Jina Tokens
    BlockStart,
    BlockEnd,

    Space,
    Break,

    String,
    Number,
    Identifier,
    Database,
    Column,
    ColumnDef,
    Schema,
    Table,
    Var,
    BitString,
    HexString,
    ByteString,

    /// Token types
    Bit,
    Boolean,
    Tinyint,
    Utinyint,
    Smallint,
    Usmallint,
    Int,
    Uint,
    Bigint,
    Ubigint,
    Float,
    Double,
    Decimal,
    Bigdecimal,
    Char,
    Nchar,
    Varchar,
    Nvarchar,
    Text,
    Mediumtext,
    Longtext,
    Mediumblob,
    Longblob,
    Binary,
    Varbinary,
    Json,
    Jsonb,
    Time,
    Timestamp,
    Timestamptz,
    Timestampltz,
    Datetime,
    Date,
    Uuid,
    Geography,
    Nullable,
    Geometry,
    Hllsketch,
    Hstore,
    Super,
    Serial,
    Smallserial,
    Bigserial,
    Xml,
    Uniqueidentifier,
    Money,
    Smallmoney,
    Rowversion,
    Image,
    Variant,
    Object,
    Inet,

    // Token keywords
    Alias,
    Alter,
    Always,
    All,
    Anti,
    Any,
    Apply,
    Array,
    Asc,
    Asof,
    AtTimeZone,
    AutoIncrement,
    Begin,
    Between,
    Both,
    Bucket,
    ByDefault,
    Cache,
    Cascade,
    Case,
    CharacterSet,
    ClusterBy,
    Collate,
    Command,
    Comment,
    Commit,
    Compound,
    Constraint,
    Create,
    Cross,
    Cube,
    CurrentDate,
    CurrentDatetime,
    CurrentRow,
    CurrentTime,
    CurrentTimestamp,
    CurrentUser,
    Default,
    Delete,
    Desc,
    Describe,
    Distinct,
    DistinctFrom,
    DistributeBy,
    Div,
    Drop,
    Else,
    End,
    Escape,
    Except,
    Execute,
    Exists,
    False,
    Fetch,
    Filter,
    Final,
    First,
    Following,
    For,
    ForeignKey,
    Format,
    From,
    Full,
    Function,
    Glob,
    Global,
    GroupBy,
    GroupingSets,
    Having,
    Hint,
    If,
    IgnoreNulls,
    ILike,
    ILikeAny,
    In,
    Index,
    Inner,
    Insert,
    Intersect,
    Interval,
    Into,
    Introducer,
    IRLike,
    Is,
    IsNull,
    Join,
    JoinMarker,
    Language,
    Lateral,
    Lazy,
    Leading,
    Left,
    Like,
    LikeAny,
    Limit,
    LoadData,
    Local,
    Map,
    MatchRecognize,
    Materialized,
    Merge,
    Mod,
    Natural,
    Next,
    NoAction,
    NotNull,
    Null,
    NullsFirst,
    NullsLast,
    Offset,
    On,
    Only,
    Options,
    OrderBy,
    Ordered,
    Ordinality,
    Outer,
    OutOf,
    Over,
    Overlaps,
    Overwrite,
    Partition,
    PartitionBy,
    Percent,
    Pivot,
    Placeholder,
    Pragma,
    Preceding,
    PrimaryKey,
    Procedure,
    Properties,
    PseudoType,
    Qualify,
    Quote,
    Range,
    Recursive,
    Replace,
    RespectNulls,
    Returning,
    References,
    Right,
    RLike,
    Rollback,
    Rollup,
    Row,
    Rows,
    Seed,
    Select,
    Semi,
    Separator,
    SerdeProperties,
    Set,
    Show,
    SimilarTo,
    Some,
    SortKey,
    SortBy,
    Struct,
    TableSample,
    Temporary,
    Top,
    Then,
    Trailing,
    True,
    Unbounded,
    Uncache,
    Union,
    Unlogged,
    Unnest,
    Unpivot,
    Update,
    Use,
    Using,
    Values,
    View,
    Volatile,
    When,
    Where,
    Window,
    With,
    WithTimeZone,
    WithLocalTimeZone,
    WithinGroup,
    WithoutTimeZone,
    Unique,
}

/// This is the overarching Token structure that contains all of the information
/// about each token. It contains the token type, the text, the line number,
/// the column number, the end number, and the comments.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub end: usize,
    pub comments: Vec<String>,
}

/// These are the associated functions of the Token struct
impl Token {

    /// This is the constructor function to create the Token
    pub fn new(
        token_type: TokenType,
        text: String,
        line: usize,
        col: usize,
        end: usize,
        comments: Vec<String>,
    ) -> Self {
        let size = text.len();
        let end = if end > 0 { end } else { size };

        Self {
            token_type,
            text,
            line,
            col,
            end,
            comments,
        }
    }

    /// This function is a method that calculates the starting position of the 
    /// token in the parsed text. It computes the starting position by 
    /// subtracting the length of the text field from the end field. 
    /// The start function does not modify the Token struct; it only calculates 
    /// and returns the value.
    pub fn start(&self) -> usize {
        self.end - self.text.len()
    }

    /// This function takes an i64 integer value, creates a new Token instance 
    /// with the TokenType::Number variant, and assigns the string representation 
    /// of the input number to the text field. It initializes other fields with 
    /// default values: line and col are set to 1, end is set to 0, and comments 
    /// is an empty vector. This function is used to create a Token instance 
    /// representing a number in the parsed text.
    pub fn number(number: i64) -> Self {
        Self {
            token_type: TokenType::Number,
            text: number.to_string(),
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    /// This function takes a String value, creates a new Token instance with 
    /// the TokenType::String variant, and assigns the input string to the text 
    /// field. Similar to the number function, it initializes other fields with 
    /// default values. This function is used to create a Token instance 
    /// representing a string in the parsed text.
    pub fn string(string: String) -> Self {
        Self {
            token_type: TokenType::String,
            text: string,
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    /// This function takes a String value, creates a new Token instance with 
    /// the TokenType::Identifier variant, and assigns the input string to the 
    /// text field. It initializes other fields with default values just like 
    /// the other functions. This function is used to create a Token instance 
    /// representing an identifier (e.g., a variable or column name) in the parsed text.
    pub fn identifier(identifier: String) -> Self {
        Self {
            token_type: TokenType::Identifier,
            text: identifier,
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    /// This function takes a String value, creates a new Token instance with 
    /// the TokenType::Var variant, and assigns the input string to the text 
    /// field. It initializes other fields with default values, similar to the 
    /// other functions. This function is used to create a Token instance 
    /// representing a variable in the parsed text.
    pub fn var(var: String) -> Self {
        Self {
            token_type: TokenType::Var,
            text: var,
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }
}

/// This is the Display implementation for the Token struct. It is used to
/// display the Token in a readable format.
impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let attributes = [
            format!("token_type: {:?}", self.token_type),
            format!("text: {}", self.text),
            format!("line: {}", self.line),
            format!("col: {}", self.col),
            format!("end: {}", self.end),
            format!("comments: {:?}", self.comments),
        ]
        .join(", ");
        write!(f, "<Token {}>", attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenType};

    /// This is a test for the number function of the Token Struct.
    /// It tests that the number function creates a Token instance with the
    /// correct TokenType::Number variant and the correct text field.
    #[test]
    fn test_number() {
        let number_token = Token::number(42);
        assert_eq!(number_token.token_type, TokenType::Number);
        assert_eq!(number_token.text, "42");
        assert_eq!(number_token.line, 1);
        assert_eq!(number_token.col, 1);
        assert_eq!(number_token.end, 0);
        assert!(number_token.comments.is_empty());
    }

    /// This is a test for the string function of the Token Struct.
    /// It tests that the string function creates a Token instance with the
    /// correct TokenType::String variant and the correct text field.
    #[test]
    fn test_string() {
        let string_token = Token::string("hello".to_string());
        assert_eq!(string_token.token_type, TokenType::String);
        assert_eq!(string_token.text, "hello");
        assert_eq!(string_token.line, 1);
        assert_eq!(string_token.col, 1);
        assert_eq!(string_token.end, 0);
        assert!(string_token.comments.is_empty());
    }

    /// This is a test for the identifier function of the Token Struct.
    /// It tests that the identifier function creates a Token instance with the
    /// correct TokenType::Identifier variant and the correct text field.
    #[test]
    fn test_identifier() {
        let identifier_token = Token::identifier("my_var".to_string());
        assert_eq!(identifier_token.token_type, TokenType::Identifier);
        assert_eq!(identifier_token.text, "my_var");
        assert_eq!(identifier_token.line, 1);
        assert_eq!(identifier_token.col, 1);
        assert_eq!(identifier_token.end, 0);
        assert!(identifier_token.comments.is_empty());
    }

    /// This is a test for the var function of the Token Struct.
    /// It tests that the var function creates a Token instance with the
    /// correct TokenType::Var variant and the correct text field.
    #[test]
    fn test_var() {
        let var_token = Token::var("my_var".to_string());
        assert_eq!(var_token.token_type, TokenType::Var);
        assert_eq!(var_token.text, "my_var");
        assert_eq!(var_token.line, 1);
        assert_eq!(var_token.col, 1);
        assert_eq!(var_token.end, 0);
        assert!(var_token.comments.is_empty());
    }

    /// This is a test for the start function of the Token Struct. 
    /// It tests that the start function calculates the correct starting
    /// position of the token in the parsed text.
    #[test]
    fn test_start() {
        let token = Token {
            token_type: TokenType::Identifier,
            text: "test".to_string(),
            line: 1,
            col: 1,
            end: 5,
            comments: vec![],
        };
        assert_eq!(token.start(), 1);
    }

}
