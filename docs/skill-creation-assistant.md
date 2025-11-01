# Skill Creation Assistant Design

## Overview

The Skill Creation Assistant is a focused chat interface that helps users create skills through guided conversation. It operates within strict boundaries to ensure skill creation remains the primary focus.

## Core Principles

### 1. Purpose-Limited Scope
- **Primary Goal**: Create a single skill JSON document
- **Secondary Goal**: Create supporting files referenced by the skill
- **Forbidden**: General development tasks, unrelated file modifications

### 2. Discovery-Based Approach
- Ask what the user is trying to accomplish
- Guide based on their goals, not assumptions
- Respect the constraints of the chosen skill type

### 3. Skill Type Awareness
Each skill type has specific capabilities and limitations:

| Skill Type | Purpose | Constraints |
|------------|---------|-------------|
| `command` | Execute single command | One command + args only |
| `template` | Generate text from template | Text output with parameters |
| `assistant` | AI conversation | Context-aware chat |
| `repl` | Interactive session | Persistent environment |

## System Architecture

### Entry Point
```
/skills create <name> <type>
```
Triggers skill creation assistant for the specified skill type.

### Assistant Capabilities

#### Allowed Operations
- **Read filesystem** - understand project structure
- **Create skill-related files** - scripts, configs, templates
- **Update skill JSON** - central configuration document
- **Reference existing files** - include in skill configuration
- **Ask clarifying questions** - understand user intent

#### Forbidden Operations
- **Modify unrelated files** - no general project changes
- **General Q&A** - stay focused on skill creation
- **System administration** - no broad system access

### Conversation Flow

1. **Discovery Phase**
   ```
   What are you trying to accomplish with this <type> skill?
   ```

2. **Configuration Phase**
   - Guide user through skill-specific options
   - Create supporting files if needed
   - Build skill JSON incrementally

3. **Validation Phase**
   - Verify skill configuration
   - Test if possible
   - Confirm completion

4. **Completion Phase**
   - Save skill JSON to `.q-skills/`
   - Return to main interface
   - Provide usage instructions

## Implementation Details

### Skill JSON Structure
All skills follow this base structure:
```json
{
  "name": "skill-name",
  "description": "What this skill does",
  "version": "1.0.0",
  "type": "command|template|assistant|repl",
  // Type-specific fields...
}
```

### Type-Specific Fields

#### Command Skills
```json
{
  "type": "command",
  "command": "executable",
  "args": ["arg1", "arg2"],
  "timeout": 30
}
```

#### Template Skills
```json
{
  "type": "template",
  "prompt": "Template with {parameters}",
  "parameters": [
    {
      "name": "param_name",
      "type": "string",
      "required": true
    }
  ]
}
```

#### Assistant Skills
```json
{
  "type": "assistant",
  "prompt_template": "You are a helpful assistant",
  "context_files": {
    "patterns": ["*.rs", "*.py"],
    "max_files": 10,
    "max_file_size_kb": 100
  }
}
```

#### REPL Skills
```json
{
  "type": "repl",
  "command": "python3",
  "session_config": {
    "session_timeout": 3600,
    "max_sessions": 5,
    "cleanup_on_exit": true
  }
}
```

### Prompt Testing and Refinement

For skills that involve prompts (template and assistant skills), the assistant should help users test and refine their prompts:

#### Template Skills
```
> /skills create email-gen template

What template are you trying to create?
> Professional email responses

Let me help you build and test the template.
Initial template: "Dear {recipient}, Thank you for {reason}. Best regards, {sender}"

Let's test this template:
- recipient: "John"  
- reason: "your inquiry"
- sender: "Alice"

Result: "Dear John, Thank you for your inquiry. Best regards, Alice"

Does this look right, or should we refine the template?
> Add a more formal tone and include company name

Updated template: "Dear {recipient}, Thank you for {reason}. We appreciate your business. Sincerely, {sender}, {company}"

Test again?
> Yes, same parameters plus company: "Acme Corp"

Result: "Dear John, Thank you for your inquiry. We appreciate your business. Sincerely, Alice, Acme Corp"

✅ Template refined and tested. Skill ready!
```

#### Assistant Skills  
```
> /skills create code-reviewer assistant

What should this assistant help with?
> Review Python code for best practices

Let me help you craft the prompt:
Initial prompt: "You are a Python code reviewer. Analyze code for best practices."

Let's test this prompt with sample code:
```python
def calc(x,y):
    return x+y
```

Assistant response: "This function lacks type hints, docstring, and proper spacing..."

Is this the kind of review you want?
> Yes, but also check for security issues

Updated prompt: "You are a Python code reviewer. Analyze code for best practices, security vulnerabilities, and maintainability."

Test again with the same code?
> Yes

Assistant response: "This function needs: 1) Type hints 2) Docstring 3) Input validation for security..."

Better? Ready to save this assistant skill?
> Perfect!

✅ Assistant prompt tested and refined. Skill ready!
```

#### Test Case Management

The assistant maintains a collection of test cases throughout the skill creation process:

**Test Case Structure:**
```json
{
  "test_cases": [
    {
      "name": "basic_email",
      "description": "Standard professional email",
      "inputs": {
        "recipient": "John Smith",
        "reason": "your inquiry about our services",
        "sender": "Alice Johnson",
        "company": "Acme Corp"
      },
      "expected_output": "Professional tone with company branding"
    },
    {
      "name": "urgent_response", 
      "description": "Urgent customer issue",
      "inputs": {
        "recipient": "Sarah Wilson",
        "reason": "the urgent issue you reported",
        "sender": "Support Team",
        "company": "Acme Corp"
      },
      "expected_output": "Urgent but professional tone"
    }
  ]
}
```

**Test Case Workflow:**

1. **Initial Test Cases**
   ```
   Let me create some test cases for your email template:
   
   Test Case 1: "basic_email" 
   - Standard professional inquiry response
   
   Test Case 2: "formal_complaint"
   - Response to customer complaint
   
   Should I add more test scenarios?
   ```

2. **User Adds Test Cases**
   ```
   > I need to test with international customers too
   
   Adding Test Case 3: "international_customer"
   What parameters should I use for this case?
   > recipient: "Hans Mueller", company: "Acme GmbH"
   
   ✅ Test case added. Running all tests...
   ```

3. **Issue Discovery & New Tests**
   ```
   Test results show the template doesn't handle missing company names well.
   
   Should I add a test case for this edge case?
   > Yes, test with no company parameter
   
   Test Case 4: "no_company"
   Result: "Sincerely, Alice, " (trailing comma issue)
   
   Let me fix the template to handle optional company names...
   ```

4. **Regression Testing**
   ```
   Template updated. Re-running all 4 test cases:
   ✅ basic_email - PASS
   ✅ formal_complaint - PASS  
   ✅ international_customer - PASS
   ✅ no_company - PASS (fixed trailing comma)
   
   All tests passing! Ready to save skill?
   ```

**Test Case Commands:**
- `add test case` - Add new test scenario
- `run tests` - Execute all test cases
- `show test results` - Display latest test outputs
- `remove test case` - Delete unnecessary tests
- `edit test case` - Modify existing test parameters

**Test Case Storage:**
Test cases are stored alongside the skill JSON:
```
.q-skills/
├── email-gen.json          # Main skill configuration
└── email-gen.tests.json    # Test cases for this skill
```

**Benefits:**
- **Comprehensive testing** - catch edge cases early
- **Regression prevention** - ensure changes don't break existing functionality  
- **Documentation** - test cases serve as usage examples
- **Confidence** - user knows skill works across scenarios

#### Testing Workflow

1. **Initial Prompt Creation**
   - User describes what they want
   - Assistant suggests initial prompt/template

2. **Test Phase**
   - Assistant provides sample inputs
   - Shows expected outputs
   - User evaluates results

3. **Refinement Phase**
   - User requests changes
   - Assistant updates prompt/template
   - Re-test with same or new inputs

4. **Validation Phase**
   - Final test with user's real data
   - Confirm prompt meets requirements
   - Save skill configuration

### Error Handling

- **Invalid skill type** - show available types
- **Missing required fields** - prompt for missing information
- **File creation failures** - offer alternatives
- **JSON validation errors** - guide user to fix issues

### Security Considerations

- **Sandboxed execution** - no access to sensitive system areas
- **File creation limits** - only in appropriate directories
- **Command validation** - warn about potentially dangerous commands
- **Parameter sanitization** - validate user inputs

## User Experience

### Success Criteria
- User can create a working skill in under 5 minutes
- **Prompts are tested and refined during creation**
- No need to manually edit JSON files
- Clear understanding of what the skill will do
- Immediate ability to test the created skill
- **Template/assistant skills produce expected outputs**

### Example Interactions

#### Command Skill Creation
```
> /skills create backup-db command

🛠️ Skill Creation Assistant
Creating command skill: backup-db

What are you trying to accomplish with this skill?
> Create database backups

What command should execute the backup?
> pg_dump

What database should it backup by default?
> myapp_production

Should users be able to specify different databases?
> Yes, add a database parameter

✅ Skill created: backup-db
Command: pg_dump myapp_production
Parameters: database (optional)
Use: /skills run backup-db --params '{"database":"other_db"}'
```

#### Template Skill Creation
```
> /skills create readme-gen template

🛠️ Skill Creation Assistant  
Creating template skill: readme-gen

What are you trying to accomplish with this skill?
> Generate README files for new projects

What should the README template include?
> Project name, description, installation steps

I'll create a template with those sections.
What parameters should users provide?
> project_name, description, install_command

✅ Skill created: readme-gen
Template: README with {project_name}, {description}, {install_command}
Use: /skills run readme-gen --params '{"project_name":"MyApp",...}'
```

## Future Enhancements

- **Skill templates** - pre-built skill configurations
- **Advanced testing** - automated test case generation
- **Test case sharing** - export/import test suites between skills
- **Performance testing** - measure skill execution time
- **Skill sharing** - export/import skill configurations
- **Advanced parameters** - complex parameter validation
- **Multi-file skills** - skills that manage multiple files
- **Test coverage analysis** - identify untested parameter combinations

## Success Metrics

- **Creation time** - average time to create a working skill
- **Success rate** - percentage of skills that work on first try
- **User satisfaction** - feedback on creation experience
- **Skill usage** - how often created skills are actually used
