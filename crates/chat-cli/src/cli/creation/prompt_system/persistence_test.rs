#[cfg(test)]
mod persistence_tests {
    use eyre::Result;

    use crate::cli::creation::prompt_system::*;

    #[test]
    fn test_save_load_roundtrip() -> Result<()> {
        let template = PromptBuilder::new()
            .with_name("Test Assistant".to_string())
            .with_description("Test".to_string())
            .with_role("You are a test".to_string())
            .add_capability("testing".to_string())
            .build()?;

        let id = template.id.clone();

        // Save
        let path = save_template(&template)?;
        assert!(path.exists());

        // Load
        let loaded = load_template(&id)?;
        assert_eq!(loaded.name, template.name);
        assert_eq!(loaded.id, template.id);
        assert_eq!(loaded.role, template.role);

        // Cleanup
        delete_template(&id)?;

        Ok(())
    }

    #[test]
    fn test_list_saved_templates() -> Result<()> {
        let template1 = PromptBuilder::new()
            .with_name("Test1".to_string())
            .with_role("Role1".to_string())
            .build()?;

        let template2 = PromptBuilder::new()
            .with_name("Test2".to_string())
            .with_role("Role2".to_string())
            .build()?;

        save_template(&template1)?;
        save_template(&template2)?;

        let list = list_templates()?;
        assert!(list.len() >= 2);

        // Cleanup
        delete_template(&template1.id)?;
        delete_template(&template2.id)?;

        Ok(())
    }

    #[test]
    fn test_delete_removes_template() -> Result<()> {
        let template = PromptBuilder::new()
            .with_name("ToDelete".to_string())
            .with_role("Role".to_string())
            .build()?;

        save_template(&template)?;
        assert!(load_template(&template.id).is_ok());

        delete_template(&template.id)?;
        assert!(load_template(&template.id).is_err());

        Ok(())
    }
}
