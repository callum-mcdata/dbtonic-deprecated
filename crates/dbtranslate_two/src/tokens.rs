use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::collections::{HashMap, HashSet};
use std::collections::BTreeMap;


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
    pub start: usize,
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
        let start = end - size;

        Self {
            token_type,
            text,
            line,
            col,
            start,
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
            start: 0,
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
            start: 0,
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
            start: 0,
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
            start: 0,
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
            format!("start: {}", self.start()),
            format!("end: {}", self.end),
            format!("comments: {:?}", self.comments),
        ]
        .join(", ");
        write!(f, "<Token {}>", attributes)
    }
}

/// This function creates a hashmap of all the single tokens in the dbtranslate
/// It maps the single token to the TokenType. This is then used in the Tokenizer
/// to determine the TokenType.
pub fn single_tokens() -> BTreeMap<char, TokenType> {
    let single_tokens = maplit::btreemap! {
        '(' => TokenType::LParen,
        ')' => TokenType::RParen,
        '[' => TokenType::LBracket,
        ']' => TokenType::RBracket,
        '{' => TokenType::LBrace,
        '}' => TokenType::RBrace,
        '&' => TokenType::Amp,
        '^' => TokenType::Caret,
        ':' => TokenType::Colon,
        ',' => TokenType::Comma,
        '.' => TokenType::Dot,
        '-' => TokenType::Dash,
        '=' => TokenType::Eq,
        '>' => TokenType::Gt,
        '<' => TokenType::Lt,
        '%' => TokenType::Mod,
        '!' => TokenType::Not,
        '|' => TokenType::Pipe,
        '+' => TokenType::Plus,
        ';' => TokenType::Semicolon,
        '/' => TokenType::Slash,
        '\\' => TokenType::Backslash,
        '*' => TokenType::Star,
        '~' => TokenType::Tilda,
        '?' => TokenType::Placeholder,
        '@' => TokenType::Parameter,
        '\'' => TokenType::Quote,
        '`' => TokenType::Identifier,
        '\"' => TokenType::Identifier,
        '#' => TokenType::Hash,
    };

    single_tokens

}

/// This function creates a hashmap of all the keywords in dbtranslate
/// It maps the keyword to the TokenType. This is then used in the Tokenizer
/// to determine the TokenType.
pub fn keywords() -> HashMap<String, TokenType> {
    let keywords = maplit::hashmap! {
        "{{+".to_string() => TokenType::BlockStart,
        "{%".to_string() => TokenType::BlockStart,
        "{%-".to_string() => TokenType::BlockStart,
        "{{-".to_string() => TokenType::BlockStart,
        "-%}".to_string() => TokenType::BlockEnd,
        "%}".to_string() => TokenType::BlockEnd,
        "+}}".to_string() => TokenType::BlockEnd,
        "-}}".to_string() => TokenType::BlockEnd,
        "/*+".to_string() => TokenType::Hint,
        "==".to_string() => TokenType::Eq,
        "::".to_string() => TokenType::DColon,
        "||".to_string() => TokenType::DPipe,
        ">=".to_string() => TokenType::Gte,
        "<=".to_string() => TokenType::Lte,
        "<>".to_string() => TokenType::Neq,
        "!=".to_string() => TokenType::Neq,
        "<=>".to_string() => TokenType::NullsafeEq,
        "->".to_string() => TokenType::Arrow,
        "->>".to_string() => TokenType::DArrow,
        "=>".to_string() => TokenType::FArrow,
        "#>".to_string() => TokenType::HashArrow,
        "#>>".to_string() => TokenType::DHashArrow,
        "<->".to_string() => TokenType::LrArrow,
        "&&".to_string() => TokenType::Damp,
        "ALL".to_string() => TokenType::All,
        "ALWAYS".to_string() => TokenType::Always,
        "AND".to_string() => TokenType::And,
        "ANTI".to_string() => TokenType::Anti,
        "ANY".to_string() => TokenType::Any,
        "ASC".to_string() => TokenType::Asc,
        "AS".to_string() => TokenType::Alias,
        "AT TIME ZONE".to_string() => TokenType::AtTimeZone,
        "AUTOINCREMENT".to_string() => TokenType::AutoIncrement,
        "AUTO_INCREMENT".to_string() => TokenType::AutoIncrement,
        "BEGIN".to_string() => TokenType::Begin,
        "BETWEEN".to_string() => TokenType::Between,
        "BOTH".to_string() => TokenType::Both,
        "BUCKET".to_string() => TokenType::Bucket,
        "BY DEFAULT".to_string() => TokenType::ByDefault,
        "CACHE".to_string() => TokenType::Cache,
        "UNCACHE".to_string() => TokenType::Uncache,
        "CASE".to_string() => TokenType::Case,
        "CASCADE".to_string() => TokenType::Cascade,
        "CHARACTER SET".to_string() => TokenType::CharacterSet,
        "CLUSTER BY".to_string() => TokenType::ClusterBy,
        "COLLATE".to_string() => TokenType::Collate,
        "COLUMN".to_string() => TokenType::Column,
        "COMMIT".to_string() => TokenType::Commit,
        "COMPOUND".to_string() => TokenType::Compound,
        "CONSTRAINT".to_string() => TokenType::Constraint,
        "CREATE".to_string() => TokenType::Create,
        "CROSS".to_string() => TokenType::Cross,
        "CUBE".to_string() => TokenType::Cube,
        "CURRENT_DATE".to_string() => TokenType::CurrentDate,
        "CURRENT ROW".to_string() => TokenType::CurrentRow,
        "CURRENT_TIME".to_string() => TokenType::CurrentTime,
        "CURRENT_TIMESTAMP".to_string() => TokenType::CurrentTimestamp,
        "CURRENT_USER".to_string() => TokenType::CurrentUser,
        "DATABASE".to_string() => TokenType::Database,
        "DEFAULT".to_string() => TokenType::Default,
        "DELETE".to_string() => TokenType::Delete,
        "DESC".to_string() => TokenType::Desc,
        "DESCRIBE".to_string() => TokenType::Describe,
        "DISTINCT".to_string() => TokenType::Distinct,
        "DISTINCT FROM".to_string() => TokenType::DistinctFrom,
        "DISTRIBUTE BY".to_string() => TokenType::DistributeBy,
        "DIV".to_string() => TokenType::Div,
        "DROP".to_string() => TokenType::Drop,
        "ELSE".to_string() => TokenType::Else,
        "END".to_string() => TokenType::End,
        "ESCAPE".to_string() => TokenType::Escape,
        "EXCEPT".to_string() => TokenType::Except,
        "EXECUTE".to_string() => TokenType::Execute,
        "EXISTS".to_string() => TokenType::Exists,
        "FALSE".to_string() => TokenType::False,
        "FETCH".to_string() => TokenType::Fetch,
        "FILTER".to_string() => TokenType::Filter,
        "FIRST".to_string() => TokenType::First,
        "FULL".to_string() => TokenType::Full,
        "FUNCTION".to_string() => TokenType::Function,
        "FOLLOWING".to_string() => TokenType::Following,
        "FOR".to_string() => TokenType::For,
        "FOREIGN KEY".to_string() => TokenType::ForeignKey,
        "FORMAT".to_string() => TokenType::Format,
        "FROM".to_string() => TokenType::From,
        "GLOB".to_string() => TokenType::Glob,
        "GROUP BY".to_string() => TokenType::GroupBy,
        "GROUPING SETS".to_string() => TokenType::GroupingSets,
        "HAVING".to_string() => TokenType::Having,
        "IF".to_string() => TokenType::If,
        "ILIKE".to_string() => TokenType::ILike,
        "IGNORE NULLS".to_string() => TokenType::IgnoreNulls,
        "IN".to_string() => TokenType::In,
        "INDEX".to_string() => TokenType::Index,
        "INET".to_string() => TokenType::Inet,
        "INNER".to_string() => TokenType::Inner,
        "INSERT".to_string() => TokenType::Insert,
        "INTERVAL".to_string() => TokenType::Interval,
        "INTERSECT".to_string() => TokenType::Intersect,
        "INTO".to_string() => TokenType::Into,
        "IS".to_string() => TokenType::Is,
        "ISNULL".to_string() => TokenType::IsNull,
        "JOIN".to_string() => TokenType::Join,
        "LATERAL".to_string() => TokenType::Lateral,
        "LAZY".to_string() => TokenType::Lazy,
        "LEADING".to_string() => TokenType::Leading,
        "LEFT".to_string() => TokenType::Left,
        "LIKE".to_string() => TokenType::Like,
        "LIMIT".to_string() => TokenType::Limit,
        "LOAD DATA".to_string() => TokenType::LoadData,
        "LOCAL".to_string() => TokenType::Local,
        "MATERIALIZED".to_string() => TokenType::Materialized,
        "MERGE".to_string() => TokenType::Merge,
        "NATURAL".to_string() => TokenType::Natural,
        "NEXT".to_string() => TokenType::Next,
        "NO ACTION".to_string() => TokenType::NoAction,
        "NOT".to_string() => TokenType::Not,
        "NOTNULL".to_string() => TokenType::NotNull,
        "NULL".to_string() => TokenType::Null,
        "NULLS FIRST".to_string() => TokenType::NullsFirst,
        "NULLS LAST".to_string() => TokenType::NullsLast,
        "OBJECT".to_string() => TokenType::Object,
        "OFFSET".to_string() => TokenType::Offset,
        "ON".to_string() => TokenType::On,
        "ONLY".to_string() => TokenType::Only,
        "OPTIONS".to_string() => TokenType::Options,
        "OR".to_string() => TokenType::Or,
        "ORDER BY".to_string() => TokenType::OrderBy,
        "ORDINALITY".to_string() => TokenType::Ordinality,
        "OUTER".to_string() => TokenType::Outer,
        "OUT OF".to_string() => TokenType::OutOf,
        "OVER".to_string() => TokenType::Over,
        "OVERLAPS".to_string() => TokenType::Overlaps,
        "OVERWRITE".to_string() => TokenType::Overwrite,
        "PARTITION".to_string() => TokenType::Partition,
        "PARTITION BY".to_string() => TokenType::PartitionBy,
        "PARTITIONED BY".to_string() => TokenType::PartitionBy,
        "PARTITIONED_BY".to_string() => TokenType::PartitionBy,
        "PERCENT".to_string() => TokenType::Percent,
        "PIVOT".to_string() => TokenType::Pivot,
        "PRAGMA".to_string() => TokenType::Pragma,
        "PRECEDING".to_string() => TokenType::Preceding,
        "PRIMARY KEY".to_string() => TokenType::PrimaryKey,
        "PROCEDURE".to_string() => TokenType::Procedure,
        "QUALIFY".to_string() => TokenType::Qualify,
        "RANGE".to_string() => TokenType::Range,
        "RECURSIVE".to_string() => TokenType::Recursive,
        "REGEXP".to_string() => TokenType::RLike,
        "REPLACE".to_string() => TokenType::Replace,
        "RESPECT NULLS".to_string() => TokenType::RespectNulls,
        "REFERENCES".to_string() => TokenType::References,
        "RIGHT".to_string() => TokenType::Right,
        "RLIKE".to_string() => TokenType::RLike,
        "ROLLBACK".to_string() => TokenType::Rollback,
        "ROLLUP".to_string() => TokenType::Rollup,
        "ROW".to_string() => TokenType::Row,
        "ROWS".to_string() => TokenType::Rows,
        "SCHEMA".to_string() => TokenType::Schema,
        "SEED".to_string() => TokenType::Seed,
        "SELECT".to_string() => TokenType::Select,
        "SEMI".to_string() => TokenType::Semi,
        "SET".to_string() => TokenType::Set,
        "SHOW".to_string() => TokenType::Show,
        "SIMILAR TO".to_string() => TokenType::SimilarTo,
        "SOME".to_string() => TokenType::Some,
        "SORTKEY".to_string() => TokenType::SortKey,
        "SORT BY".to_string() => TokenType::SortBy,
        "TABLE".to_string() => TokenType::Table,
        "TABLESAMPLE".to_string() => TokenType::TableSample,
        "TEMP".to_string() => TokenType::Temporary,
        "TEMPORARY".to_string() => TokenType::Temporary,
        "THEN".to_string() => TokenType::Then,
        "TRUE".to_string() => TokenType::True,
        "TRAILING".to_string() => TokenType::Trailing,
        "UNBOUNDED".to_string() => TokenType::Unbounded,
        "UNION".to_string() => TokenType::Union,
        "UNLOGGED".to_string() => TokenType::Unlogged,
        "UNNEST".to_string() => TokenType::Unnest,
        "UNPIVOT".to_string() => TokenType::Unpivot,
        "UPDATE".to_string() => TokenType::Update,
        "USE".to_string() => TokenType::Use,
        "USING".to_string() => TokenType::Using,
        "UUID".to_string() => TokenType::Uuid,
        "VALUES".to_string() => TokenType::Values,
        "VIEW".to_string() => TokenType::View,
        "VOLATILE".to_string() => TokenType::Volatile,
        "WHEN".to_string() => TokenType::When,
        "WHERE".to_string() => TokenType::Where,
        "WINDOW".to_string() => TokenType::Window,
        "WITH".to_string() => TokenType::With,
        "WITH TIME ZONE".to_string() => TokenType::WithTimeZone,
        "WITH LOCAL TIME ZONE".to_string() => TokenType::WithLocalTimeZone,
        "WITHIN GROUP".to_string() => TokenType::WithinGroup,
        "WITHOUT TIME ZONE".to_string() => TokenType::WithoutTimeZone,
        "APPLY".to_string() => TokenType::Apply,
        "ARRAY".to_string() => TokenType::Array,
        "BIT".to_string() => TokenType::Bit,
        "BOOL".to_string() => TokenType::Boolean,
        "BOOLEAN".to_string() => TokenType::Boolean,
        "BYTE".to_string() => TokenType::Tinyint,
        "TINYINT".to_string() => TokenType::Tinyint,
        "SHORT".to_string() => TokenType::Smallint,
        "SMALLINT".to_string() => TokenType::Smallint,
        "INT2".to_string() => TokenType::Smallint,
        "INTEGER".to_string() => TokenType::Int,
        "INT".to_string() => TokenType::Int,
        "INT4".to_string() => TokenType::Int,
        "LONG".to_string() => TokenType::Bigint,
        "BIGINT".to_string() => TokenType::Bigint,
        "INT8".to_string() => TokenType::Bigint,
        "DEC".to_string() => TokenType::Decimal,
        "DECIMAL".to_string() => TokenType::Decimal,
        "BIGDECIMAL".to_string() => TokenType::Bigdecimal,
        "BIGNUMERIC".to_string() => TokenType::Bigdecimal,
        "MAP".to_string() => TokenType::Map,
        "NULLABLE".to_string() => TokenType::Nullable,
        "NUMBER".to_string() => TokenType::Decimal,
        "NUMERIC".to_string() => TokenType::Decimal,
        "FIXED".to_string() => TokenType::Decimal,
        "REAL".to_string() => TokenType::Float,
        "FLOAT".to_string() => TokenType::Float,
        "FLOAT4".to_string() => TokenType::Float,
        "FLOAT8".to_string() => TokenType::Double,
        "DOUBLE".to_string() => TokenType::Double,
        "DOUBLE PRECISION".to_string() => TokenType::Double,
        "JSON".to_string() => TokenType::Json,
        "CHAR".to_string() => TokenType::Char,
        "CHARACTER".to_string() => TokenType::Char,
        "NCHAR".to_string() => TokenType::Nchar,
        "VARCHAR".to_string() => TokenType::Varchar,
        "VARCHAR2".to_string() => TokenType::Varchar,
        "NVARCHAR".to_string() => TokenType::Nvarchar,
        "NVARCHAR2".to_string() => TokenType::Nvarchar,
        "STR".to_string() => TokenType::Text,
        "STRING".to_string() => TokenType::Text,
        "TEXT".to_string() => TokenType::Text,
        "CLOB".to_string() => TokenType::Text,
        "LONGVARCHAR".to_string() => TokenType::Text,
        "BINARY".to_string() => TokenType::Binary,
        "BLOB".to_string() => TokenType::Varbinary,
        "BYTEA".to_string() => TokenType::Varbinary,
        "VARBINARY".to_string() => TokenType::Varbinary,
        "TIME".to_string() => TokenType::Time,
        "TIMESTAMP".to_string() => TokenType::Timestamp,
        "TIMESTAMPTZ".to_string() => TokenType::Timestamptz,
        "TIMESTAMPLTZ".to_string() => TokenType::Timestampltz,
        "DATE".to_string() => TokenType::Date,
        "DATETIME".to_string() => TokenType::Datetime,
        "UNIQUE".to_string() => TokenType::Unique,
        "STRUCT".to_string() => TokenType::Struct,
        "VARIANT".to_string() => TokenType::Variant,
        "ALTER".to_string() => TokenType::Alter,
        "ALTER AGGREGATE".to_string() => TokenType::Command,
        "ALTER DEFAULT".to_string() => TokenType::Command,
        "ALTER DOMAIN".to_string() => TokenType::Command,
        "ALTER ROLE".to_string() => TokenType::Command,
        "ALTER RULE".to_string() => TokenType::Command,
        "ALTER SEQUENCE".to_string() => TokenType::Command,
        "ALTER TYPE".to_string() => TokenType::Command,
        "ALTER USER".to_string() => TokenType::Command,
        "ALTER VIEW".to_string() => TokenType::Command,
        "ANALYZE".to_string() => TokenType::Command,
        "CALL".to_string() => TokenType::Command,
        "COMMENT".to_string() => TokenType::Comment,
        "COPY".to_string() => TokenType::Command,
        "EXPLAIN".to_string() => TokenType::Command,
        "GRANT".to_string() => TokenType::Command,
        "OPTIMIZE".to_string() => TokenType::Command,
        "PREPARE".to_string() => TokenType::Command,
        "TRUNCATE".to_string() => TokenType::Command,
        "VACUUM".to_string() => TokenType::Command,
    };

    keywords
}

/// This function creates a hashmap of all the white space tokens in dbtranslate
/// It maps the white space to the TokenType. This is then used in the Tokenizer
/// to determine the TokenType.
pub fn white_space() -> BTreeMap<char, TokenType> {
    let white_space = maplit::btreemap! {
        ' ' => TokenType::Space,
        '\t' => TokenType::Space,
        '\n' => TokenType::Break,
        '\r' => TokenType::Break,
    };
    white_space
}

pub fn comment_tokens() -> HashMap<String, Option<String>> {

    let comment_tokens = maplit::hashmap! {
        "--".to_string() => None,
        "/*".to_string() => Some("*/".to_string()),
        "{#".to_string() => Some("#}".to_string()),
    };

    comment_tokens
}

pub fn commands() -> HashSet<TokenType> {

    let commands: HashSet<TokenType> = [
        TokenType::Command,
        TokenType::Execute,
        TokenType::Fetch,
        TokenType::Show,
    ].iter().cloned().collect();

    commands
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
            start: 1,
            end: 5,
            comments: vec![],
        };
        assert_eq!(token.start, 1);
    }

}
