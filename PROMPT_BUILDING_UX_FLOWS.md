# Prompt Building System - UX Design Flows

## User Journey Overview

### **Primary User Flow: Creating an AI Assistant Skill**
```
Start → Choose Method → Build/Iterate → Test → Save → Monitor
  ↓         ↓            ↓           ↓      ↓       ↓
Entry    Template/     Creation    Test   Deploy  Runtime
Point    Builder/      Iteration   Cases          Optimization
         Manual
```

## Detailed UX Flows

### **Flow 1: Template-Based Creation (Beginner-Friendly)**

```
q create skill code-reviewer guided

Creating skill 'code-reviewer'...

What type of skill do you want to create?
→ 1. Command Execution - Run shell commands and scripts
  2. AI Assistant - Chat-based conversational helper  ← [Selected]
  3. Text Template - Generate text with variables
  4. Interactive Session - Long-running interpreter

How do you want to create the prompt?
→ 1. Choose from pre-built templates  ← [Selected]
  2. Build step-by-step with guidance
  3. Write my own prompt

Available templates:
→ 1. Code Reviewer - Reviews code for security and best practices  ← [Selected]
  2. Documentation Writer - Creates clear technical documentation
  3. Domain Expert - Specialized knowledge assistant
  4. General Assistant - Flexible helper for various tasks

Template: Code Reviewer
"You are an expert code reviewer with 10+ years of experience. 
You focus on security vulnerabilities, performance optimization, 
and best practices. Always explain your reasoning and provide 
specific suggestions for improvement."

Customize this template?
→ 1. Use as-is  ← [Selected]
  2. Customize focus areas
  3. Adjust tone and style
  4. Add specific constraints

Creating test cases...
✓ Basic functionality test
✓ Edge case: empty code input
✓ Edge case: very large code file
✓ Security-focused review test

Testing your prompt...
Test 1/4: Basic functionality ✓ PASSED (4.2/5.0)
Test 2/4: Empty input handling ✓ PASSED (3.8/5.0)  
Test 3/4: Large file handling ✓ PASSED (3.9/5.0)
Test 4/4: Security focus ✓ PASSED (4.5/5.0)

Overall test score: 4.1/5.0 ✓ EXCELLENT

✓ Skill 'code-reviewer' created successfully!
  Saved to: .q-skills/code-reviewer.json
  Test cases: 4 created, 4 passing
  
Try it: /skills run code-reviewer
Monitor: Automatic optimization enabled (weekly reviews)
```

### **Flow 2: Interactive Builder (Intermediate Users)**

```
How do you want to create the prompt?
  1. Choose from pre-built templates
→ 2. Build step-by-step with guidance  ← [Selected]
  3. Write my own prompt

🔧 Prompt Builder - Step 1/4: Define Role

What should this assistant specialize in?
→ 1. Code and software development  ← [Selected]
  2. Writing and documentation
  3. Data analysis and research
  4. General problem solving
  5. Custom specialization

Great! What specific role within code/software?
→ 1. Code reviewer (security, quality, best practices)  ← [Selected]
  2. Debugging assistant (find and fix issues)
  3. Architecture advisor (design and patterns)
  4. Performance optimizer (speed and efficiency)
  5. Custom role

🎯 Prompt Builder - Step 2/4: Choose Capabilities

What should this code reviewer focus on? (select multiple: 1,2,4)
□ 1. Security vulnerabilities  ← [Selected]
□ 2. Performance optimization  ← [Selected]
□ 3. Code style and formatting
□ 4. Architecture and design  ← [Selected]
□ 5. Testing coverage
□ 6. Documentation quality

📋 Prompt Builder - Step 3/4: Set Constraints

How should the assistant behave?
□ 1. Always explain reasoning  ← [Selected]
□ 2. Provide specific examples  ← [Selected]
□ 3. Be concise and direct
□ 4. Ask clarifying questions
□ 5. Suggest alternatives  ← [Selected]

🔍 Prompt Builder - Step 4/4: Preview & Test

Generated prompt:
"You are an expert code reviewer specializing in security 
vulnerabilities, performance optimization, and architecture design. 
Always explain your reasoning, provide specific examples, and 
suggest alternative approaches when reviewing code."

Test this prompt? (Y/n): Y

Creating test cases...
Enter a typical code review request: 
> Review this function for security issues: def login(user, pass): return user == "admin"

Expected response should contain (keywords): security, vulnerability, password
Testing... ✓ Response contains required keywords
Quality score: 4.3/5.0 ✓ EXCELLENT

Satisfied with this prompt? (Y/n): Y
✓ Prompt created successfully!
```

### **Flow 3: Creation-Time Iteration (When Things Need Refinement)**

```
Testing your prompt...
Test 1/3: Basic functionality ✓ PASSED (4.1/5.0)
Test 2/3: Edge case handling ⚠ FAILED (2.1/5.0)
Test 3/3: Security focus ✓ PASSED (4.0/5.0)

Overall score: 3.4/5.0 ⚠ NEEDS IMPROVEMENT

The prompt failed on edge case handling. Let's improve it.

What would you like to improve? (select multiple: 1,3)
□ 1. Make instructions clearer  ← [Selected]
□ 2. Add more examples
□ 3. Add constraints for edge cases  ← [Selected]
□ 4. Adjust tone/style
□ 5. Start over with different approach

Improving prompt...

Updated prompt:
"You are an expert code reviewer... When encountering incomplete 
or unclear code, ask for clarification rather than making assumptions..."

Test again? (Y/n): Y

Testing improved prompt...
Test 1/3: Basic functionality ✓ PASSED (4.1/5.0)
Test 2/3: Edge case handling ✓ PASSED (3.8/5.0)  ← [Improved!]
Test 3/3: Security focus ✓ PASSED (4.0/5.0)

Overall score: 3.97/5.0 ✓ GOOD

Satisfied with this version? (Y/n): Y
✓ Skill created successfully!
```

### **Flow 4: Runtime Optimization (Background/Admin)**

```
# Weekly automated optimization report (shown to user)

📊 Skill Performance Report: code-reviewer

Performance This Week:
  Usage: 47 interactions
  Success Rate: 72% (↓ from 85% last week)
  User Satisfaction: 3.2/5 (↓ from 4.1/5 last week)
  Avg Response Time: 18s (↑ from 12s last week)

Issues Detected:
⚠ Users reporting responses are too verbose
⚠ Struggling with modern JavaScript frameworks
⚠ Missing context about project requirements

Suggested Improvements:
1. Simplify language to reduce response time (Confidence: 85%)
2. Add knowledge about React/Vue patterns (Confidence: 70%)
3. Ask for project context upfront (Confidence: 60%)

Apply automatic improvements? (Y/n): Y

Deploying optimized version...
├─ Creating test cases from recent usage ✓
├─ Running regression tests ✓ (4/4 passed)
├─ Deploying to 10% of users ✓
├─ Monitoring for 24 hours...
└─ Performance improved! Rolling out to all users ✓

New Performance:
  Success Rate: 89% (↑ 17%)
  User Satisfaction: 4.3/5 (↑ 1.1 points)
  Avg Response Time: 14s (↓ 4s)

✓ Optimization successful! Your skill is now performing better.
```

## Progressive Disclosure UX Strategy

### **Level 1: Beginner (Hide Complexity)**
```
Simple choices:
- "What should this assistant help with?" 
- "Use template" vs "Build custom"
- Basic testing with pass/fail
```

### **Level 2: Intermediate (Show Some Options)**
```
More control:
- Template customization options
- Multiple test case types
- Quality scoring details
- Basic iteration loop
```

### **Level 3: Advanced (Full Control)**
```
Expert features:
- Raw prompt editing
- Custom validation rules
- Advanced test case creation
- Performance analytics
- Manual optimization triggers
```

## Error States & Recovery UX

### **When Template Loading Fails**
```
⚠ Unable to load templates (network issue)

Don't worry! You can still create your skill:
→ 1. Try again
  2. Use offline templates
  3. Build from scratch
  4. Save and continue later
```

### **When Tests Fail Repeatedly**
```
😅 This prompt seems tricky to get right.

Let's try a different approach:
→ 1. Start with a simpler template
  2. Get help from examples
  3. Write it manually (skip testing)
  4. Save as draft and continue later

💡 Tip: Most users find the "Code Reviewer" template works well for this type of skill.
```

### **When Runtime Optimization Fails**
```
⚠ Automatic optimization didn't improve performance

Your skill is still working, but we couldn't make it better automatically.

Options:
→ 1. Keep current version (recommended)
  2. Try manual improvements
  3. Revert to previous version
  4. Get community suggestions

📊 Current performance is still acceptable (3.8/5.0)
```

## Accessibility & Inclusion Features

### **Plain Language Mode**
```
Technical: "Configure validation rules for output quality assessment"
Plain: "Set up checks to make sure responses are good quality"

Technical: "Iterate on prompt optimization parameters"  
Plain: "Try different ways to improve your assistant"
```

### **Contextual Help System**
```
What's a "system prompt"? [?]
├─ A system prompt tells the AI what role to play
├─ Example: "You are a helpful coding assistant"
├─ Good prompts are specific and clear
└─ [See examples] [Video tutorial] [Skip help]
```

### **Confidence Indicators**
```
Prompt Quality: ████████░░ 8/10 ✓ VERY GOOD
├─ Clear role definition ✓
├─ Specific capabilities ✓  
├─ Helpful constraints ✓
└─ Could add more examples ⚠

This prompt should work well for most users.
```

## Mobile/Responsive Considerations

### **Condensed Mobile Flow**
```
# Shorter prompts, fewer options per screen
Create skill: code-reviewer

Type: AI Assistant ✓

Method:
→ Template
  Custom

Template: Code Reviewer ✓
"Expert code reviewer focusing on..."

[Customize] [Test] [Save]

Tests: 3/3 ✓ 
Score: 4.1/5 ✓

[Save Skill]
```

This UX design ensures the complex prompt building system remains approachable for beginners while providing power features for advanced users, with clear error recovery and accessibility throughout.
