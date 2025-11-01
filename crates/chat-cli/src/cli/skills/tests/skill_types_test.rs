use super::super::types::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_code_inline_skill_execution() {
    let skill_json = json!({
        "name": "echo-test",
        "type": "code_inline",
        "command": "echo",
        "args": ["hello world"],
        "timeout": 30
    });
    
    let skill = JsonSkill::from_json(skill_json).unwrap();
    let result = skill.execute(HashMap::new()).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("hello world"));
}

#[test]
fn test_skill_type_parsing() {
    assert_eq!(SkillType::from_str("code_inline").unwrap(), SkillType::CodeInline);
    assert_eq!(SkillType::from_str("code_session").unwrap(), SkillType::CodeSession);
    assert_eq!(SkillType::from_str("conversation").unwrap(), SkillType::Conversation);
    assert_eq!(SkillType::from_str("prompt_inline").unwrap(), SkillType::PromptInline);
}

#[test]
fn test_json_skill_creation() {
    let skill_json = json!({
        "name": "test-skill",
        "type": "code_inline",
        "command": "echo",
        "args": ["test"]
    });
    
    let skill = JsonSkill::from_json(skill_json).unwrap();
    assert_eq!(skill.name, "test-skill");
    assert_eq!(skill.skill_type, SkillType::CodeInline);
}
