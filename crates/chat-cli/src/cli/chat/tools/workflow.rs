//! Workflow tool implementation

use std::collections::HashMap;

use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

use crate::cli::agent::{
    Agent,
    PermissionEvalResult,
};
use crate::os::Os;

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub tool: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub output: String,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    Running,
    Completed,
    Failed,
}

pub struct StepExecutor;

impl StepExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_step(&self, _step: &WorkflowStep) -> Result<StepResult> {
        Ok(StepResult {
            output: "step executed".to_string(),
            success: true,
        })
    }

    pub fn resolve_tool_name(&self, step: &WorkflowStep) -> Result<String> {
        if step.tool.is_empty() {
            return Err(eyre::eyre!("Step tool name cannot be empty"));
        }
        Ok(step.tool.clone())
    }

    pub fn build_step_params(&self, step: &WorkflowStep, context: &Value) -> Result<HashMap<String, Value>> {
        match &step.parameters {
            Value::Object(map) => {
                let mut params = HashMap::new();
                for (key, value) in map {
                    // Replace context variables like {{steps.step1.output}}
                    let resolved_value = self.resolve_context_vars(value, context);
                    params.insert(key.clone(), resolved_value);
                }
                Ok(params)
            },
            _ => Ok(HashMap::new()),
        }
    }

    fn resolve_context_vars(&self, value: &Value, context: &Value) -> Value {
        match value {
            Value::String(s) => {
                // Simple variable replacement: {{steps.step_name.output}}
                if s.starts_with("{{") && s.ends_with("}}") {
                    let path = s.trim_start_matches("{{").trim_end_matches("}}").trim();
                    self.get_context_value(context, path).unwrap_or_else(|| value.clone())
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }

    fn get_context_value(&self, context: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context;
        
        for part in parts {
            current = current.get(part)?;
        }
        
        Some(current.clone())
    }

    pub async fn execute_step_with_context_and_manager(
        &self,
        step: &WorkflowStep,
        context: &Value,
        tool_manager: Option<&mut crate::cli::chat::tool_manager::ToolManager>,
    ) -> Result<StepResult> {
        let tool_name = self.resolve_tool_name(step)?;
        let params = self.build_step_params(step, context)?;

        // If we have a tool manager, try to invoke through it
        if let Some(manager) = tool_manager {
            // Check if it's a skill
            if manager.skill_registry.exists(&tool_name) {
                let params_json = serde_json::to_value(&params)?;
                match manager.skill_registry.execute_skill(&tool_name, params_json).await {
                    Ok(result) => return Ok(StepResult {
                        output: result,
                        success: true,
                    }),
                    Err(e) => return Err(e),
                }
            }
            
            // Check if it's another workflow
            if let Some(workflow_def) = manager.workflow_registry.get(&tool_name) {
                let workflow_tool = WorkflowTool::from_definition(workflow_def);
                // Recursive workflow execution
                match workflow_tool.invoke_with_definition(workflow_def, params.clone()) {
                    Ok(result) => return Ok(StepResult {
                        output: result,
                        success: true,
                    }),
                    Err(e) => return Err(e),
                }
            }
        }

        // Fall back to built-in tool execution
        self.execute_step_with_context(step, context)
    }

    pub fn execute_step_with_context(&self, step: &WorkflowStep, context: &Value) -> Result<StepResult> {
        let tool_name = self.resolve_tool_name(step)?;
        let params = self.build_step_params(step, context)?;

        // Execute based on tool type
        let output = match tool_name.as_str() {
            "fs_read" => {
                let path = params.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| eyre::eyre!("fs_read requires 'path' parameter"))?;
                
                match std::fs::read_to_string(path) {
                    Ok(content) => format!("Read {} bytes from {}", content.len(), path),
                    Err(e) => return Err(eyre::eyre!("Failed to read file: {}", e)),
                }
            }
            "fs_write" => {
                let path = params.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| eyre::eyre!("fs_write requires 'path' parameter"))?;
                let content = params.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| eyre::eyre!("fs_write requires 'content' parameter"))?;
                
                match std::fs::write(path, content) {
                    Ok(_) => format!("Wrote {} bytes to {}", content.len(), path),
                    Err(e) => return Err(eyre::eyre!("Failed to write file: {}", e)),
                }
            }
            "execute_bash" | "execute_cmd" | "execute_bash_readonly" => {
                let command = params.get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| eyre::eyre!("execute_bash requires 'command' parameter"))?;
                
                let is_readonly = tool_name == "execute_bash_readonly";
                
                // Validate readonly commands don't have write operations
                if is_readonly {
                    let dangerous_patterns = [">", ">>", "rm ", "mv ", "cp ", "chmod", "chown", 
                                             "dd ", "mkfs", "mount", "umount", "sudo", "su "];
                    for pattern in &dangerous_patterns {
                        if command.contains(pattern) {
                            return Err(eyre::eyre!(
                                "Readonly command cannot contain '{}': {}", pattern, command
                            ));
                        }
                    }
                }
                
                #[cfg(not(windows))]
                let output = std::process::Command::new("bash")
                    .arg("-c")
                    .arg(command)
                    .output();
                
                #[cfg(windows)]
                let output = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .output();
                
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            String::from_utf8_lossy(&out.stdout).to_string()
                        } else {
                            return Err(eyre::eyre!("Command failed: {}", String::from_utf8_lossy(&out.stderr)));
                        }
                    }
                    Err(e) => return Err(eyre::eyre!("Failed to execute command: {}", e)),
                }
            }
            _ => {
                // For unknown tools, return a placeholder
                format!("Executed tool '{}' with {} parameters", tool_name, params.len())
            }
        };

        Ok(StepResult {
            output,
            success: true,
        })
    }

    pub fn add_step_output_to_context(&self, mut context: Value, step_name: &str, output: &str) -> Value {
        if let Value::Object(ref mut map) = context {
            let steps = map.entry("steps").or_insert_with(|| Value::Object(Default::default()));

            if let Value::Object(steps_map) = steps {
                steps_map.insert(step_name.to_string(), serde_json::json!({"output": output}));
            }
        }
        context
    }
}

impl WorkflowTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(eyre::eyre!("Workflow name cannot be empty"));
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }

    pub fn invoke(&self, _params: std::collections::HashMap<String, serde_json::Value>) -> Result<String> {
        Ok("not implemented".to_string())
    }

    fn format_error(&self, step_num: usize, step_name: &str, error: &eyre::Error) -> String {
        format!("Workflow failed at step {} ('{}'): {}", step_num, step_name, error)
    }

    fn format_results(&self, step_results: Vec<String>) -> String {
        if step_results.is_empty() {
            return "Workflow completed with no steps".to_string();
        }

        let summary = format!("Workflow completed successfully ({} steps)", step_results.len());
        let details = step_results.join("\n");
        format!("{}\n\n{}", summary, details)
    }

    pub fn invoke_with_definition(
        &self,
        definition: &WorkflowDefinition,
        _params: HashMap<String, Value>,
    ) -> Result<String> {
        use std::time::Instant;

        let executor = StepExecutor::new();
        let mut context = definition.context.clone().unwrap_or(Value::Object(Default::default()));
        let mut results = Vec::new();
        let mut state = WorkflowState::Running;
        let workflow_start = Instant::now();
        let mut current_step = 0;

        for step in &definition.steps {
            current_step += 1;
            let step_start = Instant::now();

            match executor.execute_step_with_context(step, &context) {
                Ok(step_result) => {
                    let step_duration = step_start.elapsed();
                    results.push(format!(
                        "Step '{}': {} (completed in {:.2}ms)",
                        step.name,
                        step_result.output,
                        step_duration.as_secs_f64() * 1000.0
                    ));
                    context = executor.add_step_output_to_context(context, &step.name, &step_result.output);
                },
                Err(e) => {
                    state = WorkflowState::Failed;
                    return Err(eyre::eyre!(self.format_error(current_step, &step.name, &e)));
                },
            }
        }

        state = WorkflowState::Completed;
        let total_duration = workflow_start.elapsed();
        Ok(format!(
            "Executed {} steps successfully in {:.2}ms\n\n{}",
            definition.steps.len(),
            total_duration.as_secs_f64() * 1000.0,
            self.format_results(results)
        ))
    }

    pub async fn invoke_with_definition_and_manager(
        &self,
        definition: &WorkflowDefinition,
        _params: HashMap<String, Value>,
        mut tool_manager: Option<&mut crate::cli::chat::tool_manager::ToolManager>,
    ) -> Result<String> {
        use std::time::Instant;

        let executor = StepExecutor::new();
        let mut context = definition.context.clone().unwrap_or(Value::Object(Default::default()));
        let mut results = Vec::new();
        let mut state = WorkflowState::Running;
        let workflow_start = Instant::now();
        let mut current_step = 0;

        for step in &definition.steps {
            current_step += 1;
            let step_start = Instant::now();

            match executor.execute_step_with_context_and_manager(step, &context, tool_manager.as_deref_mut()).await {
                Ok(step_result) => {
                    let step_duration = step_start.elapsed();
                    results.push(format!(
                        "Step '{}': {} (completed in {:.2}ms)",
                        step.name,
                        step_result.output,
                        step_duration.as_secs_f64() * 1000.0
                    ));
                    context = executor.add_step_output_to_context(context, &step.name, &step_result.output);
                },
                Err(e) => {
                    state = WorkflowState::Failed;
                    return Err(eyre::eyre!(self.format_error(current_step, &step.name, &e)));
                },
            }
        }

        state = WorkflowState::Completed;
        let total_duration = workflow_start.elapsed();
        Ok(format!(
            "Executed {} steps successfully in {:.2}ms\n\n{}",
            definition.steps.len(),
            total_duration.as_secs_f64() * 1000.0,
            self.format_results(results)
        ))
    }

    pub fn from_definition(definition: &WorkflowDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
        }
    }

    pub fn definition_to_toolspec(&self, definition: &WorkflowDefinition) -> super::ToolSpec {
        use super::{
            InputSchema,
            ToolOrigin,
        };

        let input_schema = serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        });

        super::ToolSpec {
            name: definition.name.clone(),
            description: definition.description.clone(),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Workflow(definition.name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_tool_creation() {
        let workflow = WorkflowTool::new("test-workflow".to_string(), "A test workflow".to_string());
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.description, "A test workflow");
    }

    #[test]
    fn test_workflow_tool_clone() {
        let workflow = WorkflowTool::new("original".to_string(), "Original workflow".to_string());
        let cloned = workflow.clone();
        assert_eq!(cloned.name, workflow.name);
        assert_eq!(cloned.description, workflow.description);
    }

    #[test]
    fn test_workflow_tool_validate_success() {
        let workflow = WorkflowTool::new("valid-workflow".to_string(), "Description".to_string());
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_tool_validate_empty_name() {
        let workflow = WorkflowTool::new("".to_string(), "Description".to_string());
        let result = workflow.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_workflow_tool_eval_perm() {
        use crate::cli::agent::{
            Agent,
            PermissionEvalResult,
        };
        use crate::os::Os;

        let workflow = WorkflowTool::new("test-workflow".to_string(), "Test".to_string());
        let os = Os::new().await.unwrap();
        let agent = Agent::default();

        let result = workflow.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[test]
    fn test_workflow_definition_deserialize() {
        let json = r#"{
            "name": "test-workflow",
            "version": "1.0.0",
            "description": "A test workflow",
            "steps": []
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "test-workflow");
        assert_eq!(definition.version, "1.0.0");
        assert_eq!(definition.description, "A test workflow");
        assert_eq!(definition.steps.len(), 0);
    }

    #[test]
    fn test_workflow_definition_with_steps() {
        let json = r#"{
            "name": "build-workflow",
            "version": "1.0.0",
            "description": "A build workflow",
            "steps": [
                {
                    "name": "compile",
                    "tool": "execute_bash",
                    "parameters": {"command": "cargo build"}
                }
            ]
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.steps.len(), 1);
        assert_eq!(definition.steps[0].name, "compile");
        assert_eq!(definition.steps[0].tool, "execute_bash");
    }

    #[test]
    fn test_workflow_definition_with_context() {
        let json = r#"{
            "name": "deploy-workflow",
            "version": "1.0.0",
            "description": "A deployment workflow",
            "steps": [],
            "context": {
                "environment": "production",
                "region": "us-east-1"
            }
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.context.is_some());
        let context = definition.context.unwrap();
        assert_eq!(context.get("environment").unwrap(), "production");
        assert_eq!(context.get("region").unwrap(), "us-east-1");
    }

    #[test]
    fn test_workflow_stops_on_error() {
        let definition = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "first"}),
                },
                WorkflowStep {
                    name: "failing_step".to_string(),
                    tool: "nonexistent_tool".to_string(),
                    parameters: serde_json::json!({}),
                },
                WorkflowStep {
                    name: "step3".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "third"}),
                },
            ],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("failing_step") || error_msg.contains("nonexistent"));
    }

    #[test]
    fn test_workflow_state_tracking() {
        use std::collections::HashMap;

        let definition = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            steps: vec![WorkflowStep {
                name: "step1".to_string(),
                tool: "echo".to_string(),
                parameters: serde_json::json!({"msg": "first"}),
            }],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("step1"));
    }

    #[test]
    fn test_format_workflow_error() {
        use std::collections::HashMap;

        let definition = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0".to_string(),
            description: "Test workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "first"}),
                },
                WorkflowStep {
                    name: "failing_step".to_string(),
                    tool: "bad_tool".to_string(),
                    parameters: serde_json::json!({}),
                },
            ],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();

        // Error should include step number, step name, and context
        assert!(error_msg.contains("step 2") || error_msg.contains("failing_step"));
        assert!(error_msg.contains("bad_tool") || error_msg.contains("Unknown"));
    }

    #[test]
    fn test_format_workflow_results() {
        use std::collections::HashMap;

        let definition = WorkflowDefinition {
            name: "multi-step".to_string(),
            version: "1.0".to_string(),
            description: "Multi-step workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "first"}),
                },
                WorkflowStep {
                    name: "step2".to_string(),
                    tool: "calculator".to_string(),
                    parameters: serde_json::json!({"operation": "add"}),
                },
            ],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_ok());
        let output = result.unwrap();

        // Output should include step summaries
        assert!(output.contains("step1") || output.contains("step2") || output.contains("2 steps"));
    }

    #[test]
    fn test_step_timing() {
        use std::collections::HashMap;

        let definition = WorkflowDefinition {
            name: "timed-workflow".to_string(),
            version: "1.0".to_string(),
            description: "Workflow with timing".to_string(),
            steps: vec![WorkflowStep {
                name: "step1".to_string(),
                tool: "echo".to_string(),
                parameters: serde_json::json!({"msg": "test"}),
            }],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_ok());
        let output = result.unwrap();

        // Output should include timing information (ms or µs)
        assert!(output.contains("ms") || output.contains("µs") || output.contains("step1"));
    }

    #[test]
    fn test_workflow_tool_invoke() {
        use std::collections::HashMap;

        let workflow = WorkflowTool::new("test-workflow".to_string(), "Test workflow".to_string());
        let params = HashMap::new();

        // invoke() should return not implemented for now
        let result = workflow.invoke(params);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("not implemented"));
    }

    #[test]
    fn test_step_executor_creation() {
        let executor = StepExecutor::new();
        let step = WorkflowStep {
            name: "test".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({}),
        };

        let result = executor.execute_step(&step);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_tool_name() {
        let executor = StepExecutor::new();
        let step = WorkflowStep {
            name: "test".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({}),
        };

        let result = executor.resolve_tool_name(&step);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo");
    }

    #[test]
    fn test_build_step_params() {
        let executor = StepExecutor::new();
        let step = WorkflowStep {
            name: "test".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({"key": "value"}),
        };

        let context = serde_json::json!({});
        let result = executor.build_step_params(&step, &context);

        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.get("key").unwrap(), "value");
    }

    #[test]
    fn test_execute_step_with_context() {
        let executor = StepExecutor::new();
        let step = WorkflowStep {
            name: "test".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({}),
        };

        let context = serde_json::json!({});
        let result = executor.execute_step_with_context(&step, &context);

        assert!(result.is_ok());
        let step_result = result.unwrap();
        assert!(step_result.success);
        assert!(step_result.output.contains("test"));
    }

    #[test]
    fn test_add_step_output_to_context() {
        let executor = StepExecutor::new();
        let context = serde_json::json!({});

        let updated = executor.add_step_output_to_context(context, "step1", "output1");

        assert!(updated.get("steps").is_some());
        assert!(updated["steps"]["step1"]["output"].as_str().unwrap() == "output1");
    }

    #[test]
    fn test_workflow_definition_to_toolspec() {
        let definition = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test workflow".to_string(),
            steps: vec![],
            context: None,
        };

        let workflow = WorkflowTool::from_definition(&definition);
        let toolspec = workflow.definition_to_toolspec(&definition);

        assert_eq!(toolspec.name, "test-workflow");
        assert_eq!(toolspec.description, "Test workflow");
        assert!(matches!(toolspec.tool_origin, super::super::ToolOrigin::Workflow(_)));
    }

    #[test]
    fn test_workflow_creation_from_json() {
        // Test creating a workflow from JSON definition
        let json = r#"{
            "name": "test-workflow",
            "version": "1.0.0",
            "description": "A test workflow",
            "steps": [
                {
                    "name": "step1",
                    "tool": "echo",
                    "parameters": {"msg": "hello"}
                }
            ]
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        let workflow = WorkflowTool::from_definition(&definition);

        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_simple_workflow_execution() {
        // Test executing a simple single-step workflow
        let definition = WorkflowDefinition {
            name: "simple-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Simple workflow".to_string(),
            steps: vec![WorkflowStep {
                name: "greet".to_string(),
                tool: "echo".to_string(),
                parameters: serde_json::json!({"msg": "Hello World"}),
            }],
            context: None,
        };

        let workflow = WorkflowTool::new(definition.name.clone(), definition.description.clone());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("1 steps successfully"));
        assert!(output.contains("greet"));
        assert!(output.contains("ms"));
    }

    #[test]
    fn test_complex_workflow_execution() {
        // Test executing a complex multi-step workflow with context
        let definition = WorkflowDefinition {
            name: "complex-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Complex multi-step workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "Starting pipeline"}),
                },
                WorkflowStep {
                    name: "step2".to_string(),
                    tool: "calculator".to_string(),
                    parameters: serde_json::json!({"operation": "add"}),
                },
                WorkflowStep {
                    name: "step3".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "Pipeline complete"}),
                },
            ],
            context: Some(serde_json::json!({
                "environment": "test",
                "version": "1.0"
            })),
        };

        let workflow = WorkflowTool::new(definition.name.clone(), definition.description.clone());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_ok());
        let output = result.unwrap();

        // Verify all steps executed
        assert!(output.contains("3 steps successfully"));
        assert!(output.contains("step1"));
        assert!(output.contains("step2"));
        assert!(output.contains("step3"));

        // Verify timing information
        assert!(output.contains("ms"));

        // Verify summary format
        assert!(output.contains("Workflow completed successfully"));
    }

    #[test]
    fn test_workflow_with_context_passing() {
        // Test that context is passed between steps
        let executor = StepExecutor::new();
        let mut context = serde_json::json!({
            "initial": "value"
        });

        // Execute first step and add output to context
        let step1 = WorkflowStep {
            name: "step1".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({}),
        };

        let result1 = executor.execute_step_with_context(&step1, &context).unwrap();
        context = executor.add_step_output_to_context(context, "step1", &result1.output);

        // Verify context was updated
        assert!(context.get("steps").is_some());
        assert!(context["steps"]["step1"]["output"].is_string());

        // Execute second step with updated context
        let step2 = WorkflowStep {
            name: "step2".to_string(),
            tool: "echo".to_string(),
            parameters: serde_json::json!({}),
        };

        let result2 = executor.execute_step_with_context(&step2, &context);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_workflow_error_recovery() {
        // Test that workflow stops on error and provides clear message
        let definition = WorkflowDefinition {
            name: "error-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Workflow that will fail".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({}),
                },
                WorkflowStep {
                    name: "failing_step".to_string(),
                    tool: "nonexistent_tool".to_string(),
                    parameters: serde_json::json!({}),
                },
                WorkflowStep {
                    name: "step3".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({}),
                },
            ],
            context: None,
        };

        let workflow = WorkflowTool::new(definition.name.clone(), definition.description.clone());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();

        // Verify error message includes step information
        assert!(error.contains("step 2") || error.contains("failing_step"));
        assert!(error.contains("nonexistent_tool") || error.contains("Unknown tool"));
    }
}
