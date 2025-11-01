use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkillType {
    #[serde(rename = "code_inline")]
    CodeInline {
        command: String,
        args: Option<Vec<String>>,
        working_dir: Option<String>,
    },
    #[serde(rename = "code_session")]
    CodeSession {
        command: String,
        args: Option<Vec<String>>,
        working_dir: Option<String>,
        session_timeout: Option<u64>,
    },
    #[serde(rename = "conversation")]
    Conversation {
        prompt_template: String,
        context_files: Option<Vec<String>>,
        model: Option<String>,
    },
    #[serde(rename = "prompt_inline")]
    PromptInline {
        prompt: String,
        parameters: Option<Vec<SkillParameter>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub aliases: Option<Vec<String>>,
    pub scope: Option<SkillScope>,
    #[serde(flatten)]
    pub skill_type: SkillType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillScope {
    Workspace,
    Global,
}

impl Default for SkillScope {
    fn default() -> Self {
        Self::Workspace
    }
}
