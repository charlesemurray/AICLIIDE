//! Interactive prompt builder using terminal UI

use eyre::Result;

use super::*;
use crate::cli::creation::{
    SemanticColor,
    TerminalUI,
};

pub struct InteractivePromptBuilder<'a, T: TerminalUI> {
    ui: &'a mut T,
}

impl<'a, T: TerminalUI> InteractivePromptBuilder<'a, T> {
    pub fn new(ui: &'a mut T) -> Self {
        Self { ui }
    }

    /// Interactive template-based creation
    pub fn create_from_template(&mut self) -> Result<PromptTemplate> {
        let template_choice = self.ui.select_option("How would you like to create your assistant?", &[
            ("custom", "Create from scratch - Build your own assistant interactively"),
            (
                "code_reviewer",
                "Code Reviewer - Reviews code for security and best practices",
            ),
            (
                "doc_writer",
                "Documentation Writer - Creates clear technical documentation",
            ),
            ("domain_expert", "Domain Expert - Specialized knowledge assistant"),
            ("conversation", "General Assistant - Flexible helper for various tasks"),
        ])?;

        match template_choice.as_str() {
            "custom" => self.create_custom(),
            _ => self.customize_template(&template_choice),
        }
    }

    /// Step-by-step custom creation
    pub fn create_custom(&mut self) -> Result<PromptTemplate> {
        self.ui
            .show_message("Building a custom AI assistant...", SemanticColor::Info);

        let name = self.ui.prompt_required("Assistant name")?;
        let description = self.ui.prompt_required("Description")?;

        let role_type = self.ui.select_option("What should this assistant specialize in?", &[
            ("code", "Code and software development"),
            ("writing", "Writing and documentation"),
            ("data", "Data analysis and research"),
            ("general", "General problem solving"),
        ])?;

        let role = self.build_role(&role_type)?;
        let capabilities = self.select_capabilities(&role_type)?;
        let constraints = self.select_constraints()?;

        let category = self.map_role_to_category(&role_type);
        let difficulty = self.select_difficulty()?;

        let mut builder = PromptBuilder::new()
            .with_name(name)
            .with_description(description)
            .with_role(role)
            .with_capabilities(capabilities)
            .with_constraints(constraints)
            .with_category(category)
            .with_difficulty(difficulty);

        if self.ui.confirm("Add an example conversation")? {
            let input = self.ui.prompt_required("Example input")?;
            let output = self.ui.prompt_required("Expected output")?;
            builder = builder.with_example(input, output);
        }

        self.preview_and_build(builder)
    }

    fn customize_template(&mut self, template_id: &str) -> Result<PromptTemplate> {
        let base = match template_id {
            "code_reviewer" => self.create_code_reviewer_base(),
            "doc_writer" => self.create_doc_writer_base(),
            "domain_expert" => self.create_domain_expert_base(),
            _ => self.create_conversation_base(),
        };

        let name = self.ui.prompt_optional("Name", Some(&base.0))?.unwrap_or(base.0);

        self.ui.show_message(&format!("Role: {}", base.1), SemanticColor::Debug);
        let use_default_role = self.ui.confirm("Use this role")?;
        let role = if use_default_role {
            base.1
        } else {
            self.ui.prompt_required("Custom role")?
        };

        let builder = PromptBuilder::new()
            .with_name(name)
            .with_description(base.2)
            .with_role(role)
            .with_capabilities(base.3)
            .with_constraints(base.4)
            .with_category(base.5)
            .with_difficulty(base.6);

        self.preview_and_build(builder)
    }

    fn build_role(&mut self, role_type: &str) -> Result<String> {
        let base = match role_type {
            "code" => "You are an expert software engineer",
            "writing" => "You are an experienced technical writer",
            "data" => "You are a data analyst and researcher",
            _ => "You are a helpful assistant",
        };

        let custom = self.ui.prompt_optional("Additional role details", Some(base))?;
        Ok(custom.unwrap_or_else(|| base.to_string()))
    }

    fn select_capabilities(&mut self, role_type: &str) -> Result<Vec<String>> {
        let options = match role_type {
            "code" => vec![
                ("security", "Security vulnerability analysis"),
                ("performance", "Performance optimization"),
                ("architecture", "Architecture and design patterns"),
                ("testing", "Testing and quality assurance"),
            ],
            "writing" => vec![
                ("api_docs", "API documentation"),
                ("tutorials", "Tutorial writing"),
                ("guides", "User guides"),
                ("technical", "Technical specifications"),
            ],
            _ => vec![
                ("analysis", "Problem analysis"),
                ("research", "Research and investigation"),
                ("explanation", "Clear explanations"),
                ("guidance", "Step-by-step guidance"),
            ],
        };

        let selected = self
            .ui
            .select_multiple("Select capabilities (choose multiple):", &options, true)?;

        Ok(selected)
    }

    fn select_constraints(&mut self) -> Result<Vec<String>> {
        let selected = self.ui.select_multiple(
            "Select behavioral constraints:",
            &[
                ("explain", "Always explain reasoning"),
                ("examples", "Provide specific examples"),
                ("concise", "Be concise and direct"),
                ("clarify", "Ask clarifying questions"),
            ],
            true,
        )?;

        Ok(selected)
    }

    fn select_difficulty(&mut self) -> Result<DifficultyLevel> {
        let choice = self.ui.select_option("Difficulty level:", &[
            ("beginner", "Beginner - Simple and approachable"),
            ("intermediate", "Intermediate - Balanced complexity"),
            ("advanced", "Advanced - Expert-level"),
        ])?;

        Ok(match choice.as_str() {
            "beginner" => DifficultyLevel::Beginner,
            "advanced" => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Intermediate,
        })
    }

    fn map_role_to_category(&self, role_type: &str) -> TemplateCategory {
        match role_type {
            "code" => TemplateCategory::CodeReviewer,
            "writing" => TemplateCategory::DocumentationWriter,
            "data" => TemplateCategory::DomainExpert,
            _ => TemplateCategory::ConversationAssistant,
        }
    }

    fn preview_and_build(&mut self, builder: PromptBuilder) -> Result<PromptTemplate> {
        // Show human-readable preview first
        let preview = builder.preview();
        self.ui.show_message("\n📋 Prompt Preview:", SemanticColor::Info);
        self.ui.show_preview(&preview);

        // Validate before building (since build() consumes the builder)
        let validation = builder.validate()?;
        
        // Build the template to show actual file contents
        let template = builder.build()?;
        let json_content = serde_json::to_string_pretty(&template)?;
        
        self.ui.show_message("\n📄 File Contents (JSON):", SemanticColor::Info);
        self.ui.show_preview(&json_content);

        self.ui.show_message(
            &format!("Quality score: {:.1}/1.0", validation.score),
            if validation.score > 0.7 {
                SemanticColor::Success
            } else {
                SemanticColor::Warning
            },
        );

        for issue in &validation.issues {
            let color = match issue.severity {
                IssueSeverity::Error => SemanticColor::Error,
                IssueSeverity::Warning => SemanticColor::Warning,
                IssueSeverity::Info => SemanticColor::Info,
            };
            self.ui.show_message(&issue.message, color);
        }

        if !validation.is_valid {
            return Err(eyre::eyre!("Validation failed"));
        }

        if self.ui.confirm("Create this assistant")? {
            Ok(template)
        } else {
            Err(eyre::eyre!("Creation cancelled"))
        }
    }

    // Template bases (name, role, description, capabilities, constraints, category, difficulty)
    fn create_code_reviewer_base(
        &self,
    ) -> (
        String,
        String,
        String,
        Vec<String>,
        Vec<String>,
        TemplateCategory,
        DifficultyLevel,
    ) {
        (
            "Code Reviewer".to_string(),
            "You are an expert code reviewer with 10+ years of experience".to_string(),
            "Reviews code for security, performance, and best practices".to_string(),
            vec!["security".to_string(), "performance".to_string()],
            vec!["explain".to_string(), "examples".to_string()],
            TemplateCategory::CodeReviewer,
            DifficultyLevel::Advanced,
        )
    }

    fn create_doc_writer_base(
        &self,
    ) -> (
        String,
        String,
        String,
        Vec<String>,
        Vec<String>,
        TemplateCategory,
        DifficultyLevel,
    ) {
        (
            "Documentation Writer".to_string(),
            "You are an experienced technical writer".to_string(),
            "Creates clear, comprehensive technical documentation".to_string(),
            vec!["api_docs".to_string(), "tutorials".to_string()],
            vec!["concise".to_string(), "examples".to_string()],
            TemplateCategory::DocumentationWriter,
            DifficultyLevel::Intermediate,
        )
    }

    fn create_domain_expert_base(
        &self,
    ) -> (
        String,
        String,
        String,
        Vec<String>,
        Vec<String>,
        TemplateCategory,
        DifficultyLevel,
    ) {
        (
            "Domain Expert".to_string(),
            "You are a specialized domain expert".to_string(),
            "Provides expert knowledge in a specific domain".to_string(),
            vec!["analysis".to_string(), "research".to_string()],
            vec!["explain".to_string(), "clarify".to_string()],
            TemplateCategory::DomainExpert,
            DifficultyLevel::Advanced,
        )
    }

    fn create_conversation_base(
        &self,
    ) -> (
        String,
        String,
        String,
        Vec<String>,
        Vec<String>,
        TemplateCategory,
        DifficultyLevel,
    ) {
        (
            "General Assistant".to_string(),
            "You are a helpful and friendly assistant".to_string(),
            "Flexible helper for various tasks".to_string(),
            vec!["guidance".to_string(), "explanation".to_string()],
            vec!["clarify".to_string()],
            TemplateCategory::ConversationAssistant,
            DifficultyLevel::Beginner,
        )
    }
}
