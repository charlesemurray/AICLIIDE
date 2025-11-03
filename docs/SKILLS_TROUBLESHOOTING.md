# Skills Troubleshooting Guide

Quick solutions to common issues with Q Skills.

## Common Issues

### Skill Not Found

**Problem**: `Skill 'my-skill' not found`

**Solutions**:
1. Check skill exists: `ls ~/.q-skills/my-skill.json`
2. List available skills: `q skills list`
3. Verify skill name spelling
4. Ensure file is in correct directory

**Example**:
```bash
# Check if skill file exists
ls ~/.q-skills/

# List loaded skills
q skills list
```

### Invalid JSON

**Problem**: `Invalid JSON: expected value at line 1`

**Solutions**:
1. Validate JSON: `cat ~/.q-skills/my-skill.json | jq .`
2. Check for missing commas, quotes, brackets
3. Use a JSON validator
4. Compare with example skills

**Example**:
```bash
# Validate JSON syntax
jq . ~/.q-skills/my-skill.json

# Pretty-print to find issues
cat ~/.q-skills/my-skill.json | jq .
```

### Execution Failed

**Problem**: `Skill execution failed: command not found`

**Solutions**:
1. Test command manually: `echo test`
2. Check command is in PATH: `which <command>`
3. Use full path to command: `/usr/bin/echo`
4. Verify file permissions: `ls -la`

**Example**:
```bash
# Test command works
echo "Hello"

# Check if command exists
which echo

# Use full path in skill
/usr/bin/echo "Hello"
```

### Division by Zero

**Problem**: `Invalid input: Division by zero`

**Solutions**:
1. Check parameter values
2. Validate input before division
3. Handle zero case in skill logic

### Missing Parameters

**Problem**: `Missing or invalid parameter 'x'`

**Solutions**:
1. Check required parameters: `q skills info <skill-name>`
2. Provide all required parameters
3. Check parameter types (string, number, boolean)
4. Use correct parameter names

**Example**:
```bash
# Check what parameters are needed
q skills info calculator

# Provide all required parameters
q chat "use calculator with a=5, b=3, op=add"
```

## Debugging Steps

### 1. Verify Skill File

```bash
# Check file exists
ls -la ~/.q-skills/my-skill.json

# Validate JSON
jq . ~/.q-skills/my-skill.json

# Check file permissions
chmod 644 ~/.q-skills/my-skill.json
```

### 2. Test Command Manually

```bash
# Extract command from skill
cat ~/.q-skills/my-skill.json | jq -r '.implementation.command'

# Run command manually
echo "test"
```

### 3. Check Skill Loading

```bash
# List all skills
q skills list

# Get skill details
q skills info my-skill

# Check for errors in output
```

### 4. Enable Verbose Mode

```bash
# Run with verbose output
q chat "use my-skill" --verbose

# Check logs
tail -f ~/.q/logs/skills.log
```

## FAQ

### Q: Where are skills stored?

**A**: Skills are stored in `~/.q-skills/` directory. Each skill is a separate JSON file.

### Q: How do I reload skills?

**A**: Skills are loaded automatically. Restart Q CLI or run `q skills list` to reload.

### Q: Can I have multiple versions of a skill?

**A**: No, skill names must be unique. Use different names for different versions.

### Q: How do I share skills?

**A**: Copy the JSON file and share it. Others can place it in their `~/.q-skills/` directory.

### Q: What if my skill needs dependencies?

**A**: Ensure dependencies are installed and in PATH. Document requirements in skill description.

### Q: Can skills call other skills?

**A**: Not directly. Use workflows to chain multiple skills together.

## Error Messages Reference

### `Skill not found`
- Skill file doesn't exist
- Wrong skill name
- File not in ~/.q-skills/

### `Invalid JSON`
- Syntax error in JSON
- Missing quotes, commas, brackets
- Invalid escape sequences

### `Missing implementation`
- No implementation field
- Implementation is empty
- Wrong implementation type

### `Execution failed`
- Command not found
- Permission denied
- Command returned error

### `Invalid input`
- Wrong parameter type
- Missing required parameter
- Invalid parameter value

### `Timeout`
- Skill took too long
- Infinite loop
- Blocking operation

## Getting Help

### Documentation
- Quick Start: `docs/SKILLS_QUICKSTART.md`
- User Guide: `docs/SKILLS_USER_GUIDE.md`
- Examples: `examples/skills/`

### Commands
- List skills: `q skills list`
- Get info: `q skills info <name>`
- Show help: `q skills help`

### Support
- Check examples in `examples/skills/`
- Review error messages carefully
- Test commands manually first

## Best Practices

### 1. Test Commands First
Always test commands manually before adding to skills:
```bash
# Test command works
echo "Hello, World!"

# Then add to skill
```

### 2. Validate JSON
Use `jq` to validate JSON before saving:
```bash
cat skill.json | jq .
```

### 3. Use Descriptive Names
Choose clear, descriptive skill names:
- ✓ `format-json`
- ✓ `count-lines`
- ✗ `skill1`
- ✗ `test`

### 4. Add Good Descriptions
Help users understand what your skill does:
```json
{
  "name": "format-json",
  "description": "Format and validate JSON data with proper indentation"
}
```

### 5. Document Parameters
Clearly describe what each parameter does:
```json
{
  "parameters": [
    {
      "name": "input",
      "type": "string",
      "required": true,
      "description": "JSON string to format"
    }
  ]
}
```

---

**Last Updated**: 2025-11-03  
**Version**: 1.0
