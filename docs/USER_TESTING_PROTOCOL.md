# Skills & Workflows User Testing Protocol

**Version**: 1.0  
**Date**: 2025-11-03  
**Purpose**: Validate that users can successfully discover, understand, and use skills through natural language

## Test Objectives

1. Validate users can discover available skills
2. Validate users can invoke skills through natural language
3. Validate error messages are clear and helpful
4. Validate feedback is useful and not overwhelming
5. Identify usability issues and pain points

## Test Participants

**Target**: 5 users with varying experience levels
- 2 beginners (new to CLI tools)
- 2 intermediate (familiar with CLI)
- 1 advanced (power user)

**Recruitment Criteria**:
- Has basic command line experience
- Has not used Q CLI skills feature before
- Available for 30-minute session

## Test Scenarios

### Scenario 1: Skill Discovery (5 min)

**Goal**: User discovers what skills are available

**Tasks**:
1. Find out what skills are available
2. Get details about a specific skill
3. Understand how to use a skill

**Success Criteria**:
- User finds `q skills list` command
- User understands skill descriptions
- User knows how to get more info

**Observation Points**:
- Does user find the list command easily?
- Is the output clear and understandable?
- Does user know what to do next?

### Scenario 2: Natural Language Invocation (10 min)

**Goal**: User invokes a skill through natural language

**Tasks**:
1. Use calculator skill to add two numbers
2. Use calculator skill to multiply numbers
3. Handle an error (division by zero)

**Success Criteria**:
- User successfully invokes skill via chat
- User understands execution feedback
- User recovers from error

**Observation Points**:
- Does user understand how to phrase requests?
- Is execution feedback helpful?
- Are error messages clear?

### Scenario 3: Error Recovery (5 min)

**Goal**: User recovers from common errors

**Tasks**:
1. Try to use non-existent skill
2. Follow error message suggestions
3. Successfully use correct skill

**Success Criteria**:
- User understands error message
- User follows recovery suggestions
- User successfully completes task

**Observation Points**:
- Are error messages clear?
- Are recovery tips actionable?
- Does user feel frustrated or guided?

### Scenario 4: Skill Creation (10 min)

**Goal**: User creates a simple skill

**Tasks**:
1. Discover how to create a skill
2. Create a simple "hello world" skill
3. Test the new skill

**Success Criteria**:
- User finds creation command
- User successfully creates skill
- User can use their new skill

**Observation Points**:
- Is creation process intuitive?
- Is documentation helpful?
- Does user feel confident?

## Testing Session Structure

### Pre-Test (5 min)
1. Welcome and introduction
2. Explain think-aloud protocol
3. Confirm recording consent
4. Brief background questions

### Test Execution (30 min)
1. Scenario 1: Discovery (5 min)
2. Scenario 2: Invocation (10 min)
3. Scenario 3: Error Recovery (5 min)
4. Scenario 4: Creation (10 min)

### Post-Test (5 min)
1. Overall impressions
2. What worked well?
3. What was confusing?
4. Suggestions for improvement

**Total Time**: 40 minutes per participant

## Observation Checklist

### For Each Scenario

**Task Completion**:
- [ ] Completed successfully
- [ ] Completed with help
- [ ] Partially completed
- [ ] Failed to complete

**Time to Complete**:
- [ ] < Expected time
- [ ] Within expected time
- [ ] > Expected time

**User Confidence**:
- [ ] Very confident
- [ ] Somewhat confident
- [ ] Not confident

**Errors/Issues**:
- [ ] None
- [ ] Minor (recovered easily)
- [ ] Major (needed help)

### Specific Observations

**Skill Discovery**:
- [ ] Found list command without help
- [ ] Understood skill descriptions
- [ ] Knew how to get more info
- [ ] Found empty state guidance helpful

**Natural Language Invocation**:
- [ ] Phrased requests naturally
- [ ] Understood execution feedback
- [ ] Noticed timing information
- [ ] Feedback was not overwhelming

**Error Handling**:
- [ ] Understood error messages
- [ ] Found tips helpful
- [ ] Followed recovery suggestions
- [ ] Successfully recovered

**Overall Experience**:
- [ ] Feature feels intuitive
- [ ] Documentation is helpful
- [ ] Feedback is appropriate
- [ ] Would use this feature

## Feedback Collection

### Quantitative Metrics

**Task Success Rate**:
- Scenario 1: ___% completed
- Scenario 2: ___% completed
- Scenario 3: ___% completed
- Scenario 4: ___% completed

**Time on Task**:
- Scenario 1: ___ minutes (expected: 5)
- Scenario 2: ___ minutes (expected: 10)
- Scenario 3: ___ minutes (expected: 5)
- Scenario 4: ___ minutes (expected: 10)

**Satisfaction Rating** (1-5 scale):
- Ease of use: ___
- Clarity of feedback: ___
- Error messages: ___
- Overall experience: ___

### Qualitative Feedback

**What worked well?**
- (Open-ended response)

**What was confusing?**
- (Open-ended response)

**What would you change?**
- (Open-ended response)

**Would you use this feature?**
- [ ] Yes, definitely
- [ ] Yes, probably
- [ ] Maybe
- [ ] Probably not
- [ ] Definitely not

**Why or why not?**
- (Open-ended response)

## Success Criteria

### Must Pass (Critical)
- [ ] 80%+ task completion rate across all scenarios
- [ ] 4+ average satisfaction rating
- [ ] No critical usability blockers identified
- [ ] Users can complete basic tasks without help

### Should Pass (Important)
- [ ] 90%+ task completion rate
- [ ] 4.5+ average satisfaction rating
- [ ] Users feel confident using the feature
- [ ] Error recovery is intuitive

### Nice to Have
- [ ] 100% task completion rate
- [ ] 5.0 average satisfaction rating
- [ ] Users express excitement about feature
- [ ] Users suggest advanced use cases

## Issue Tracking

### Issue Template

**Issue ID**: UT-001  
**Severity**: Critical / Major / Minor  
**Scenario**: (Which scenario)  
**Description**: (What happened)  
**User Quote**: (Exact words if relevant)  
**Frequency**: (How many users hit this)  
**Suggested Fix**: (Potential solution)

### Severity Definitions

**Critical**: Prevents task completion, affects 3+ users  
**Major**: Causes significant confusion, affects 2+ users  
**Minor**: Small inconvenience, affects 1 user

## Analysis Process

### After Each Session
1. Review notes and recordings
2. Document issues found
3. Update observation checklist
4. Note quotes and insights

### After All Sessions
1. Compile all feedback
2. Calculate success metrics
3. Prioritize issues by severity
4. Create action plan
5. Write testing report

## Deliverables

1. **Testing Report** (`docs/USER_TESTING_REPORT.md`)
   - Executive summary
   - Methodology
   - Findings by scenario
   - Issue list with priorities
   - Recommendations

2. **Issue List** (GitHub issues or similar)
   - One issue per identified problem
   - Tagged with severity and scenario
   - Assigned for resolution

3. **Action Plan** (in testing report)
   - Critical fixes (must do)
   - Important improvements (should do)
   - Nice-to-have enhancements (could do)
   - Timeline for fixes

## Testing Environment

**Setup Required**:
- Q CLI installed and configured
- Calculator skill available
- Clean state (no custom skills)
- Screen recording enabled
- Note-taking ready

**Test Data**:
- Example calculations: 5+3, 10*2, 15/0
- Example skill names: calculator, formatter
- Non-existent skill: my-calculator

## Notes for Facilitator

**Do**:
- Encourage think-aloud
- Observe without interfering
- Take detailed notes
- Record exact quotes
- Note non-verbal cues

**Don't**:
- Lead the user
- Explain how things work
- Interrupt during tasks
- Defend design choices
- Skip scenarios

**If User Gets Stuck**:
1. Wait 30 seconds
2. Ask: "What are you thinking?"
3. If still stuck, give minimal hint
4. Note that help was needed

## Post-Testing Actions

1. **Immediate** (same day):
   - Back up recordings
   - Write up notes
   - Document critical issues

2. **Within 24 hours**:
   - Transcribe key quotes
   - Create issue tickets
   - Share preliminary findings

3. **Within 1 week**:
   - Complete testing report
   - Present findings to team
   - Begin implementing fixes

---

**Protocol Version**: 1.0  
**Last Updated**: 2025-11-03  
**Owner**: Skills & Workflows Team
