//! Comprehensive test suite for unified creation system
//! 
//! Tests cover: CLI parsing, creation flows, terminal UI, context intelligence,
//! backward compatibility, and end-to-end workflows.

mod unit;
mod integration;
mod cli;
mod ux;
mod compatibility;

pub use unit::*;
pub use integration::*;
pub use cli::*;
pub use ux::*;
pub use compatibility::*;

use crate::cli::creation::*;
use std::path::PathBuf;
use tempfile::TempDir;

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
        let base = temp_dir.path();
        
        Self {
            temp_dir,
            commands_dir: base.join(".q-commands"),
            skills_dir: base.join(".q-skills"),
            agents_dir: base.join(".amazonq/cli-agents"),
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
    
    pub fn record_output(&mut self, output: String) {
        self.outputs.push(output);
    }
}
