---
description: Create a detailed multi-phase plan with sub-agent ownership and parallel reviews for Rust CLI/library development
---

# Multi-Phase Planning with Sub-Agent Review (TA Edition)

You have been asked to create a comprehensive implementation plan for the TA (TypeScript Analyzer) project. This command orchestrates a sophisticated planning workflow that leverages specialized sub-agents for domain expertise and parallel review.

**IMPORTANT:** Use the TodoWrite tool to track your progress through these steps.

## Overview

This planning workflow:

1. Gathers requirements and analyzes the task
2. Creates a detailed plan with phases and assigns principal owners
3. Launches parallel reviews by domain specialists
4. Consolidates feedback and identifies parallelization opportunities
5. Produces a final, implementation-ready plan

## Available Sub-Agents (Principal Owners)

| Sub-Agent | Domain | Assign When |
|-----------|--------|-------------|
| **Rust Developer** | Core library, CLI, OXC integration, performance-critical code | Systems programming, AST analysis, symbol tracking, trait design |
| **Feature Tester (Rust)** | Rust testing strategy, cargo test, proptest | Testing strategy, TDD workflow, property-based tests |

**NOTE:** Database and Schema agents are NOT used for TA, as this project has no database or complex data modeling requirements. Frontend agent is also not used (no UI).

---

## Prerequisites

Before starting:

1. **Ensure required directories exist:**

   ```bash
   mkdir -p .ai/plans .ai/logs
   ```

2. **Verify sub-agent definitions are accessible:**

   These commands require sub-agent definitions in one of these locations:
   - `.claude/agents/` (project-level, preferred)
   - `~/.claude/agents/` (user-level, fallback)

   Required agent files:
   - `agents/rust-developer.md`
   - `agents/feature-tester-rust.md`

---

## Step 0: Detect Requested Skills

**Purpose:** Identify which Claude Code skills the user wants to activate for this planning session.

**Actions:**

1. **Check user request for skill mentions:**
   - Look for phrases like "use the [skill-name] skill"
   - Look for skill activation requests in user's message
   - Examples: "use rust-testing", "activate the clap skill", "with thiserror skill"

2. **Parse skill list:**
   - Extract skill names (e.g., `rust-testing`, `clap`, `thiserror`, `rust-logging`)
   - Create a comma-separated list for passing to sub-agents

3. **Communicate to user via STDOUT:**
   ```text
   📋 Planning with skills: [skill1, skill2, skill3]

   These skills will be activated for all sub-agents during plan review.
   ```

4. **If no skills requested:**
   - Use default skill set based on project type
   - For Rust projects: `rust-testing`, `thiserror`
   - Communicate: "📋 Using default skills for Rust: rust-testing, thiserror"

5. **Store skills for sub-agent prompts:**
   - Keep the skill list available for use in Step 3 (review prompts)

---

## Step 1: Requirements Gathering

### 1.1 Understand the Task

Ask the user clarifying questions to fully understand what needs to be built:

1. **What is being built?**
   - Feature name and description
   - Primary goal and business value
   - Which module(s) affected: `/lib`, `/cli`, `/ts`

2. **Who are the stakeholders?**
   - End users (CLI users, TypeScript developers)
   - Integration scenarios (CI/CD pipelines, editor integrations)

3. **What are the constraints?**
   - Performance requirements (AST parsing speed, watch latency)
   - Compatibility needs (OXC version, TypeScript versions)
   - Timeline expectations (scope, not duration)

### 1.2 Identify Requirements

Document both functional and non-functional requirements:

**Functional Requirements (FR):**

- What the system should DO
- CLI commands and arguments
- Analysis features (type errors, symbols, dependencies, tests)
- Output formats (console, JSON, HTML)
- Watch event types and handlers

**Non-Functional Requirements (NFR):**

- Performance (AST parsing speed, analysis throughput)
- Reliability (error handling with anyhow/thiserror, graceful degradation)
- Maintainability (module structure, documentation)
- Usability (CLI interface, error messages, output formatting)

### 1.3 Codebase Analysis

Use the Task tool with `subagent_type=Explore` to understand the current codebase:

```
Explore the codebase to understand:
1. Existing architecture and module structure (/lib, /cli, /ts)
2. Relevant Rust modules and files
3. Testing infrastructure (cargo test setup, unit tests)
4. Build processes
5. OXC integration patterns
6. CLI command structure and output formatting
```

---

## Step 2: Create the Initial Plan

### 2.1 Plan Structure

Create a plan document at `.ai/plans/YYYY-MM-DD.plan-name.md`:

```markdown
# [Plan Name]

**Created:** [Date]
**Status:** Draft - Awaiting Review

## Executive Summary

[2-3 sentence overview of what this plan accomplishes]

## Requirements

### Functional Requirements

| ID | Requirement | Priority | Owner |
|----|-------------|----------|-------|
| FR-1 | [requirement] | High/Med/Low | [sub-agent] |
| FR-2 | [requirement] | High/Med/Low | [sub-agent] |

### Non-Functional Requirements

| ID | Requirement | Target | Owner |
|----|-------------|--------|-------|
| NFR-1 | [requirement] | [metric] | [sub-agent] |
| NFR-2 | [requirement] | [metric] | [sub-agent] |

## Architecture Overview

[High-level architecture description]

### Component Diagram

[ASCII or description of component relationships between lib, cli, and ts modules]

### Data Flow

[How data moves through the system - source files → OXC parser → AST → analysis → output]

## Phases

### Phase 1: [Phase Name]

**Principal Owner:** [Rust Developer/Feature Tester]

**Goal:** [What this phase accomplishes]

**Dependencies:** None / [list dependencies]

**Blast Radius:** [Test scope - cargo test command or empty string for all tests]

**Deliverables:**
- [Deliverable 1]
- [Deliverable 2]

**Technical Details:**
- Modules to create/modify (in /lib or /cli)
- Key traits/structs/enums
- OXC integration patterns
- Integration points with existing code

**Acceptance Criteria:**
- [ ] [Criterion 1]
- [ ] [Criterion 2]

---

### Phase 2: [Phase Name]

[Repeat structure - include Blast Radius field]

---

## Blast Radius Analysis

For each phase, determine the **blast radius** - the scope of tests that should be run to verify both new functionality AND detect unintended downstream effects.

### How to Determine Blast Radius

1. **Identify direct test files:**
   - Tests for modules being created/modified
   - Example: `cargo test --lib analysis` for analysis module changes

2. **Identify downstream dependencies:**
   - What modules import/depend on the code being changed?
   - What tests cover those dependent modules?

3. **Construct the test command:**
   - Use `cargo test --lib [module]` for library changes
   - Use `cargo test --bin cli` for CLI changes
   - Use `cargo test -p lib` or `cargo test -p cli` for package-specific tests
   - Use `cargo test` (empty string equivalent) for full coverage

4. **Use full test suite for foundational changes:**
   - If changes affect core traits, error types, or shared utilities
   - If unsure about impact scope
   - Empty string `""` or `cargo test` runs ALL tests

### Blast Radius Examples

| Change Type | Blast Radius |
|-------------|--------------|
| New analysis feature in /lib | `cargo test -p lib` |
| CLI command addition | `cargo test -p cli` |
| Output formatter changes | `cargo test` (affects both lib and cli) |
| Core trait changes | `cargo test` (full suite) |
| Error type modifications | `cargo test` (full suite - errors affect everything) |
| OXC integration patterns | `cargo test -p lib` |
| TypeScript handler types | `cd ts && pnpm test` (if tests exist) |

---

## Cross-Cutting Concerns

### Testing Strategy
- Unit tests: `#[cfg(test)] mod tests` blocks in each module
- Integration tests: `tests/` directory for CLI end-to-end workflows
- Property-based tests: proptest for AST analysis invariants
- Doc tests: Examples in rustdoc comments
- Benchmarks: criterion for performance-critical paths (AST parsing, symbol extraction)

### Error Handling
- anyhow/thiserror for library errors
- Informative error messages with context
- STDERR for status/progress, STDOUT for data output
- Graceful degradation for partial analysis results

### Performance Considerations
- OXC parser efficiency (avoid unnecessary allocations)
- Symbol table lookups (consider caching/indexing)
- File watching optimizations (debouncing, incremental analysis)
- Output formatting efficiency (streaming vs buffering)

### CLI Design Principles
- Consistent command structure across all commands
- Three output formats: console (ANSI colors), JSON, HTML
- JSON output includes both structured data and formatted representations
- HTML output includes metadata in `data-*` attributes
- Filter support for all commands (glob patterns)

### OXC Integration
- AST traversal patterns (visitors vs iterators)
- Semantic analysis integration
- Type system integration
- Position/offset tracking for error reporting

### TypeScript Handler Integration
- Minimal TypeScript in `/ts` for handler type definitions
- Bun runtime for executing handlers
- Event type definitions for watch handlers
- Handler discovery and validation

## Parallelization Opportunities

[Phases that can be executed in parallel]

| Parallel Group | Phases | Reason |
|----------------|--------|--------|
| Group A | Phase 1, Phase 2 | No dependencies |
| Group B | Phase 3 | Depends on Group A |

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| [Risk 1] | High/Med/Low | [Mitigation strategy] |

## Open Questions

- [ ] [Question 1]
- [ ] [Question 2]
```

### 2.2 Assign Principal Owners

For each phase and requirement, assign a principal owner based on:

| Content Type | Primary Owner | Secondary |
|--------------|---------------|-----------|
| Core library code (AST analysis, symbol tracking) | Rust Developer | - |
| CLI implementation (commands, output formatting) | Rust Developer | - |
| OXC integration (parser, semantic analysis) | Rust Developer | - |
| Testing strategy, TDD, property-based tests | Feature Tester (Rust) | Rust Developer |
| Performance optimization, benchmarking | Rust Developer | - |
| TypeScript handler types | Rust Developer | - |

---

## Step 3: Parallel Sub-Agent Reviews

**CRITICAL:** Launch ALL reviews in PARALLEL using multiple Task tool calls in a single message.

### 3.1 Review Prompts

For each sub-agent with assigned ownership, create a review task:

**Rust Developer Review:**

**Before launching, communicate to user via STDOUT:**
```text
🔧 Rust Developer Review - Activating skills: [skill-list-from-step-0]
```

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Rust review of [plan-name]",
    model: "claude-sonnet-4-5-20250929",
    run_in_background: true,
    prompt: `You are the Rust Developer sub-agent reviewing a plan for the TA (TypeScript Analyzer) project.

## MANDATORY: Activate Requested Skills FIRST

**User-requested skills to activate:**
[INSERT SKILLS FROM STEP 0 HERE - e.g., "rust-testing, thiserror, clap"]

**How to activate:**
Before proceeding with the review, explicitly activate each skill:
- For each skill in the list above, search for and read the skill file
- Skills are located in \`.claude/skills/[skill-name]/SKILL.md\` or \`~/.claude/skills/[skill-name]/SKILL.md\`
- Read the SKILL.md file to load the expertise
- Apply the skill's guidance throughout your review

**After activating skills, output to STDOUT:**
\`\`\`
✅ Activated skills: [list of skills successfully loaded]
\`\`\`

## Context
Read your expertise guidelines in: .claude/agents/rust-developer.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review ALL sections where Rust Developer is assigned as owner, plus:

1. **Architecture**
   - Is the module structure appropriate for a Cargo workspace?
   - Are trait boundaries well-defined?
   - Is the error type design sound (anyhow/thiserror)?
   - Are OXC integration patterns correct?
   - Is separation between lib/cli/ts modules clear?

2. **Performance**
   - Are allocation patterns optimized (avoid cloning AST nodes)?
   - Are hot paths identified (AST traversal, symbol extraction)?
   - Is async/await used appropriately (file watching)?
   - Are data structures efficient (symbol tables, dependency graphs)?

3. **Safety**
   - Are ownership patterns correct?
   - Is unsafe code minimized and justified?
   - Are lifetimes handled properly (especially for OXC AST references)?
   - Are file path operations safe?

4. **Testing**
   - Are unit tests planned in \`#[cfg(test)] mod tests\` blocks?
   - Are integration tests planned for CLI commands?
   - Is property-based testing (proptest) considered for AST invariants?
   - Are doc tests planned for public APIs?
   - Is TDD workflow incorporated?

5. **CLI Design**
   - Is command structure consistent?
   - Are three output formats (console, JSON, HTML) properly supported?
   - Is STDOUT/STDERR usage correct (data vs status)?
   - Are filter patterns well-designed?
   - Are error messages informative?

6. **OXC Integration**
   - Are AST traversal patterns efficient?
   - Is semantic analysis integration correct?
   - Is position/offset tracking accurate for error reporting?
   - Are type error extraction patterns sound?

7. **TypeScript Handler Integration**
   - Are TypeScript type definitions sufficient in /ts?
   - Is Bun runtime integration correct?
   - Are watch event types well-defined?
   - Is handler discovery/validation robust?

## Output Format
Return your review as:

### Rust Developer Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Parallelization Notes:**
- [which Rust phases can run in parallel]
- [dependencies to be aware of]

**Missing Considerations:**
- [anything overlooked]`
})
```

**Feature Tester Review (Rust):**

**Before launching, communicate to user via STDOUT:**
```text
🧪 Feature Tester (Rust) Review - Activating skills: [skill-list-from-step-0]
```

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Rust testing strategy review of [plan-name]",
    model: "claude-sonnet-4-5-20250929",
    run_in_background: true,
    prompt: `You are the Feature Tester (Rust) sub-agent reviewing a plan for the TA (TypeScript Analyzer) project.

## MANDATORY: Activate Requested Skills FIRST

**User-requested skills to activate:**
[INSERT SKILLS FROM STEP 0 HERE - e.g., "rust-testing, proptest"]

**How to activate:**
Before proceeding with the review, explicitly activate each skill:
- For each skill in the list above, search for and read the skill file
- Skills are located in \`.claude/skills/[skill-name]/SKILL.md\` or \`~/.claude/skills/[skill-name]/SKILL.md\`
- Read the SKILL.md file to load the expertise
- Apply the skill's guidance throughout your review

**After activating skills, output to STDOUT:**
\`\`\`
✅ Activated skills: [list of skills successfully loaded]
\`\`\`

## Context
Read your expertise guidelines in: .claude/agents/feature-tester-rust.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review the Rust testing strategy and ensure comprehensive test coverage:

1. **Test Strategy Completeness**
   - Are unit tests planned in \`#[cfg(test)] mod tests\` blocks?
   - Are integration tests planned for CLI commands?
   - Is TDD workflow incorporated appropriately?
   - Are doc tests planned for public APIs?

2. **Test Coverage**
   - Are happy paths covered?
   - Are edge cases and error conditions addressed?
   - Are property-based tests (proptest) planned for:
     - AST analysis invariants
     - Symbol table consistency
     - Dependency graph properties
   - Are snapshot tests considered for output formatting?

3. **Test Organization**
   - Are unit tests in same file as implementation?
   - Are integration tests testing CLI commands end-to-end?
   - Are test utilities organized (common test helpers)?
   - Is test data organized (fixture TypeScript files)?

4. **Acceptance Criteria Testability**
   - Can each acceptance criterion be verified by a test?
   - Are there missing criteria that should be added?
   - Are all CLI commands testable?
   - Are all output formats (console, JSON, HTML) tested?

5. **Testing Dependencies**
   - Are external dependencies properly abstracted for testing?
   - Are file system operations mockable?
   - Is OXC integration testable in isolation?

6. **Rust-Specific Testing**
   - Is cargo-nextest considered for better output?
   - Are benchmarks (criterion) planned for:
     - AST parsing
     - Symbol extraction
     - Dependency analysis
     - Output formatting
   - Are compilation tests planned for type constraints?

7. **TA-Specific Testing**
   - Are type error detection tests comprehensive?
   - Is symbol extraction tested with various TypeScript constructs?
   - Are dependency analysis tests covering all scopes (local, repo, module, external)?
   - Is file watching tested with various event types?
   - Are handler execution tests included?

## Output Format
Return your review as:

### Feature Tester (Rust) Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Test Scenarios to Add:**
- [missing test scenario 1]
- [missing test scenario 2]

**Missing Considerations:**
- [anything overlooked]`
})
```

### 3.2 Launch Reviews in Parallel

**IMPORTANT:** Send ALL relevant Task calls in a SINGLE message to run them in parallel.

Only invoke sub-agents that have assigned ownership in the plan.

**Before launching, communicate the complete skill strategy to user via STDOUT:**

```text
🚀 Launching parallel reviews with skill configuration:

**Rust Developer Review:**
  Skills: [skill-list-from-step-0]

**Feature Tester (Rust) Review:**
  Skills: [skill-list-from-step-0]

All sub-agents will activate these skills before beginning their reviews.
```

Example parallel invocation:

```typescript
// All in ONE message for parallel execution
Task({ /* Rust Developer review */ run_in_background: true })
Task({ /* Feature Tester (Rust) review */ run_in_background: true })
```

### 3.3 Collect Review Results

Use TaskOutput to collect results from all background tasks:

```typescript
TaskOutput({ task_id: "rust-review-id", block: true })
TaskOutput({ task_id: "tester-review-id", block: true })
```

---

## Step 4: Consolidation and Optimization

After all reviews complete, perform a final consolidation pass:

### 4.1 Synthesize Feedback

1. **Aggregate Concerns:** Group similar concerns across reviews
2. **Resolve Conflicts:** If reviewers disagree, determine the best path
3. **Prioritize Changes:** Order suggested changes by impact

### 4.2 Update the Plan

Incorporate review feedback into the plan:

1. Update requirement assignments if suggested
2. Modify phase details based on concerns
3. Add missing considerations identified by reviewers
4. Update acceptance criteria
5. Add project-specific considerations:
   - OXC integration patterns
   - CLI output formatting
   - File watching optimizations
   - TypeScript handler execution

### 4.3 Finalize Parallelization Analysis

Based on all reviews, create the final parallelization strategy:

```markdown
## Implementation Parallelization Strategy

### Parallel Execution Groups

| Group | Phases | Can Start After | Assignees |
|-------|--------|-----------------|-----------|
| A | 1, 2 | Plan approval | Rust Dev |
| B | 3 | Group A complete | Rust Dev, Tester |

### Parallelization Diagram

```text
Timeline:
─────────────────────────────────────────────────────►

Group A: ████████████ (Phase 1 + Phase 2 in parallel)
                     │
Group B:             └──████████████ (Phase 3)
```

### Synchronization Points

1. **After Group A:** Core traits and types must be finalized
2. **Final:** Integration testing across lib/cli modules

### 4.4 Update Plan Status

Change the plan status and add the review summary:

```markdown
**Status:** Reviewed - Ready for Implementation

## Review Summary

**Reviews Completed:** [Date]

**Reviewers:**
- Rust Developer: [Approve/Approve with Changes/Request Revision]
- Feature Tester (Rust): [Approve/Approve with Changes/Request Revision]

**Key Changes from Review:**
1. [Change 1]
2. [Change 2]

**Resolved Concerns:**
- [Concern] → [Resolution]
```

---

## Step 5: Present to User

### 5.1 Summary Report

Present the final plan to the user with:

1. **Executive Summary** - What will be built
2. **Phase Overview** - High-level view of all phases
3. **Owner Assignments** - Who owns what
4. **Parallelization Strategy** - How to maximize efficiency
5. **Key Risks** - Top risks and mitigations
6. **Open Questions** - Items needing user input

### 5.2 Request Approval

Ask the user to:

1. Review the plan at `.ai/plans/YYYY-MM-DD.plan-name.md`
2. Answer any open questions
3. Approve or request changes

---

## Output Artifacts

This command produces:

| Artifact | Location | Purpose |
|----------|----------|---------|
| Plan Document | `.ai/plans/YYYY-MM-DD.plan-name.md` | Complete implementation plan |
| Review Log | Embedded in plan | Sub-agent feedback |

---

## Example Workflow

```text
User: Create a plan for adding incremental analysis to the watch command

Main Thread:
├── Step 1: Gather requirements
│   ├── Ask clarifying questions
│   ├── Document FR and NFR
│   └── Explore codebase (watch implementation, analysis modules)
│
├── Step 2: Create initial plan
│   ├── Draft plan with phases
│   ├── Assign principal owners:
│   │   ├── Rust Developer: Incremental analysis engine, file change tracking
│   │   └── Feature Tester: Test strategy for incremental correctness
│   └── Save to .ai/plans/
│
├── Step 3: Parallel reviews (ALL AT ONCE)
│   ├── Rust Developer ──────┐
│   └── Feature Tester (Rust) ┘── Running in parallel
│
├── Step 4: Consolidation
│   ├── Synthesize feedback
│   ├── Update plan
│   ├── Finalize parallelization:
│   │   ├── Group A: Core incremental engine (single phase)
│   │   └── Group B: CLI integration + tests (after Group A)
│   └── Mark as reviewed
│
└── Step 5: Present to user
    └── Request approval
```

---

## Tips for Success

1. **Be thorough in Step 1** - Good requirements lead to good plans
2. **Assign owners carefully** - Match expertise to tasks
3. **Always run reviews in parallel** - This is the key efficiency gain
4. **Don't skip consolidation** - Cross-cutting concerns emerge in review
5. **Document parallelization clearly** - Implementation teams need this
6. **Keep the plan living** - Update as implementation reveals new information
7. **Consider TA specifics:**
   - OXC AST integration patterns
   - CLI command consistency
   - Output format handling (console, JSON, HTML)
   - File watching performance
   - TypeScript handler execution

---

## Next Steps After Planning

Once the plan is approved:

1. **For TDD workflow:** Use `/execute-phase` to implement each phase
2. **For parallel implementation:** Coordinate sub-agents based on parallelization groups
3. **Run tests:** `cargo test` or `cargo nextest run`
4. **Check benchmarks:** `cargo bench` for performance-critical changes
5. **Verify CLI:** Test all commands with various filters and output formats

---

## ⚠️ CRITICAL: Plan Execution Warning

**If you use `/execute-plan` to implement this plan:**

The `/execute-plan` command uses orchestrator agents to coordinate implementation. **These orchestrators have historically failed by writing completion reports WITHOUT actually creating code.**

**To prevent this failure, your plan MUST:**

1. **Be EXTREMELY specific about files to create:**
   - List EXACT file paths for every file
   - Don't say "create analysis module" - say "create lib/src/analysis/mod.rs, lib/src/analysis/symbols.rs, lib/src/analysis/types.rs"

2. **Include verification steps in acceptance criteria:**
   - ✅ GOOD: "File `lib/src/analysis/symbols.rs` exists with `SymbolExtractor` struct"
   - ❌ BAD: "Symbol extraction implemented"

3. **Specify test count expectations:**
   - ✅ GOOD: "Add 8 unit tests for SymbolExtractor in #[cfg(test)] mod"
   - ❌ BAD: "Tests cover symbol extraction"

4. **List dependencies explicitly:**
   - ✅ GOOD: "Add `oxc = \"0.10\"` to lib/Cargo.toml dependencies"
   - ❌ BAD: "Add OXC dependency"

**Why This Matters:**

Orchestrators will verify implementation by:
- Running `ls` on expected files
- Running `grep` on Cargo.toml for dependencies
- Running `cargo test` within the blast radius
- Checking line counts with `wc -l`

If your plan is vague, orchestrators may SIMULATE completion instead of VERIFYING it.

**Example of a GOOD Phase:**

```markdown
### Phase 1: Symbol Extraction Core

**Files to create:**
- `lib/src/analysis/mod.rs` - Module entry point
- `lib/src/analysis/symbols.rs` - SymbolExtractor struct and implementation

**Dependencies to add:**
- None (uses existing oxc dependencies)

**Acceptance Criteria:**
- [ ] File `lib/src/analysis/mod.rs` exists
- [ ] File `lib/src/analysis/symbols.rs` exists with >300 lines
- [ ] `grep "pub struct SymbolExtractor" lib/src/analysis/symbols.rs` succeeds
- [ ] `cargo test -p lib analysis::symbols` runs 8+ tests
- [ ] All new tests pass
```

**Example of a BAD Phase:**

```markdown
### Phase 1: Symbol Extraction Core

**Deliverables:**
- Symbol extraction logic

**Acceptance Criteria:**
- [ ] Symbol extraction is implemented
```

The BAD example gives orchestrators no way to verify success, so they'll just write a report claiming success.

---
