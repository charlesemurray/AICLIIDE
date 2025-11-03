/// Tests for skills onboarding experience
use chat_cli::cli::skills::onboarding;

#[test]
fn test_tutorial_shows_welcome() {
    let mut output = Vec::new();
    onboarding::show_tutorial(&mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Welcome to Q Skills!"));
    assert!(text.contains("🎉"));
}

#[test]
fn test_tutorial_shows_quick_start() {
    let mut output = Vec::new();
    onboarding::show_tutorial(&mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Quick Start:"));
    assert!(text.contains("1."));
    assert!(text.contains("2."));
    assert!(text.contains("3."));
}

#[test]
fn test_tutorial_shows_commands() {
    let mut output = Vec::new();
    onboarding::show_tutorial(&mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("q skills list"));
    assert!(text.contains("q chat"));
    assert!(text.contains("q skills info"));
}

#[test]
fn test_tutorial_shows_resources() {
    let mut output = Vec::new();
    onboarding::show_tutorial(&mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("examples/skills/"));
    assert!(text.contains("docs/SKILLS_QUICKSTART.md"));
}

#[test]
fn test_tutorial_shows_example_usage() {
    let mut output = Vec::new();
    onboarding::show_tutorial(&mut output).unwrap();

    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("calculator"));
    assert!(text.contains("add 5 and 3"));
}
