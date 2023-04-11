

//! AST types specific to loading and unloading syntax, like one available in Snowflake which
//! contains: STAGE ddl operations, PUT upload or COPY INTO
//! See [this page](https://docs.snowflake.com/en/sql-reference/commands-data-loading) for more details.

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Formatter;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "visitor")]
use sqlparser_derive::{Visit, VisitMut};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
pub struct StageParamsObject {
    pub url: Option<String>,
    pub encryption: DataLoadingOptions,
    pub endpoint: Option<String>,
    pub storage_integration: Option<String>,
    pub credentials: DataLoadingOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
pub struct DataLoadingOptions {
    pub options: Vec<DataLoadingOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
pub enum DataLoadingOptionType {
    STRING,
    BOOLEAN,
    ENUM,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
pub struct DataLoadingOption {
    pub option_name: String,
    pub option_type: DataLoadingOptionType,
    pub value: String,
}

impl fmt::Display for StageParamsObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let url = &self.url.as_ref();
        let storage_integration = &self.storage_integration.as_ref();
        let endpoint = &self.endpoint.as_ref();

        if url.is_some() {
            write!(f, " URL='{}'", url.unwrap())?;
        }
        if storage_integration.is_some() {
            write!(f, " STORAGE_INTEGRATION={}", storage_integration.unwrap())?;
        }
        if endpoint.is_some() {
            write!(f, " ENDPOINT='{}'", endpoint.unwrap())?;
        }
        if !self.credentials.options.is_empty() {
            write!(f, " CREDENTIALS=({})", self.credentials)?;
        }
        if !self.encryption.options.is_empty() {
            write!(f, " ENCRYPTION=({})", self.encryption)?;
        }

        Ok(())
    }
}

impl fmt::Display for DataLoadingOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.options.is_empty() {
            for option in &self.options {
                write!(f, "{}", option)?;
                if !option.eq(self.options.last().unwrap()) {
                    write!(f, " ")?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for DataLoadingOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.option_type {
            DataLoadingOptionType::STRING => {
                write!(f, "{}='{}'", self.option_name, self.value)?;
            }
            DataLoadingOptionType::ENUM => {
                // single quote is omitted
                write!(f, "{}={}", self.option_name, self.value)?;
            }
            DataLoadingOptionType::BOOLEAN => {
                // single quote is omitted
                write!(f, "{}={}", self.option_name, self.value)?;
            }
        }
        Ok(())
    }
}
