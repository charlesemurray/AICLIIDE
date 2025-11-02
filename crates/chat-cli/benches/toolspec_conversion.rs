use chat_cli::cli::skills::toolspec_conversion::ToToolSpec;
use chat_cli::cli::skills::types::JsonSkill;
use chat_cli::cli::workflow::types::{
    StepType,
    Workflow,
    WorkflowInput,
    WorkflowStep,
};
use criterion::{
    Criterion,
    black_box,
    criterion_group,
    criterion_main,
};
use serde_json::json;

fn create_test_skill() -> JsonSkill {
    JsonSkill {
        name: "test_skill".to_string(),
        description: Some("A test skill for benchmarking".to_string()),
        parameters: Some(vec![
            serde_json::from_value(json!({
                "name": "input1",
                "type": "string",
                "required": true
            }))
            .unwrap(),
            serde_json::from_value(json!({
                "name": "input2",
                "type": "number",
                "required": false
            }))
            .unwrap(),
        ]),
        implementation: None,
        security: None,
    }
}

fn create_test_workflow() -> Workflow {
    Workflow {
        name: "test_workflow".to_string(),
        description: "A test workflow for benchmarking".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: StepType::Skill {
                    name: "skill1".to_string(),
                    inputs: json!({"param": "value"}),
                },
            },
            WorkflowStep {
                id: "step2".to_string(),
                step_type: StepType::Skill {
                    name: "skill2".to_string(),
                    inputs: json!({"param": "{{step1.output}}"}),
                },
            },
        ],
        inputs: vec![WorkflowInput {
            name: "workflow_input".to_string(),
            input_type: "string".to_string(),
            required: true,
        }],
    }
}

fn bench_skill_to_toolspec(c: &mut Criterion) {
    let skill = create_test_skill();
    c.bench_function("skill_to_toolspec", |b| {
        b.iter(|| black_box(skill.to_toolspec().unwrap()));
    });
}

fn bench_workflow_to_toolspec(c: &mut Criterion) {
    let workflow = create_test_workflow();
    c.bench_function("workflow_to_toolspec", |b| {
        b.iter(|| black_box(workflow.to_toolspec().unwrap()));
    });
}

fn bench_skill_to_toolspec_complex(c: &mut Criterion) {
    let mut skill = create_test_skill();
    // Add more parameters to make it complex
    skill.parameters = Some(
        (0..10)
            .map(|i| {
                serde_json::from_value(json!({
                    "name": format!("param{}", i),
                    "type": "string",
                    "required": i % 2 == 0
                }))
                .unwrap()
            })
            .collect(),
    );

    c.bench_function("skill_to_toolspec_complex", |b| {
        b.iter(|| black_box(skill.to_toolspec().unwrap()));
    });
}

fn bench_workflow_to_toolspec_complex(c: &mut Criterion) {
    let workflow = Workflow {
        name: "complex_workflow".to_string(),
        description: "A complex workflow with many steps".to_string(),
        version: "1.0.0".to_string(),
        steps: (0..10)
            .map(|i| WorkflowStep {
                id: format!("step{}", i),
                step_type: StepType::Skill {
                    name: format!("skill{}", i),
                    inputs: json!({"param": format!("value{}", i)}),
                },
            })
            .collect(),
        inputs: vec![
            WorkflowInput {
                name: "input1".to_string(),
                input_type: "string".to_string(),
                required: true,
            },
            WorkflowInput {
                name: "input2".to_string(),
                input_type: "number".to_string(),
                required: false,
            },
        ],
    };

    c.bench_function("workflow_to_toolspec_complex", |b| {
        b.iter(|| black_box(workflow.to_toolspec().unwrap()));
    });
}

criterion_group!(
    benches,
    bench_skill_to_toolspec,
    bench_workflow_to_toolspec,
    bench_skill_to_toolspec_complex,
    bench_workflow_to_toolspec_complex
);
criterion_main!(benches);
