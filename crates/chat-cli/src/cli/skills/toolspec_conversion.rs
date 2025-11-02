use crate::cli::chat::tools::ToolSpec;

pub trait ToToolSpec {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_error_display() {
        let err = ConversionError::MissingField("name".to_string());
        assert_eq!(err.to_string(), "Missing required field: name");

        let err = ConversionError::InvalidSchema("bad type".to_string());
        assert_eq!(err.to_string(), "Invalid schema: bad type");
    }
}
