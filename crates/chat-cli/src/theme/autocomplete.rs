use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Autocomplete configuration for commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutocompleteConfig {
    pub subcommands: HashMap<String, SubcommandConfig>,
    pub dynamic_sources: Option<HashMap<String, DynamicSource>>,
}

/// Configuration for a subcommand's autocomplete
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubcommandConfig {
    pub description: String,
    pub options: OptionConfig,
}

/// Configuration for command options/parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OptionConfig {
    /// Simple list of option flags
    Simple(Vec<String>),
    /// Complex options with types and suggestions
    Complex(HashMap<String, OptionType>),
}

/// Types of command options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OptionType {
    #[serde(rename = "flag")]
    Flag {
        description: String,
    },
    #[serde(rename = "string")]
    String {
        suggestions: Option<Vec<String>>,
        description: Option<String>,
    },
    #[serde(rename = "dynamic")]
    Dynamic {
        source: String,
        description: String,
    },
}

/// Dynamic completion source configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicSource {
    pub command: String,
    pub cache_duration: u64, // seconds
    pub parser: String,      // "lines", "json", "csv"
}

/// Autocomplete suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionSuggestion {
    pub text: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Autocomplete engine for commands
#[derive(Debug)]
pub struct AutocompleteEngine {
    configs: HashMap<String, AutocompleteConfig>,
    cache: HashMap<String, (Vec<String>, std::time::Instant)>,
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    /// Register autocomplete configuration for a command
    pub fn register_command(&mut self, command: String, config: AutocompleteConfig) {
        self.configs.insert(command, config);
    }

    /// Get completions for a command input
    pub fn get_completions(&mut self, input: &str) -> Vec<CompletionSuggestion> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return self.get_root_completions();
        }

        let command = parts[0].trim_start_matches('/');
        
        if let Some(config) = self.configs.get(command) {
            if parts.len() == 1 {
                // Complete subcommands
                self.get_subcommand_completions(config)
            } else if parts.len() == 2 {
                // Complete options for subcommand
                let subcommand = parts[1];
                self.get_option_completions(config, subcommand)
            } else {
                // Complete option values
                self.get_option_value_completions(config, &parts)
            }
        } else {
            // Command not found, return root completions
            self.get_root_completions()
        }
    }

    fn get_root_completions(&self) -> Vec<CompletionSuggestion> {
        self.configs
            .keys()
            .map(|cmd| CompletionSuggestion {
                text: format!("/{}", cmd),
                description: None,
                category: None,
            })
            .collect()
    }

    fn get_subcommand_completions(&self, config: &AutocompleteConfig) -> Vec<CompletionSuggestion> {
        config
            .subcommands
            .iter()
            .map(|(name, sub_config)| CompletionSuggestion {
                text: name.clone(),
                description: Some(sub_config.description.clone()),
                category: None,
            })
            .collect()
    }

    fn get_option_completions(&self, config: &AutocompleteConfig, subcommand: &str) -> Vec<CompletionSuggestion> {
        if let Some(sub_config) = config.subcommands.get(subcommand) {
            match &sub_config.options {
                OptionConfig::Simple(options) => options
                    .iter()
                    .map(|opt| CompletionSuggestion {
                        text: opt.clone(),
                        description: None,
                        category: None,
                    })
                    .collect(),
                OptionConfig::Complex(options) => options
                    .iter()
                    .map(|(name, opt_type)| {
                        let description = match opt_type {
                            OptionType::Flag { description } => Some(description.clone()),
                            OptionType::String { description, .. } => description.clone(),
                            OptionType::Dynamic { description, .. } => Some(description.clone()),
                        };
                        CompletionSuggestion {
                            text: name.clone(),
                            description,
                            category: None,
                        }
                    })
                    .collect(),
            }
        } else {
            Vec::new()
        }
    }

    fn get_option_value_completions(&mut self, config: &AutocompleteConfig, parts: &[&str]) -> Vec<CompletionSuggestion> {
        if parts.len() < 3 {
            return Vec::new();
        }

        let subcommand = parts[1];
        let option = parts[2];

        if let Some(sub_config) = config.subcommands.get(subcommand) {
            if let OptionConfig::Complex(options) = &sub_config.options {
                if let Some(opt_type) = options.get(option) {
                    match opt_type {
                        OptionType::String { suggestions, .. } => {
                            suggestions.as_ref().map_or(Vec::new(), |suggs| {
                                suggs
                                    .iter()
                                    .map(|s| CompletionSuggestion {
                                        text: s.clone(),
                                        description: None,
                                        category: None,
                                    })
                                    .collect()
                            })
                        },
                        OptionType::Dynamic { source, .. } => {
                            self.get_dynamic_completions(config, source)
                        },
                        OptionType::Flag { .. } => Vec::new(),
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    fn get_dynamic_completions(&mut self, config: &AutocompleteConfig, source_name: &str) -> Vec<CompletionSuggestion> {
        if let Some(dynamic_sources) = &config.dynamic_sources {
            if let Some(source) = dynamic_sources.get(source_name) {
                // Check cache first
                let cache_key = format!("{}:{}", source_name, source.command);
                let now = std::time::Instant::now();
                
                if let Some((cached_results, timestamp)) = self.cache.get(&cache_key) {
                    if now.duration_since(*timestamp).as_secs() < source.cache_duration {
                        return cached_results
                            .iter()
                            .map(|s| CompletionSuggestion {
                                text: s.clone(),
                                description: None,
                                category: None,
                            })
                            .collect();
                    }
                }

                // Execute command and cache results
                if let Ok(output) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&source.command)
                    .output()
                {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let results: Vec<String> = match source.parser.as_str() {
                        "lines" => stdout.lines().map(|s| s.trim().to_string()).collect(),
                        _ => vec![stdout.trim().to_string()],
                    };

                    // Cache the results
                    self.cache.insert(cache_key, (results.clone(), now));

                    return results
                        .into_iter()
                        .map(|s| CompletionSuggestion {
                            text: s,
                            description: None,
                            category: None,
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = std::time::Instant::now();
        self.cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < 3600 // Keep for 1 hour max
        });
    }
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AutocompleteConfig {
        let mut subcommands = HashMap::new();
        let mut options = HashMap::new();
        
        options.insert("--message".to_string(), OptionType::String {
            suggestions: Some(vec!["Initial commit".to_string(), "Fix bug".to_string()]),
            description: Some("Commit message".to_string()),
        });
        options.insert("--all".to_string(), OptionType::Flag {
            description: "Stage all changes".to_string(),
        });

        subcommands.insert("commit".to_string(), SubcommandConfig {
            description: "Commit changes".to_string(),
            options: OptionConfig::Complex(options),
        });

        subcommands.insert("status".to_string(), SubcommandConfig {
            description: "Show status".to_string(),
            options: OptionConfig::Simple(vec!["--short".to_string(), "--branch".to_string()]),
        });

        let mut dynamic_sources = HashMap::new();
        dynamic_sources.insert("git_remotes".to_string(), DynamicSource {
            command: "echo 'origin\nupstream'".to_string(),
            cache_duration: 300,
            parser: "lines".to_string(),
        });

        AutocompleteConfig {
            subcommands,
            dynamic_sources: Some(dynamic_sources),
        }
    }

    #[test]
    fn test_autocomplete_config_creation() {
        let config = create_test_config();
        assert_eq!(config.subcommands.len(), 2);
        assert!(config.subcommands.contains_key("commit"));
        assert!(config.subcommands.contains_key("status"));
    }

    #[test]
    fn test_option_config_simple() {
        let simple = OptionConfig::Simple(vec!["--flag1".to_string(), "--flag2".to_string()]);
        match simple {
            OptionConfig::Simple(flags) => {
                assert_eq!(flags.len(), 2);
                assert_eq!(flags[0], "--flag1");
            },
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_option_config_complex() {
        let mut options = HashMap::new();
        options.insert("--message".to_string(), OptionType::String {
            suggestions: Some(vec!["test".to_string()]),
            description: Some("Message".to_string()),
        });

        let complex = OptionConfig::Complex(options);
        match complex {
            OptionConfig::Complex(opts) => {
                assert_eq!(opts.len(), 1);
                assert!(opts.contains_key("--message"));
            },
            _ => panic!("Expected Complex variant"),
        }
    }

    #[test]
    fn test_option_types() {
        let flag = OptionType::Flag {
            description: "Test flag".to_string(),
        };
        assert!(matches!(flag, OptionType::Flag { .. }));

        let string_opt = OptionType::String {
            suggestions: Some(vec!["test".to_string()]),
            description: Some("Test string".to_string()),
        };
        assert!(matches!(string_opt, OptionType::String { .. }));

        let dynamic = OptionType::Dynamic {
            source: "test_source".to_string(),
            description: "Test dynamic".to_string(),
        };
        assert!(matches!(dynamic, OptionType::Dynamic { .. }));
    }

    #[test]
    fn test_completion_suggestion() {
        let suggestion = CompletionSuggestion {
            text: "test".to_string(),
            description: Some("Test description".to_string()),
            category: Some("test_category".to_string()),
        };

        assert_eq!(suggestion.text, "test");
        assert_eq!(suggestion.description, Some("Test description".to_string()));
        assert_eq!(suggestion.category, Some("test_category".to_string()));
    }

    #[test]
    fn test_autocomplete_engine_creation() {
        let engine = AutocompleteEngine::new();
        assert_eq!(engine.configs.len(), 0);
        assert_eq!(engine.cache.len(), 0);
    }

    #[test]
    fn test_register_command() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        assert_eq!(engine.configs.len(), 1);
        assert!(engine.configs.contains_key("git"));
    }

    #[test]
    fn test_get_root_completions() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].text, "/git");
    }

    #[test]
    fn test_get_subcommand_completions() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("/git");
        assert_eq!(completions.len(), 2);
        
        let commit_completion = completions.iter().find(|c| c.text == "commit").unwrap();
        assert_eq!(commit_completion.description, Some("Commit changes".to_string()));
        
        let status_completion = completions.iter().find(|c| c.text == "status").unwrap();
        assert_eq!(status_completion.description, Some("Show status".to_string()));
    }

    #[test]
    fn test_get_option_completions_simple() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("/git status");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.text == "--short"));
        assert!(completions.iter().any(|c| c.text == "--branch"));
    }

    #[test]
    fn test_get_option_completions_complex() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("/git commit");
        assert_eq!(completions.len(), 2);
        
        let message_completion = completions.iter().find(|c| c.text == "--message").unwrap();
        assert!(message_completion.description.is_some());
        
        let all_completion = completions.iter().find(|c| c.text == "--all").unwrap();
        assert_eq!(all_completion.description, Some("Stage all changes".to_string()));
    }

    #[test]
    fn test_get_option_value_completions() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("/git commit --message");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.text == "Initial commit"));
        assert!(completions.iter().any(|c| c.text == "Fix bug"));
    }

    #[test]
    fn test_cleanup_cache() {
        let mut engine = AutocompleteEngine::new();
        
        // Add some cache entries
        let now = std::time::Instant::now();
        engine.cache.insert("test1".to_string(), (vec!["value1".to_string()], now));
        engine.cache.insert("test2".to_string(), (vec!["value2".to_string()], now));
        
        assert_eq!(engine.cache.len(), 2);
        
        engine.cleanup_cache();
        // Should still have entries since they're recent
        assert_eq!(engine.cache.len(), 2);
    }

    #[test]
    fn test_nonexistent_command() {
        let mut engine = AutocompleteEngine::new();
        let completions = engine.get_completions("/nonexistent");
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_empty_subcommand() {
        let mut engine = AutocompleteEngine::new();
        let config = create_test_config();
        
        engine.register_command("git".to_string(), config);
        
        let completions = engine.get_completions("/git nonexistent");
        assert_eq!(completions.len(), 0);
    }
}
