use std::fmt;

#[derive(Debug, Clone)]
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

/// This is an enum that contains all of the different error types in dbtranslate
#[derive(Debug, Clone)]
pub enum DbtranslateError {
    /// Represents an error that occurs when an unsupported operation is attempted.
    UnsupportedError,
    /// Represents an error that occurs during the parsing process. It contains 
    /// a ParseErrorDetails struct which includes a message and a list of 
    /// ParseErrorContext structs for more detailed information about each error.
    ParseError(ParseErrorDetails),
    ///  Represents an error related to tokenization.
    TokenError,
    /// Represents an error related to the schema.
    SchemaError,
    /// Represents an error that occurs during the execution of a query or operation.
    ExecuteError,
}

impl fmt::Display for DbtranslateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbtranslateError::UnsupportedError => write!(f, "Unsupported operation"),
            DbtranslateError::ParseError(details) => write!(f, "{}", details.message),
            DbtranslateError::TokenError => write!(f, "Token error"),
            DbtranslateError::SchemaError => write!(f, "Schema error"),
            DbtranslateError::ExecuteError => write!(f, "Execute error"),
        }
    }
}

/// This struct contains the message and a vector of ParseErrorContexts 
/// related to a ParseError.
#[derive(Debug, Clone)]
pub struct ParseErrorDetails {
    pub message: String,
    pub errors: Vec<ParseErrorContext>,
}

/// This struct Provides detailed information about a specific parse error, 
/// including description, line and column numbers, start and end context, 
/// and the related expression.
#[derive(Debug, Clone)]
pub struct ParseErrorContext {
    pub description: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub start_context: Option<String>,
    pub highlight: Option<String>,
    pub end_context: Option<String>,
    pub into_expression: Option<String>,
}

/// Takes a slice of DbtranslateError items and a maximum number of errors to 
/// include in the message. It returns a concatenated string representation of 
/// the error messages. If there are more errors than the maximum specified, it 
/// appends a message indicating how many more errors are remaining.
pub fn concat_messages(errors: &[DbtranslateError], maximum: usize) -> String {
    let mut msg = errors.iter().take(maximum).map(|e| format!("{}", e)).collect::<Vec<String>>();
    let remaining = errors.len() - maximum;
    if remaining > 0 {
        msg.push(format!("... and {} more", remaining));
    }
    msg.join("\n\n")
}

/// Takes a slice of ParseErrorDetails items and returns a flattened vector 
/// of ParseErrorContext structs. This function is useful for merging multiple 
/// parse errors into a single list of error contexts.
pub fn merge_errors(errors: &[ParseErrorDetails]) -> Vec<ParseErrorContext> {
    errors.iter().flat_map(|error| error.errors.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::DbtranslateError;

    /// Tests that the UnsupportedError variant of the DbtranslateError enum 
    /// returns what we expect when we call the to_string() method on it.
    #[test]
    fn test_unsupported_error() {
        let error = DbtranslateError::UnsupportedError;
        assert_eq!(error.to_string(), "Unsupported operation");
    }

    /// Tests that the ParseError variant of the DbtranslateError enum
    /// returns what we expect when we provide it with a ParseErrorDetails struct.
   #[test]
    fn test_parse_error() {
        let parse_error = DbtranslateError::ParseError(
            ParseErrorDetails {
                message: "Parse Error".to_string(),
                errors: vec![ParseErrorContext {
                    description: Some("Description".to_string()),
                    line: Some(1),
                    col: Some(1),
                    start_context: None,
                    highlight: None,
                    end_context: None,
                    into_expression: None,
                }],
            },
        );

        if let DbtranslateError::ParseError(details) = parse_error {
            assert_eq!(details.message, "Parse Error");
            assert_eq!(details.errors[0].description.as_ref().unwrap(), "Description");
        } else {
            panic!("Expected DbtranslateError::ParseError");
        }
    }

    /// Tests that the TokenError variant of the DbtranslateError enum 
    /// returns what we expect when we call the to_string() method on it.
    #[test]
    fn test_token_error() {
        let error = DbtranslateError::TokenError;
        assert_eq!(error.to_string(), "Token error");
    }

    /// Tests that the SchemaError variant of the DbtranslateError enum 
    /// returns what we expect when we call the to_string() method on it.
    #[test]
    fn test_schema_error() {
        let error = DbtranslateError::SchemaError;
        assert_eq!(error.to_string(), "Schema error");
    }

    /// Tests that the ExecuteError variant of the DbtranslateError enum 
    /// returns what we expect when we call the to_string() method on it.
    #[test]
    fn test_execute_error() {
        let error = DbtranslateError::ExecuteError;
        assert_eq!(error.to_string(), "Execute error");
    }

    /// Tests that the concat_messages() function returns what we expect when
    /// we pass it a slice of DbtranslateError items and a maximum number of
    /// errors to include in the message.
    #[test]
    fn test_concat_messages() {
        let errors = vec![
            DbtranslateError::UnsupportedError,
            DbtranslateError::TokenError,
            DbtranslateError::ExecuteError,
        ];
        let result = concat_messages(&errors, 2);
        assert_eq!(result, "Unsupported operation\n\nToken error\n\n... and 1 more");
    }

    /// Tests that the merge_errors() function returns what we expect when
    /// we pass it a slice of ParseErrorDetails items.
    #[test]
    fn test_merge_errors() {
        let errors = vec![
            ParseErrorDetails {
                message: "Error 1".to_string(),
                errors: vec![ParseErrorContext {
                    description: Some("Description 1".to_string()),
                    line: Some(1),
                    col: Some(1),
                    start_context: None,
                    highlight: None,
                    end_context: None,
                    into_expression: None,
                }],
            },
            ParseErrorDetails {
                message: "Error 2".to_string(),
                errors: vec![ParseErrorContext {
                    description: Some("Description 2".to_string()),
                    line: Some(2),
                    col: Some(2),
                    start_context: None,
                    highlight: None,
                    end_context: None,
                    into_expression: None,
                }],
            },
        ];
        let merged = merge_errors(&errors);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].description.as_ref().unwrap(), "Description 1");
        assert_eq!(merged[1].description.as_ref().unwrap(), "Description 2");
    }

}
