use std::fmt;

#[derive(Debug)]
pub enum ErrorLevel {
    /// Ignore all errors.
    Ignore,
    /// Warn about errors.
    Warn,
    /// Collect errors and raise an exception
    Raise,
    /// Immediately raise an exception on the first error found.
    Immediate,
}

#[derive(Debug)]
pub enum DbtonicError {
    UnsupportedError,
    ParseError(ParseErrorDetails),
    TokenError,
    SchemaError,
    ExecuteError,
}

#[derive(Debug)]
pub struct ParseErrorDetails {
    pub message: String,
    pub errors: Vec<ParseErrorContext>,
}

#[derive(Debug)]
pub struct ParseErrorContext {
    pub description: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub start_context: Option<String>,
    pub highlight: Option<String>,
    pub end_context: Option<String>,
    pub into_expression: Option<String>,
}

impl fmt::Display for DbtonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbtonicError::UnsupportedError => write!(f, "Unsupported operation"),
            DbtonicError::ParseError(details) => write!(f, "{}", details.message),
            DbtonicError::TokenError => write!(f, "Token error"),
            DbtonicError::SchemaError => write!(f, "Schema error"),
            DbtonicError::ExecuteError => write!(f, "Execute error"),
        }
    }
}

pub fn concat_messages(errors: &[SqlglotError], maximum: usize) -> String {
    let mut msg = errors.iter().take(maximum).map(|e| format!("{}", e)).collect::<Vec<String>>();
    let remaining = errors.len() - maximum;
    if remaining > 0 {
        msg.push(format!("... and {} more", remaining));
    }
    msg.join("\n\n")
}

pub fn merge_errors(errors: &[ParseErrorDetails]) -> Vec<ParseErrorContext> {
    errors.iter().flat_map(|error| error.errors.clone()).collect()
}