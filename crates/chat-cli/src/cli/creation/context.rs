//! Context intelligence for smart defaults and project-aware suggestions

use crate::cli::creation::{
    CreationType, CreationDefaults, ProjectType, ValidationResult, SkillType, CommandType
};
use eyre::Result;
use std::path::{Path, PathBuf};
use std::fs;

/// Context analyzer that provides smart defaults and suggestions
#[derive(Debug)]
pub struct CreationContext {
    current_dir: PathBuf,
    project_type: Option<ProjectType>,
    existing_skills: Vec<String>,
    existing_commands: Vec<String>,
    existing_agents: Vec<String>,
}

impl CreationContext {
    pub fn new(current_dir: &Path) -> Result<Self> {
        let mut context = Self {
            current_dir: current_dir.to_path_buf(),
            project_type: None,
            existing_skills: Vec::new(),
            existing_commands: Vec::new(),
            existing_agents: Vec::new(),
        };

        context.analyze_project_type();
        context.load_existing_artifacts()?;

        Ok(context)
    }

    pub fn suggest_defaults(&self, creation_type: &CreationType) -> CreationDefaults {
        let mut defaults = CreationDefaults::default();

        match creation_type {
            CreationType::Skill => {
                defaults.skill_type = self.suggest_skill_type();
                defaults.command = self.suggest_skill_command();
                defaults.description = self.generate_skill_description();
            }
            CreationType::CustomCommand => {
                defaults.command_type = self.suggest_command_type();
                defaults.description = self.generate_command_description();
            }
            CreationType::Agent => {
                defaults.mcp_servers = self.suggest_mcp_servers();
                defaults.description = self.generate_agent_description();
            }
        }

        defaults
    }

    pub fn validate_name(&self, name: &str, creation_type: &CreationType) -> ValidationResult {
        // Check if name is valid format
        if name.is_empty() {
            return ValidationResult::invalid(
                "Name cannot be empty",
                "my-skill"
            );
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            let suggestion: String = name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            
            return ValidationResult::invalid(
                &format!("Invalid characters in name '{}'", name),
                &suggestion
            );
        }

        // Check if name already exists
        let exists = match creation_type {
            CreationType::Skill => self.existing_skills.contains(&name.to_string()),
            CreationType::CustomCommand => self.existing_commands.contains(&name.to_string()),
            CreationType::Agent => self.existing_agents.contains(&name.to_string()),
        };

        if exists {
            return ValidationResult::invalid(
                &format!("{} '{}' already exists", self.format_type(creation_type), name),
                &format!("Use 'force' mode or try '{}-2'", name)
            );
        }

        ValidationResult::valid()
    }

    pub fn suggest_similar_names(&self, name: &str) -> Vec<String> {
        let mut all_names = Vec::new();
        all_names.extend(&self.existing_skills);
        all_names.extend(&self.existing_commands);
        all_names.extend(&self.existing_agents);

        all_names
            .into_iter()
            .filter(|existing| {
                // Simple similarity check
                existing.contains(name) || name.contains(existing.as_str()) ||
                self.levenshtein_distance(name, existing) <= 2
            })
            .cloned()
            .collect()
    }

    fn analyze_project_type(&mut self) {
        // Check for Python project
        if self.file_exists("requirements.txt") || 
           self.file_exists("pyproject.toml") ||
           self.file_exists("setup.py") ||
           self.has_files_with_extension("py") {
            self.project_type = Some(ProjectType::Python);
            return;
        }

        // Check for JavaScript/Node project
        if self.file_exists("package.json") ||
           self.file_exists("yarn.lock") ||
           self.has_files_with_extension("js") ||
           self.has_files_with_extension("ts") {
            self.project_type = Some(ProjectType::JavaScript);
            return;
        }

        // Check for Rust project
        if self.file_exists("Cargo.toml") ||
           self.has_files_with_extension("rs") {
            self.project_type = Some(ProjectType::Rust);
            return;
        }

        // Check for Go project
        if self.file_exists("go.mod") ||
           self.file_exists("go.sum") ||
           self.has_files_with_extension("go") {
            self.project_type = Some(ProjectType::Go);
            return;
        }

        self.project_type = Some(ProjectType::Generic);
    }

    fn load_existing_artifacts(&mut self) -> Result<()> {
        // Load existing skills
        let skills_dir = self.current_dir.join(".q-skills");
        if skills_dir.exists() {
            self.existing_skills = self.load_json_files(&skills_dir)?;
        }

        // Load existing commands
        let commands_dir = self.current_dir.join(".q-commands");
        if commands_dir.exists() {
            self.existing_commands = self.load_json_files(&commands_dir)?;
        }

        // Load existing agents
        let agents_dir = self.current_dir.join(".amazonq").join("cli-agents");
        if agents_dir.exists() {
            self.existing_agents = self.load_json_files(&agents_dir)?;
        }

        Ok(())
    }

    fn suggest_skill_type(&self) -> Option<SkillType> {
        match self.project_type {
            Some(ProjectType::Python) => Some(SkillType::CodeInline),
            Some(ProjectType::JavaScript) => Some(SkillType::CodeInline),
            Some(ProjectType::Rust) => Some(SkillType::CodeInline),
            Some(ProjectType::Go) => Some(SkillType::CodeInline),
            _ => {
                // Look at existing skills for patterns
                if !self.existing_skills.is_empty() {
                    Some(SkillType::CodeInline) // Most common
                } else {
                    None
                }
            }
        }
    }

    fn suggest_skill_command(&self) -> Option<String> {
        match self.project_type {
            Some(ProjectType::Python) => {
                if self.file_exists("main.py") {
                    Some("python main.py".to_string())
                } else {
                    Some("python script.py".to_string())
                }
            }
            Some(ProjectType::JavaScript) => {
                if self.file_exists("index.js") {
                    Some("node index.js".to_string())
                } else {
                    Some("npm start".to_string())
                }
            }
            Some(ProjectType::Rust) => Some("cargo run".to_string()),
            Some(ProjectType::Go) => Some("go run main.go".to_string()),
            _ => None,
        }
    }

    fn suggest_command_type(&self) -> Option<CommandType> {
        // Analyze existing commands for patterns
        if !self.existing_commands.is_empty() {
            Some(CommandType::Script) // Most common
        } else {
            None
        }
    }

    fn suggest_mcp_servers(&self) -> Vec<String> {
        let mut servers = Vec::new();

        // Suggest based on project type
        match self.project_type {
            Some(ProjectType::Python | ProjectType::JavaScript | ProjectType::Rust | ProjectType::Go) => {
                servers.push("filesystem".to_string());
            }
            _ => {}
        }

        // Look at existing agents for common patterns
        // This would analyze existing agent configs in a real implementation

        servers
    }

    fn generate_skill_description(&self) -> String {
        match self.project_type {
            Some(ProjectType::Python) => "Python script execution".to_string(),
            Some(ProjectType::JavaScript) => "JavaScript/Node.js execution".to_string(),
            Some(ProjectType::Rust) => "Rust application execution".to_string(),
            Some(ProjectType::Go) => "Go application execution".to_string(),
            _ => "Custom skill".to_string(),
        }
    }

    fn generate_command_description(&self) -> String {
        "Custom command".to_string()
    }

    fn generate_agent_description(&self) -> String {
        match self.project_type {
            Some(ProjectType::Python) => "Python development assistant".to_string(),
            Some(ProjectType::JavaScript) => "JavaScript development assistant".to_string(),
            Some(ProjectType::Rust) => "Rust development assistant".to_string(),
            Some(ProjectType::Go) => "Go development assistant".to_string(),
            _ => "Development assistant".to_string(),
        }
    }

    fn file_exists(&self, filename: &str) -> bool {
        self.current_dir.join(filename).exists()
    }

    fn has_files_with_extension(&self, ext: &str) -> bool {
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                if let Some(file_ext) = entry.path().extension() {
                    if file_ext == ext {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn load_json_files(&self, dir: &Path) -> Result<Vec<String>> {
        let mut names = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Some(stem) = entry.path().file_stem() {
                            if let Some(name) = stem.to_str() {
                                names.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(names)
    }

    fn format_type(&self, creation_type: &CreationType) -> &str {
        match creation_type {
            CreationType::CustomCommand => "command",
            CreationType::Skill => "skill",
            CreationType::Agent => "agent",
        }
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,
                        matrix[i + 1][j] + 1
                    ),
                    matrix[i][j] + cost
                );
            }
        }

        matrix[len1][len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_python_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("requirements.txt"), "requests==2.28.0").unwrap();
        fs::write(temp_dir.path().join("main.py"), "print('hello')").unwrap();

        let context = CreationContext::new(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Python));

        let defaults = context.suggest_defaults(&CreationType::Skill);
        assert_eq!(defaults.skill_type, Some(SkillType::CodeInline));
        assert!(defaults.command.as_ref().unwrap().contains("python"));
    }

    #[test]
    fn test_javascript_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("index.js"), "console.log('hello')").unwrap();

        let context = CreationContext::new(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::JavaScript));
    }

    #[test]
    fn test_name_validation() {
        let temp_dir = TempDir::new().unwrap();
        let context = CreationContext::new(temp_dir.path()).unwrap();

        // Valid names
        assert!(context.validate_name("valid-name", &CreationType::Skill).is_valid);
        assert!(context.validate_name("valid_name", &CreationType::Skill).is_valid);
        assert!(context.validate_name("validname123", &CreationType::Skill).is_valid);

        // Invalid names
        let result = context.validate_name("Invalid Name!", &CreationType::Skill);
        assert!(!result.is_valid);
        assert!(result.suggestion.contains("invalid-name"));

        let result = context.validate_name("", &CreationType::Skill);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_existing_artifact_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create skills directory with existing skill
        let skills_dir = temp_dir.path().join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("existing.json"), r#"{"name": "existing"}"#).unwrap();

        let context = CreationContext::new(temp_dir.path()).unwrap();
        assert!(context.existing_skills.contains(&"existing".to_string()));

        // Should detect existing name
        let result = context.validate_name("existing", &CreationType::Skill);
        assert!(!result.is_valid);
        assert!(result.error_message.contains("already exists"));
    }

    #[test]
    fn test_similar_name_suggestions() {
        let temp_dir = TempDir::new().unwrap();
        let mut context = CreationContext::new(temp_dir.path()).unwrap();
        
        context.existing_skills = vec![
            "python-script".to_string(),
            "javascript-runner".to_string(),
            "test-skill".to_string(),
        ];

        let suggestions = context.suggest_similar_names("python");
        assert!(suggestions.contains(&"python-script".to_string()));
        
        let suggestions = context.suggest_similar_names("script");
        assert!(suggestions.contains(&"python-script".to_string()));
    }
}
