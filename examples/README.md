# Skills and Workflows Examples

This directory contains example skills and workflows to help you get started.

## Skills Examples

### hello.json
A simple skill that greets a person by name.

**Usage**:
```bash
q skills install examples/skills/hello.json
q chat "Say hello to Alice"
```

### count-lines.json
Counts the number of lines in a file.

**Usage**:
```bash
q skills install examples/skills/count-lines.json
q chat "Count lines in README.md"
```

## Workflows Examples

### hello-workflow.json
A simple workflow that demonstrates sequential step execution.

**Usage**:
```bash
q workflows add examples/workflows/hello-workflow.json
q chat "Run the hello workflow"
```

### data-pipeline.json
A more complex workflow showing context usage and multi-step processing.

**Usage**:
```bash
q workflows add examples/workflows/data-pipeline.json
q workflows show data-pipeline
```

## Creating Your Own

### Skills
1. Copy an example skill
2. Modify the name, description, and implementation
3. Install it: `q skills install my-skill.json`
4. Test it: `q chat "use my skill"`

### Workflows
1. Copy an example workflow
2. Add your steps
3. Add it: `q workflows add my-workflow.json`
4. Run it through chat

## See Also

- [Skills User Guide](../docs/SKILLS_USER_GUIDE.md)
- [Workflows User Guide](../docs/WORKFLOWS_USER_GUIDE.md)
- [Quick Start Guide](../docs/SKILLS_QUICKSTART.md)
