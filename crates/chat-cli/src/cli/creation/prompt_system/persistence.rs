//! Persistence layer for saving and loading prompt templates

use std::fs;
use std::path::{
    Path,
    PathBuf,
};

use eyre::Result;

use super::PromptTemplate;

/// Get the assistants directory path
pub fn get_assistants_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre::eyre!("Could not find home directory"))?;
    Ok(home.join(".q-skills"))
}

/// Save a template to disk
pub fn save_template(template: &PromptTemplate) -> Result<PathBuf> {
    let dir = get_assistants_dir()?;
    fs::create_dir_all(&dir)?;

    let filename = format!("{}.json", template.id);
    let path = dir.join(filename);

    let json = serde_json::to_string_pretty(template)?;
    fs::write(&path, json)?;

    Ok(path)
}

/// Load a template from disk
pub fn load_template(id: &str) -> Result<PromptTemplate> {
    let dir = get_assistants_dir()?;
    let path = dir.join(format!("{}.json", id));

    let json = fs::read_to_string(&path)?;
    let template = serde_json::from_str(&json)?;

    Ok(template)
}

/// List all saved templates
pub fn list_templates() -> Result<Vec<String>> {
    let dir = get_assistants_dir()?;

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                templates.push(stem.to_string());
            }
        }
    }

    templates.sort();
    Ok(templates)
}

/// Delete a template
pub fn delete_template(id: &str) -> Result<()> {
    let dir = get_assistants_dir()?;
    let path = dir.join(format!("{}.json", id));

    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::cli::creation::prompt_system::*;

    fn with_temp_dir<F>(f: F) -> Result<()>
    where
        F: FnOnce(&Path) -> Result<()>,
    {
        let temp = TempDir::new()?;
        f(temp.path())
    }

    #[test]
    fn test_save_and_load_template() -> Result<()> {
        with_temp_dir(|_dir| {
            let template = PromptBuilder::new()
                .with_name("Test".to_string())
                .with_description("Test assistant".to_string())
                .with_role("You are a test".to_string())
                .add_capability("testing".to_string())
                .build()?;

            let path = save_template(&template)?;
            assert!(path.exists());

            let loaded = load_template(&template.id)?;
            assert_eq!(loaded.name, template.name);
            assert_eq!(loaded.id, template.id);

            Ok(())
        })
    }

    #[test]
    fn test_list_templates() -> Result<()> {
        with_temp_dir(|_dir| {
            let template1 = PromptBuilder::new()
                .with_name("Test1".to_string())
                .with_role("Role".to_string())
                .build()?;

            let template2 = PromptBuilder::new()
                .with_name("Test2".to_string())
                .with_role("Role".to_string())
                .build()?;

            save_template(&template1)?;
            save_template(&template2)?;

            let list = list_templates()?;
            assert!(list.contains(&template1.id));
            assert!(list.contains(&template2.id));

            Ok(())
        })
    }

    #[test]
    fn test_delete_template() -> Result<()> {
        with_temp_dir(|_dir| {
            let template = PromptBuilder::new()
                .with_name("Test".to_string())
                .with_role("Role".to_string())
                .build()?;

            save_template(&template)?;
            assert!(load_template(&template.id).is_ok());

            delete_template(&template.id)?;
            assert!(load_template(&template.id).is_err());

            Ok(())
        })
    }
}
