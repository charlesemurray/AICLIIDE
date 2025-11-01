//! Creation assistant orchestrator that manages the creation workflow

use crate::cli::creation::{
    CreationFlow, TerminalUI, TerminalUIImpl, CreationType, CreationPhase, PhaseResult, SemanticColor,
    CreationConfig, CreationArtifact
};
use eyre::Result;
use std::process::ExitCode;

/// Main creation assistant that orchestrates the creation workflow
pub struct CreationAssistant<F: CreationFlow> {
    flow: F,
    ui: Box<dyn TerminalUI>,
    current_phase: usize,
}

impl<F: CreationFlow> CreationAssistant<F> {
    pub fn new(flow: F) -> Self {
        Self {
            flow,
            ui: Box::new(TerminalUIImpl::new()),
            current_phase: 0,
        }
    }

    #[cfg(test)]
    pub fn new_with_ui(flow: F, ui: Box<dyn TerminalUI>) -> Self {
        Self {
            flow,
            ui,
            current_phase: 0,
        }
    }

    pub async fn run(mut self) -> Result<ExitCode> {
        let creation_type = self.flow.creation_type();
        let phases = creation_type.required_phases();
        
        self.ui.show_message(
            &format!("Creating {} '{}'", 
                self.format_creation_type(&creation_type),
                self.flow.get_config().get_name()
            ),
            SemanticColor::Info
        );

        // Execute each phase
        for (index, phase) in phases.iter().enumerate() {
            self.current_phase = index;
            self.ui.show_progress(index + 1, phases.len(), &self.format_phase(phase));

            loop {
                match self.flow.execute_phase(phase.clone())? {
                    PhaseResult::Continue => break,
                    PhaseResult::Complete => {
                        return self.complete_creation().await;
                    }
                    PhaseResult::Retry(error_msg) => {
                        self.ui.show_message(&error_msg, SemanticColor::Error);
                    }
                }
            }
        }

        self.complete_creation().await
    }

    async fn complete_creation(&mut self) -> Result<ExitCode> {
        // Show preview
        let config = self.flow.get_config();
        let preview = self.generate_preview(config);
        self.ui.show_preview(&preview);

        // Confirm creation
        if !self.ui.confirm("Create")? {
            self.ui.show_message("Creation cancelled", SemanticColor::Warning);
            return Ok(ExitCode::SUCCESS);
        }

        // Create and persist artifact
        let artifact = self.flow.create_artifact()?;
        let location = self.get_storage_location(&self.flow.creation_type());
        artifact.persist(&location)?;

        self.ui.show_message(
            &format!("Created {} '{}' successfully", 
                self.format_creation_type(&self.flow.creation_type()),
                artifact.get_name()
            ),
            SemanticColor::Success
        );

        Ok(ExitCode::SUCCESS)
    }

    fn generate_preview(&self, config: &F::Config) -> String {
        let creation_type = self.flow.creation_type();
        let type_name = self.format_creation_type(&creation_type);
        
        format!(
            "Creating: {} '{}'\nType: {}\nLocation: {}",
            type_name,
            config.get_name(),
            self.format_creation_details(config),
            self.get_storage_location(&creation_type).display()
        )
    }

    fn format_creation_type(&self, creation_type: &CreationType) -> &str {
        match creation_type {
            CreationType::CustomCommand => "command",
            CreationType::Skill => "skill",
            CreationType::Agent => "agent",
        }
    }

    fn format_phase(&self, phase: &CreationPhase) -> String {
        match phase {
            CreationPhase::Discovery => "Discovering requirements".to_string(),
            CreationPhase::Planning => "Planning creation process".to_string(),
            CreationPhase::BasicConfig => "Basic configuration".to_string(),
            CreationPhase::AdvancedConfig => "Advanced configuration".to_string(),
            CreationPhase::Security => "Security settings".to_string(),
            CreationPhase::Testing => "Testing and validation".to_string(),
            CreationPhase::Completion => "Finalizing".to_string(),
        }
    }

    fn format_creation_details(&self, _config: &F::Config) -> String {
        match self.flow.creation_type() {
            CreationType::CustomCommand => "script command".to_string(),
            CreationType::Skill => "executable skill".to_string(), 
            CreationType::Agent => "AI agent".to_string(),
        }
    }

    fn get_storage_location(&self, creation_type: &CreationType) -> std::path::PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        
        match creation_type {
            CreationType::CustomCommand => current_dir.join(".q-commands"),
            CreationType::Skill => current_dir.join(".q-skills"),
            CreationType::Agent => current_dir.join(".amazonq").join("cli-agents"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::{MockTerminalUI, CreationConfig, CreationArtifact};
    use std::path::Path;

    // Mock implementations for testing
    struct MockConfig {
        name: String,
        complete: bool,
    }

    impl CreationConfig for MockConfig {
        fn validate(&self) -> Result<()> { Ok(()) }
        fn apply_defaults(&mut self) {}
        fn is_complete(&self) -> bool { self.complete }
        fn get_name(&self) -> &str { &self.name }
    }

    struct MockArtifact {
        name: String,
    }

    impl CreationArtifact for MockArtifact {
        fn persist(&self, _location: &Path) -> Result<()> { Ok(()) }
        fn validate_before_save(&self) -> Result<()> { Ok(()) }
        fn get_name(&self) -> &str { &self.name }
    }

    struct MockFlow {
        config: MockConfig,
        phase_count: usize,
    }

    impl CreationFlow for MockFlow {
        type Config = MockConfig;
        type Artifact = MockArtifact;

        fn creation_type(&self) -> CreationType {
            CreationType::CustomCommand
        }

        fn execute_phase(&mut self, _phase: CreationPhase) -> Result<PhaseResult> {
            self.phase_count += 1;
            if self.phase_count >= 3 {
                Ok(PhaseResult::Complete)
            } else {
                Ok(PhaseResult::Continue)
            }
        }

        fn create_artifact(&self) -> Result<Self::Artifact> {
            Ok(MockArtifact {
                name: self.config.name.clone(),
            })
        }

        fn get_config(&self) -> &Self::Config {
            &self.config
        }
    }

    #[tokio::test]
    async fn test_creation_assistant_workflow() {
        let flow = MockFlow {
            config: MockConfig {
                name: "test".to_string(),
                complete: true,
            },
            phase_count: 0,
        };

        let ui = Box::new(MockTerminalUI::new(vec!["y".to_string()])); // Confirm creation
        let assistant = CreationAssistant::new_with_ui(flow, ui);

        let result = assistant.run().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_creation_assistant_cancellation() {
        let flow = MockFlow {
            config: MockConfig {
                name: "test".to_string(),
                complete: true,
            },
            phase_count: 0,
        };

        let ui = Box::new(MockTerminalUI::new(vec!["n".to_string()])); // Cancel creation
        let assistant = CreationAssistant::new_with_ui(flow, ui);

        let result = assistant.run().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[test]
    fn test_format_creation_type() {
        let flow = MockFlow {
            config: MockConfig {
                name: "test".to_string(),
                complete: true,
            },
            phase_count: 0,
        };

        let assistant = CreationAssistant::new(flow);
        assert_eq!(assistant.format_creation_type(&CreationType::CustomCommand), "command");
        assert_eq!(assistant.format_creation_type(&CreationType::Skill), "skill");
        assert_eq!(assistant.format_creation_type(&CreationType::Agent), "agent");
    }

    #[test]
    fn test_get_storage_location() {
        let flow = MockFlow {
            config: MockConfig {
                name: "test".to_string(),
                complete: true,
            },
            phase_count: 0,
        };

        let assistant = CreationAssistant::new(flow);
        
        let cmd_location = assistant.get_storage_location(&CreationType::CustomCommand);
        assert!(cmd_location.to_string_lossy().contains(".q-commands"));
        
        let skill_location = assistant.get_storage_location(&CreationType::Skill);
        assert!(skill_location.to_string_lossy().contains(".q-skills"));
        
        let agent_location = assistant.get_storage_location(&CreationType::Agent);
        assert!(agent_location.to_string_lossy().contains(".amazonq"));
    }
}
