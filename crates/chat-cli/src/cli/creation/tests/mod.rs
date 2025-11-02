//! Comprehensive test suite for unified creation system
//!
//! Tests cover: CLI parsing, creation flows, terminal UI, context intelligence,
//! and end-to-end workflows.

mod cli;
mod integration;
mod unit;
mod ux;

use std::path::PathBuf;

pub use cli::*;
use eyre::Result;
pub use integration::*;
use tempfile::TempDir;
pub use unit::*;
pub use ux::*;

use crate::cli::creation::types::{
    SemanticColor,
    TerminalUI,
};
use crate::cli::creation::*;

/// Test utilities and fixtures
pub struct TestFixtures {
    pub temp_dir: TempDir,
    pub commands_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub agents_dir: PathBuf,
}

impl TestFixtures {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        Self {
            commands_dir: base.join(".q-commands"),
            skills_dir: base.join(".q-skills"),
            agents_dir: base.join(".amazonq/cli-agents"),
            temp_dir,
        }
    }

    pub fn setup_directories(&self) {
        std::fs::create_dir_all(&self.commands_dir).unwrap();
        std::fs::create_dir_all(&self.skills_dir).unwrap();
        std::fs::create_dir_all(&self.agents_dir).unwrap();
    }
}

/// Mock terminal UI for testing
pub struct MockTerminalUI {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    input_index: usize,
}

impl MockTerminalUI {
    pub fn new(inputs: Vec<String>) -> Self {
        Self {
            inputs,
            outputs: Vec::new(),
            input_index: 0,
        }
    }

    pub fn next_input(&mut self) -> String {
        if self.input_index < self.inputs.len() {
            let input = self.inputs[self.input_index].clone();
            self.input_index += 1;
            input
        } else {
            String::new()
        }
    }

    pub fn show_success(&mut self, message: &str) {
        self.outputs.push(format!("\x1b[32mSUCCESS: {}\x1b[0m", message));
    }

    pub fn show_error(&mut self, message: &str) {
        self.outputs.push(format!("\x1b[31mERROR: {}\x1b[0m", message));
    }

    pub fn show_info(&mut self, message: &str) {
        self.outputs.push(format!("\x1b[34mINFO: {}\x1b[0m", message));
    }

    pub fn show_warning(&mut self, message: &str) {
        self.outputs.push(format!("\x1b[33mWARNING: {}\x1b[0m", message));
    }

    pub fn show_contextual_help(&mut self, context: &str) -> String {
        let help = match context {
            "skill_type" => "code_inline: Run shell commands\nconversation: Chat assistant".to_string(),
            _ => format!("Help for {}", context),
        };
        self.outputs.push(help.clone());
        help
    }

    pub fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        let percentage = (current * 100) / total;
        let filled = (current * 8) / total;
        let empty = 8 - filled;
        let bar = "█".repeat(filled) + &"░".repeat(empty);

        self.outputs
            .push(format!("{} {}% {}/{} {}", bar, percentage, current, total, message));
    }

    pub fn prompt_with_validation<F>(&mut self, prompt: &str, validator: F) -> Result<String>
    where
        F: Fn(&str) -> Result<(), String>,
    {
        loop {
            let input = self.next_input();
            self.outputs.push(format!("PROMPT: {} -> {}", prompt, input));

            match validator(&input) {
                Ok(()) => return Ok(input),
                Err(error_msg) => {
                    self.outputs.push(error_msg);
                    // Continue loop to get next input
                },
            }
        }
    }
}

impl TerminalUI for MockTerminalUI {
    fn prompt_required(&mut self, field: &str) -> Result<String> {
        let input = self.next_input();
        self.outputs.push(format!("{}: ", field));
        Ok(input)
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        let input = self.next_input();
        if input.is_empty() {
            Ok(default.map(|s| s.to_string()))
        } else {
            self.outputs.push(format!("{}: ", field));
            Ok(Some(input))
        }
    }

    fn confirm(&mut self, message: &str) -> Result<bool> {
        let input = self.next_input();
        let result = input.to_lowercase().starts_with('y');
        self.outputs.push(format!("{}? ", message));
        Ok(result)
    }

    fn show_preview(&mut self, content: &str) {
        self.outputs.push(format!("PREVIEW: {}", content));
    }

    fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        self.outputs
            .push(format!("PROGRESS: {}/{} - {}", current, total, message));
    }

    fn show_message(&mut self, message: &str, _color: SemanticColor) {
        self.outputs.push(format!("MESSAGE: {}", message));
    }

    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String> {
        let input = self.next_input();
        self.outputs.push(format!("SELECT: {} -> {}", prompt, input));
        // Return first option if input is invalid
        Ok(options.get(0).map(|(key, _)| key.to_string()).unwrap_or(input))
    }

    fn select_multiple(&mut self, prompt: &str, options: &[(&str, &str)], _allow_other: bool) -> Result<Vec<String>> {
        let input = self.next_input();
        self.outputs.push(format!("SELECT_MULTIPLE: {} -> {}", prompt, input));
        // Return first option as a vec if input is invalid
        Ok(vec![options.get(0).map(|(key, _)| key.to_string()).unwrap_or(input)])
    }
}

impl MockTerminalUI {
    // Helper method for tests (not part of trait)
    pub fn record_output(&mut self, output: String) {
        self.outputs.push(output);
    }
}
