#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    // Base types 
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
    
    // Block Types
    BlockStart,
    BlockEnd,

    // Jinja Objects & Iterators
    JinjaStart,
    JinjaEnd,
    JinjaIteratorStart,
    JinjaIteratorEnd,

    // Spacing Types
    Space,
    Break,

    // Object Types
    String,
    Number,
    Identifier,
    Database,
    Column,
    ColumnDef,
    Schema,
    Table,
    BitString,
    HexString,
    ByteString,

    // Data Types
    Bit,
    Boolean,
    TinyInt,
    UTinyInt,
    SmallInt,
    Int,
    UInt,
    BigInt,
    UBigInt,
    Float,
    Double,
    Decimal,
    Char,
    NChar,
    VarChar,
    NVarChar,
    Text,
    MediumText,
    LongText,
    MediumBlob,
    LongBlob,
    Binary,
    VarBinary,
    Json,
    JsonB,
    Time,
    TimeStamp,
    TimeStampTZ,
    TimeStampLTZ,
    DateTime,
    Date,
    UUID,
    Geography,
    Nullable,
    Geometry,
    HLLSketch,
    HStore,
    Super,
    Seriel,
    SmallSeriel,
    BigSeriel,
    XML,
    UniqueIdentifier,
    Money,
    SmallMoney,
    RowVersion,
    Image,
    Variant,
    Object,
    Inet,

    // dbt Keywords
    Source,
    Ref,
    If,
    EndIf,
    For,
    EndFor,
    Var,
    Set,

    // Keywords
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
    Show,
    SimilarTo,
    Some,
    Sortkey,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub end: usize,
    pub comments: Vec<String>,
}

impl Token {
    pub fn number(number: i32) -> Token {
        Token {
            token_type: TokenType::Number,
            text: number.to_string(),
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    pub fn string(string: &str) -> Token {
        Token {
            token_type: TokenType::String,
            text: string.to_string(),
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    pub fn identifier(identifier: &str) -> Token {
        Token {
            token_type: TokenType::Identifier,
            text: identifier.to_string(),
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    pub fn var(var: &str) -> Token {
        Token {
            token_type: TokenType::Var,
            text: var.to_string(),
            line: 1,
            col: 1,
            end: 0,
            comments: vec![],
        }
    }

    pub fn start(&self) -> usize {
        self.end.saturating_sub(self.text.len())
    }
}


#[cfg(test)]
mod tests {
    use super::{Token, TokenType};

    #[test]
    fn test_number_token() {
        let number_token = Token::number(42);
        assert_eq!(number_token.token_type, TokenType::Number);
        assert_eq!(number_token.text, "42");
    }

    #[test]
    fn test_string_token() {
        let string_token = Token::string("hello");
        assert_eq!(string_token.token_type, TokenType::String);
        assert_eq!(string_token.text, "hello");
    }

    #[test]
    fn test_identifier_token() {
        let identifier_token = Token::identifier("my_ident");
        assert_eq!(identifier_token.token_type, TokenType::Identifier);
        assert_eq!(identifier_token.text, "my_ident");
    }

    #[test]
    fn test_var_token() {
        let var_token = Token::var("my_var");
        assert_eq!(var_token.token_type, TokenType::Var);
        assert_eq!(var_token.text, "my_var");
    }

    #[test]
    fn test_start() {
        let mut token = Token::number(42);
        token.end = 10;
        assert_eq!(token.start(), 8);
    }

}