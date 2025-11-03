//! Skill validation tool

use std::path::Path;
use eyre::Result;

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn print(&self) {
        if self.valid {
            println!("✓ Skill is valid");
        } else {
            println!("✗ Skill has errors");
        }

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  ✗ {}", error);
            }
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &self.warnings {
                println!("  ⚠ {}", warning);
            }
        }
    }
}

/// Validate a skill file
pub fn validate_skill_file(path: &Path) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Check file exists
    if !path.exists() {
        result.add_error(format!("File not found: {}", path.display()));
        return Ok(result);
    }

    // Check file extension
    if path.extension().and_then(|s| s.to_str()) != Some("json") {
        result.add_warning("Skill files should have .json extension".to_string());
    }

    // Read and parse JSON
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            result.add_error(format!("Invalid JSON: {}", e));
            return Ok(result);
        }
    };

    // Validate required fields
    if !json["name"].is_string() {
        result.add_error("Missing or invalid 'name' field".to_string());
    }

    if !json["description"].is_string() {
        result.add_warning("Missing 'description' field".to_string());
    }

    if !json["implementation"].is_object() {
        result.add_error("Missing or invalid 'implementation' field".to_string());
    }

    // Validate parameters if present
    if let Some(params) = json["parameters"].as_array() {
        for (i, param) in params.iter().enumerate() {
            if !param["name"].is_string() {
                result.add_error(format!("Parameter {} missing 'name'", i));
            }
            if !param["type"].is_string() {
                result.add_error(format!("Parameter {} missing 'type'", i));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validate_valid_skill() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "test", "description": "Test", "implementation": {{"type": "command"}}}}"#).unwrap();

        let result = validate_skill_file(file.path()).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not json").unwrap();

        let result = validate_skill_file(file.path()).unwrap();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_name() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"description": "Test"}}"#).unwrap();

        let result = validate_skill_file(file.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }
}
