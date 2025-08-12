# Planning Poker Completion Specification Working Guide

## Purpose

This guide shapes how AI agents should approach discussions and work on the Planning Poker completion specification to maximize value, maintain focus, and properly document progress toward a production-ready application.

## Core Principles

### The Spec is a Living Work Log

The completion spec (@spec/completion/overview.md) is not just documentation - it's an active audit and work tracking system that:

- Tracks the transformation from prototype to production-ready application
- Serves as the single source of truth for completion efforts
- Documents both what was done and how it was validated
- Provides clear visibility into project progress

### Document Your Work Thoroughly

Every checkbox marked as complete MUST include:

1. **Location details** - Specific files, line numbers, packages affected
2. **Implementation evidence** - What was done and how
3. **Validation proof** - How you verified it works (tests, manual verification, etc.)

## Discussion Framework

### 1. Start with Orientation

Before diving into any specific topic, establish context:

- What aspect of the application are we discussing?
- Is this about identifying missing features, tracking progress, or implementing solutions?
- Are we working within existing patterns or proposing new approaches?

### 2. Respect the Document Structure

The spec follows a completion audit pattern:

- **Feature identification** → What needs to be built/fixed
- **Status tracking** → Current state of each item
- **Solution documentation** → How things were/will be implemented
- **Execution planning** → Order and dependencies

Always frame discussions within this structure rather than creating parallel taxonomies.

### 3. Use Precise Language

- **"Complete"** means fully implemented with evidence and validation
- **"In Progress"** means actively being worked on with partial completion noted
- **"Blocked"** means waiting on dependencies (document what's blocking)
- **"Pattern"** refers to a reusable solution approach
- **"Implementation"** refers to converting designs into working code

### 4. Follow the Abstraction Hierarchy

Discussions should respect these levels:

1. **Architectural** - Overall application structure and principles
2. **Feature** - User-facing functionality (game creation, voting, etc.)
3. **Component** - Specific modules (WebSocket handling, state management, etc.)
4. **Implementation** - Actual code changes

Start discussions at the appropriate level and explicitly move between levels.

## The Checkbox Protocol

### Good Documentation Example

```markdown
- [x] `packages/poker/src/lib.rs:45-89` - Vote calculation logic ✅ COMPLETED
  - Lines 45-67: Implemented median calculation for story points
  - Lines 68-78: Added outlier detection for consensus checking
  - Lines 79-89: Vote validation and sanitization
  - Validation: Unit tests pass, tested with simulator scenarios
  - Pattern: Pure functions for business logic calculations
```

### Poor Documentation Example

```markdown
- [x] Fixed voting logic ✅
```

## Working Patterns

### Pattern: Status Check

"What's the state of X?"

1. Locate X in the spec
2. Report its status marker with completion percentage if applicable
3. Identify dependencies/blockers
4. Reference the documented evidence of completion

### Pattern: Solution Proposal

"How should we handle Y?"

1. Check if Y is already addressed (look for ✅ markers)
2. Find similar solved problems (look for patterns in completed work)
3. Propose applying existing patterns (reference specific examples)
4. Only suggest new patterns if necessary

### Pattern: Work Planning

"What should we do next?"

1. Reference current phase/progress with percentages
2. Identify incomplete items (look for 🟡 markers)
3. Check for unblocked work (avoid ❌ items)
4. Consider parallel opportunities

### Pattern: Feature Discovery

"I found missing functionality Z"

1. Categorize the type of feature (core game, UI, infrastructure)
2. Check if it's already documented
3. Assess impact and priority
4. Add to spec with proper structure if genuinely new

## The Planning Poker Mental Model

The core pattern for Planning Poker completion is building a robust real-time application:

- **Client-Server Architecture** - WebSocket-based real-time communication
- **State Management** - Consistent game state across all clients
- **Error Recovery** - Graceful handling of network issues and edge cases
- **User Experience** - Intuitive interface for distributed teams

Frame solutions within this model. When documenting implementations, show how they contribute to the overall user experience.

## Documentation Standards

### File References

Always use: `packages/[package_name]/src/[file].rs:[line_numbers]`

### Status Progression

- ❌ Blocked → Document what's blocking and why
- 🟡 In Progress → Show percentage complete and what's done
- ✅ Complete → Include full implementation details and validation

### Implementation Documentation

```markdown
- [x] `packages/[name]/src/[file].rs:[lines]` - [Feature] ✅ IMPLEMENTED
  - Before: [Previous state or missing functionality]
  - After: [What was implemented]
  - Validation: [How verified - tests, manual testing, etc.]
  - Pattern: [Reusable approach for similar features]
```

## Meta-Principles

### The Spec is Authoritative

- If it's not in the spec, it's not part of the completion effort
- If it contradicts the spec, the spec wins
- If the spec is wrong, update the spec first with evidence

### Patterns Over Instances

- Look for reusable solutions in completed work
- Document patterns when you find them
- Enable parallel work through patterns

### Explicit Over Implicit

- Document blockers clearly
- State assumptions explicitly
- Make dependencies visible
- Include validation evidence

## Quality Checklist

Before marking any task complete:

- [ ] Documented all affected files with paths and line numbers
- [ ] Described what was actually implemented
- [ ] Noted how it was validated/tested
- [ ] Identified any patterns for reuse
- [ ] Updated percentages if applicable
- [ ] Marked follow-up work if needed

## How This Guide Helps

By following this framework, work on the Planning Poker completion spec will:

- Build on existing patterns rather than reinventing solutions
- Create an audit trail of changes and validations
- Enable parallel work through clear documentation
- Maintain clear communication about progress
- Provide evidence of completion
- Build a knowledge base for future maintenance

The goal is not just to check boxes, but to create a comprehensive record of building a production-ready Planning Poker application where every feature is documented, validated, and maintainable.
