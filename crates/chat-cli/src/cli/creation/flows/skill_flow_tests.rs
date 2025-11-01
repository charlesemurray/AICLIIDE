//! Tests for skill creation flow

use super::*;
use crate::cli::creation::types::CreationMode;

#[test]
fn test_skill_creation_flow_new() {
    let flow = SkillCreationFlow::new("test-skill".to_string(), CreationMode::Guided);
    assert!(flow.is_ok());
    
    let flow = flow.unwrap();
    assert_eq!(flow.name, "test-skill");
    assert_eq!(flow.mode, CreationMode::Guided);
}

#[test]
fn test_skill_creation_flow_different_modes() {
    let modes = vec![
        CreationMode::Quick,
        CreationMode::Guided,
        CreationMode::Expert,
        CreationMode::Template,
        CreationMode::Preview,
    ];
    
    for mode in modes {
        let flow = SkillCreationFlow::new("test".to_string(), mode.clone());
        assert!(flow.is_ok());
        assert_eq!(flow.unwrap().mode, mode);
    }
}
