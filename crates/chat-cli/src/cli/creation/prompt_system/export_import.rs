//! Export and import assistants

use std::fs;
use std::path::{
    Path,
    PathBuf,
};

use eyre::Result;

use super::{
    PromptTemplate,
    list_templates,
    load_template,
    save_template,
};

/// Export a single assistant to a file
pub fn export_assistant(id: &str, output_path: &Path) -> Result<PathBuf> {
    let template = load_template(id)?;
    let json = serde_json::to_string_pretty(&template)?;
    fs::write(output_path, json)?;
    Ok(output_path.to_path_buf())
}

/// Export all assistants to a directory
pub fn export_all_assistants(output_dir: &Path) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)?;

    let ids = list_templates()?;
    let mut exported = Vec::new();

    for id in ids {
        let template = load_template(&id)?;
        let filename = format!("{}.json", id);
        let path = output_dir.join(filename);

        let json = serde_json::to_string_pretty(&template)?;
        fs::write(&path, json)?;

        exported.push(path);
    }

    Ok(exported)
}

/// Import an assistant from a file
pub fn import_assistant(input_path: &Path, conflict_strategy: ConflictStrategy) -> Result<String> {
    let json = fs::read_to_string(input_path)?;
    let mut template: PromptTemplate = serde_json::from_str(&json)?;

    // Check if already exists
    if load_template(&template.id).is_ok() {
        match conflict_strategy {
            ConflictStrategy::Skip => {
                return Err(eyre::eyre!("Assistant '{}' already exists (skipped)", template.id));
            },
            ConflictStrategy::Overwrite => {
                // Just save, will overwrite
            },
            ConflictStrategy::Rename => {
                // Find unique name
                let mut counter = 2;
                let original_id = template.id.clone();
                loop {
                    template.id = format!("{}_{}", original_id, counter);
                    if load_template(&template.id).is_err() {
                        break;
                    }
                    counter += 1;
                }
            },
        }
    }

    save_template(&template)?;
    Ok(template.id)
}

/// Import all assistants from a directory
pub fn import_all_assistants(input_dir: &Path, conflict_strategy: ConflictStrategy) -> Result<Vec<String>> {
    let mut imported = Vec::new();

    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match import_assistant(&path, conflict_strategy) {
                Ok(id) => imported.push(id),
                Err(e) => eprintln!("Warning: Failed to import {}: {}", path.display(), e),
            }
        }
    }

    Ok(imported)
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Skip,
    Overwrite,
    Rename,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::prompt_system::PromptBuilder;
    use tempfile::TempDir;

    #[test]
    fn test_export_import_roundtrip() -> Result<()> {
        let temp = TempDir::new()?;
        let export_path = temp.path().join("test.json");
        
        let template = PromptBuilder::new()
            .with_name("TestExportImport".to_string())
            .with_role("Role".to_string())
            .build()?;
        
        save_template(&template)?;
        
        // Export
        export_assistant(&template.id, &export_path)?;
        assert!(export_path.exists());
        
        // Delete original
        super::super::delete_template(&template.id)?;
        
        // Import
        let imported_id = import_assistant(&export_path, ConflictStrategy::Overwrite)?;
        assert_eq!(imported_id, template.id);
        
        // Cleanup
        super::super::delete_template(&imported_id)?;
        
        Ok(())
    }

    #[test]
    fn test_export_all() -> Result<()> {
        let temp = TempDir::new()?;
        let export_dir = temp.path().join("exports");
        
        let t1 = PromptBuilder::new()
            .with_name("TestExportAll1".to_string())
            .with_role("Role".to_string())
            .build()?;
        
        let t2 = PromptBuilder::new()
            .with_name("TestExportAll2".to_string())
            .with_role("Role".to_string())
            .build()?;
        
        save_template(&t1)?;
        save_template(&t2)?;
        
        let exported = export_all_assistants(&export_dir)?;
        assert!(exported.len() >= 2);
        
        // Cleanup
        super::super::delete_template(&t1.id)?;
        super::super::delete_template(&t2.id)?;
        
        Ok(())
    }

    #[test]
    fn test_import_with_rename() -> Result<()> {
        let temp = TempDir::new()?;
        let export_path = temp.path().join("test.json");
        
        let template = PromptBuilder::new()
            .with_name("TestImportRename".to_string())
            .with_role("Role".to_string())
            .build()?;
        
        save_template(&template)?;
        export_assistant(&template.id, &export_path)?;
        
        // Import again with rename strategy
        let new_id = import_assistant(&export_path, ConflictStrategy::Rename)?;
        assert_ne!(new_id, template.id);
        assert!(new_id.starts_with(&template.id));
        
        // Cleanup
        super::super::delete_template(&template.id)?;
        super::super::delete_template(&new_id)?;
        
        Ok(())
    }
}
