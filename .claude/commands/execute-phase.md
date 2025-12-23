---
description: Execute a phase of a detailed plan from .ai/plans/
argument-hint: [phase-number]
---

# Execute Phase Workflow

You are now in **Phase Execution Mode**. Your goal is to execute a single phase from a plan following the appropriate workflow based on phase type.

## Prerequisites

Before starting, ensure:

1. A plan exists in `.ai/plans/` directory
2. You know which phase to execute
3. All previous phases are complete (if applicable)

---

## Step 0: Detect Requested Skills

**Purpose:** Identify which Claude Code skills the user wants to activate for this phase execution.

**Actions:**

1. **Check user request for skill mentions:**
   - Look for phrases like "use the [skill-name] skill"
   - Look for skill activation requests in user's message
   - Examples: "use rust-testing", "activate the clap skill", "with thiserror skill"

2. **Parse skill list:**
   - Extract skill names (e.g., `rust-testing`, `clap`, `thiserror`, `rust-logging`, `proptest`)
   - Create a comma-separated list for use during implementation

3. **Communicate to user via STDOUT:**
   ```text
   🎯 Executing phase with skills: [skill1, skill2, skill3]

   These skills will guide implementation and testing.
   ```

4. **If no skills requested:**
   - Use default skill set based on project type
   - For Rust projects: `rust-testing`, `thiserror`
   - Communicate: "🎯 Using default skills for Rust: rust-testing, thiserror"

5. **Store skills for activation:**
   - Keep the skill list available for activation before starting work
   - Skills will be activated in Step 1.5 (before exploration) or Step 2 (for design phases)

6. **Activate skills immediately:**
   ```text
   📚 Activating skills...
   ```

   For each skill in the list:
   - Search for `.claude/skills/[skill-name]/SKILL.md` or `~/.claude/skills/[skill-name]/SKILL.md`
   - Read the SKILL.md file to load the expertise
   - Apply the skill's guidance throughout the phase

   After activation:
   ```text
   ✅ Skills activated: [list of skills successfully loaded]
   ```

---

## Step 1: Identify the Phase

Ask the user:

1. **Which plan should we execute?**
   - List available plans in `.ai/plans/`
   - Or ask for the plan filename

2. **Which phase should we execute?**
   - Show available phases from the plan
   - Suggest the next incomplete phase
   - Confirm phase number with the user

3. **Read the plan file:**
   - Use Read tool to load `.ai/plans/[planName].md`
   - Extract the details for the specified phase
   - **Detect phase type:** Is this a design phase or implementation phase?
     - Design phases: Architecture, API design, schema design, planning, research
     - Implementation phases: Code, tests, features
   - **Extract the blast radius** for this phase (test scope pattern)
   - If blast radius is empty string `""`, tests will run against entire test suite

4. **Route to appropriate workflow:**
   - If DESIGN phase → Follow Design Phase Workflow (see below)
   - If IMPLEMENTATION phase → Follow Implementation Workflow (standard steps)

---

# Design Phase Workflow

**Use this workflow when the phase involves architecture, API design, schema design, planning, or research.**

## Step 1: Create Log File

**Purpose:** Document the design process and decisions.

**Actions:**

1. **Create log file:**
   - Path: `.ai/logs/YYYY-MM-DD.[planName]-phase[N]-log.md`
   - Create `.ai/logs/` directory if it doesn't exist

2. **Write log file with starting state:**

   ```markdown
   # Phase [N]: [Phase Name] (DESIGN PHASE)

   **Plan:** [planName]
   **Phase:** [N]
   **Started:** [Date and Time]
   **Phase Type:** Design

   ## Phase Overview

   [Copy phase overview from plan]

   ## Design Goals

   [List design goals from the plan]

   ## Repo Starting Position

   **Last local commit:** [git log -1 --format="%H"]
   **Last remote commit:** [git log origin/main -1 --format="%H" 2>/dev/null || echo "N/A"]
   **Branch:** [git branch --show-current]
   **Dirty files:** [git status --short || echo "None"]

   ## Design Work Log

   [This section will be updated as design work progresses]
   ```

3. **Save the log file**

## Step 2: Explore Existing Architecture

**Purpose:** Understand current state before making design decisions.

**Note:** At this point, skills from Step 0 have already been activated. Use their guidance for architecture exploration.

**Actions:**

1. **Identify relevant files:**
   - Existing types, structs, traits
   - Related documentation
   - Similar patterns in the codebase

2. **Read existing documentation:**
   - Architecture Decision Records (ADRs)
   - Design docs
   - API specifications

3. **Document findings in log:**

   ```markdown
   ### Existing Architecture Exploration

   **Files reviewed:**
   - `path/to/file` - [purpose]

   **Patterns discovered:**
   - [Pattern 1]
   - [Pattern 2]

   **Constraints identified:**
   - [Constraint 1]
   - [Constraint 2]
   ```

## Step 3: Complete Design Work

**Purpose:** Create design artifacts (documentation, ADRs, schemas, API specs).

**Note:** Use activated skills (from Step 0) to guide design decisions. Skills inform patterns and best practices.

**Communicate to user via STDOUT before design work:**
```text
📐 Creating design with guidance from skills: [skill-list-from-step-0]
```

**Actions:**

1. **Create design artifacts based on phase deliverables:**
   - Architecture Decision Records (ADRs)
   - API specifications
   - Schema definitions
   - Type definitions (Rust structs)
   - Diagrams (as markdown/mermaid)
   - Planning documents

2. **Follow project conventions:**
   - ADRs typically go in `docs/adr/` or `.ai/adr/`
   - Schemas in appropriate schema directories
   - Type definitions where they'll be used

3. **Update log file as you work:**

   ```markdown
   ### Design Artifacts Created

   **[Timestamp]** - Created ADR: [title]
   - Path: `docs/adr/NNNN-title.md`
   - Decision: [summary]

   **[Timestamp]** - Defined schema: [name]
   - Path: `path/to/schema.rs`
   - Key types: [list]

   **[Timestamp]** - Designed API: [name]
   - Endpoints/traits: [list]
   - Key decisions: [list]
   ```

## Step 4: Validate Design

**Purpose:** Ensure design is complete and coherent.

**Actions:**

1. **Review against acceptance criteria:**
   - Check each criterion from the plan
   - Mark as satisfied or note gaps

2. **Check for consistency:**
   - Do all design artifacts align?
   - Are there contradictions?
   - Are naming conventions consistent?

3. **Verify completeness:**
   - Are all deliverables from the plan created?
   - Are there open questions that need answering?

4. **Verify skeleton code compiles (if applicable):**
   ```bash
   cargo check
   ```
   Design phases may include stub code, but do NOT require tests to pass.

5. **Update log file:**

   ```markdown
   ### Design Validation

   **Acceptance Criteria Review:**
   - [x] Criterion 1 - Satisfied
   - [x] Criterion 2 - Satisfied
   - [ ] Criterion 3 - Gap identified: [description]

   **Consistency Check:**
   - All artifacts reviewed for alignment ✅
   - Naming conventions consistent ✅

   **Open Questions:**
   - [Question 1 and proposed answer]
   ```

## Step 5: Close Out Design Phase

**Purpose:** Document completion and prepare for implementation phases.

**Actions:**

1. **Update log file with completion:**

   ```markdown
   ## Phase Completion

   **Completed:** [Date and Time]
   **Duration:** [Time taken]
   **Phase Type:** Design

   ### Design Artifacts Delivered

   **Created:**
   - `path/to/artifact1` - [description]
   - `path/to/artifact2` - [description]

   **Modified:**
   - `path/to/existing-file` - [changes made]

   ### Design Decisions

   **Key decisions made:**
   1. [Decision 1 with rationale]
   2. [Decision 2 with rationale]

   ### Acceptance Criteria

   - [x] Criterion 1
   - [x] Criterion 2

   ### Notes for Implementation Phases

   - [Important context for developers]
   - [Constraints to be aware of]
   - [Suggested implementation order]
   ```

2. **Update plan status:**
   - Read the plan file
   - Mark this phase as complete
   - Update the plan's status section
   - Save the updated plan

3. **Report completion to user:**

   ```text
   ✅ Phase [N] Complete: [Phase Name] (DESIGN PHASE)

   **Design artifacts created:**
   - [List of artifacts with paths]

   **Key design decisions:**
   - [Summary of major decisions]

   **Next steps:**
   1. Review design artifacts in [paths]
   2. Run `/execute-phase [N+1]` to continue to next phase

   **Log file:** `.ai/logs/YYYY-MM-DD.[planName]-phase[N]-log.md`
   ```

## Design Phase Checklist

- [ ] Phase identified as DESIGN phase
- [ ] LOG created in `.ai/logs/`
- [ ] Starting position documented
- [ ] Existing architecture explored
- [ ] Design artifacts created (ADRs, schemas, APIs, etc.)
- [ ] Design validated against acceptance criteria
- [ ] Log file updated with completion
- [ ] Plan status updated
- [ ] User notified with summary

---

# Implementation Workflow

**Use this workflow when the phase involves writing code, tests, or features.**

## Step 1.5: EXPLORE EXISTING CODE - MANDATORY

**🚨 CRITICAL: Before doing ANYTHING else, understand what code already exists! 🚨**

**Purpose:** Prevent architectural misunderstandings by examining existing code structure BEFORE implementation.

**Note:** At this point, skills from Step 0 have already been activated. Use their guidance throughout exploration.

**Actions:**

1. **Identify files mentioned in the plan:**

   From the phase description, note ALL files that will be created or modified.

2. **Search for existing related files:**

   ```bash
   # Search for files with similar names
   find src -name "*[keyword]*" -type f

   # Or use Glob
   Glob: src/**/*[keyword]*.rs
   ```

   For example, if implementing "logging", search for:
   - Existing files with "log" in the name
   - Related service stubs
   - Similar utilities

3. **Read existing stubs/interfaces:**

   If files already exist:
   - **Read them COMPLETELY** using the Read tool
   - Understand the existing structure
   - Note TODOs or incomplete sections
   - Check if you're meant to COMPLETE existing code, not create new files

4. **Understand the architecture:**

   Before writing code, understand:
   - What patterns does the existing code use?
   - Are there type definitions you need to follow?
   - Are there existing traits or base structs?
   - How do similar features work in the codebase?

5. **Use Grep to find usage patterns:**

   ```bash
   # Find how existing code uses similar features
   Grep: "similar pattern"
   Grep: "use.*types"
   ```

6. **Document findings in log file:**

   ```markdown
   ### Existing Code Exploration

   **Files found:**
   - `lib/src/analysis/parser.rs` - EXISTS as stub (needs completion)
   - `lib/src/types.rs` - Defines core type patterns

   **Architecture notes:**
   - Analysis modules use OXC for AST parsing
   - Pattern: `pub fn analyze(source: &str) -> Result<Analysis, Error>`
   - Error handling uses thiserror crate

   **Decision:** Complete existing stub, don't create new module
   ```

7. **Validate plan against reality:**

   Ask yourself:
   - Does the plan match the existing code structure?
   - Am I creating something that already exists?
   - Am I understanding the architecture correctly?
   - Should I complete an existing stub instead of creating new files?

**If you discover a mismatch between the plan and existing code, STOP and inform the user before proceeding.**

**DO NOT SKIP THIS STEP.** Most architectural mistakes happen because this exploration was skipped.

---

## Step 2: SNAPSHOT - Capture Current Test State

**Purpose:** Establish a baseline so you can detect regressions and measure progress within the blast radius.

**Actions:**

1. **Run tests within the blast radius:**

   ```bash
   # If blast radius is empty string, run all tests:
   cargo test

   # If blast radius is a pattern, run scoped tests:
   cargo test [blast-radius]
   ```

2. **Create snapshot:**

   Create a simple representation of test results:

   ```xml
   <test-snapshot date="YYYY-MM-DD">
     <blast-radius>[pattern or "all"]</blast-radius>
     <suite name="unit-tests" total="X" passed="Y" failed="Z" />
     <suite name="integration-tests" total="X" passed="Y" failed="Z" />
     <starting-failures>
       <failure test="module::test_name" />
     </starting-failures>
   </test-snapshot>
   ```

3. **Document starting failures within blast radius** - these are your baseline, don't fix them yet

## Step 3: CREATE LOG - Document Starting Position

**Purpose:** Create a detailed record for debugging and tracking progress.

**Actions:**

1. **Create log file:**
   - Path: `.ai/logs/YYYY-MM-DD.[planName]-phase[N]-log.md`
   - Example: `.ai/logs/2025-12-20-symbol-analysis-phase1-log.md`
   - Create `.ai/logs/` directory if it doesn't exist

2. **Write log file with starting state:**

   ```markdown
   # Phase [N]: [Phase Name]

   **Plan:** [planName]
   **Phase:** [N]
   **Started:** [Date and Time]
   **Blast Radius:** [test scope pattern or "all"]

   ## Phase Overview

   [Copy phase overview from plan]

   ## Starting Test Position

       <test-snapshot date="YYYY-MM-DD">
         <blast-radius>[pattern or "all"]</blast-radius>
         <suite name="unit-tests" total="X" passed="Y" failed="Z" />
         <suite name="integration-tests" total="X" passed="Y" failed="Z" />
         <starting-failures>
           <failure test="module::test_name" />
         </starting-failures>
       </test-snapshot>

   ## Repo Starting Position

   **Last local commit:** [git log -1 --format="%H"]
   **Last remote commit:** [git log origin/main -1 --format="%H" 2>/dev/null || echo "N/A"]
   **Branch:** [git branch --show-current]
   **Dirty files:** [git status --short || echo "None"]

   ## Work Log

   [This section will be updated as work progresses]
   ```

3. **Save the log file**

## Step 4: WRITE TESTS - Create Tests FIRST

**Purpose:** Tests define the contract and expected behavior before any code is written.

**🚨 CRITICAL: This is TRUE Test-Driven Development - tests MUST be written BEFORE implementation! 🚨**

**Note:** Use activated skills (from Step 0) to guide test creation. If `rust-testing` is active, follow its patterns.

**Communicate to user via STDOUT before writing tests:**
```text
🧪 Writing tests with guidance from skills: [skill-list-from-step-0]
```

**Actions:**

1. **Review test requirements from plan:**

   - Happy path tests
   - Edge case tests
   - Error condition tests
   - Integration tests (if applicable)

2. **Create test files:**

   **For unit tests:**
   - Add `#[cfg(test)]` modules within source files
   - Use `#[test]` attribute for test functions
   - Use `use super::*;` to access private functions

   **For integration tests:**
   - Create files in `tests/` directory
   - Each file is a separate test crate
   - Test public API only

3. **Write comprehensive tests:**

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn it_returns_error_for_invalid_input() {
           let result = parse_source("invalid code");
           assert!(result.is_err());
       }

       #[test]
       fn it_parses_valid_typescript_correctly() {
           let result = parse_source("const x = 1;");
           assert!(result.is_ok());
           let ast = result.unwrap();
           assert_eq!(ast.statements.len(), 1);
       }

       #[test]
       #[should_panic(expected = "empty input")]
       fn it_panics_on_empty_input() {
           parse_source("");
       }
   }
   ```

4. **Verify tests FAIL initially:**

   ```bash
   cargo test
   # or
   cargo test [module_name]
   ```

   - Confirm tests fail (no implementation exists yet)
   - This verifies the tests are checking for real functionality, not trivially passing

5. **Think critically about test completeness:**

   - Review each test and ask: **If the functionality were built, would this test be meaningful?**
   - Consider all variants the function can express:
     - Different input types and combinations
     - Boundary conditions and edge cases
     - Error states and failure modes
     - Return value variations
   - **Think hardest here** - missing variants now means gaps in coverage later
   - Are you testing behavior, not just implementation details?
   - Would these tests catch regressions if someone changed the code?

6. **Update log file with test creation:**

   Add to "Work Log" section:

   ```markdown
   ### Tests Created

   - Unit tests in `lib/src/analysis/parser.rs`
     - `it_returns_error_for_invalid_input`
     - `it_parses_valid_typescript_correctly`
     - `it_panics_on_empty_input`

   **Initial test run:** All tests fail as expected (no implementation yet)
   ```

## Step 4.5: VALIDATE TESTS - Critical Checkpoint

**⚠️ MANDATORY: Before proceeding to implementation, validate your tests are correct**

**Purpose:** Catch testing pattern mistakes NOW, before they're baked into implementation. This checkpoint prevents hours of rework.

**Actions:**

1. **Validate test structure:**

   - Unit tests in `#[cfg(test)] mod tests` blocks within source files
   - Integration tests in `tests/` directory
   - `use super::*;` present for accessing private items
   - Descriptive test names: `fn it_returns_error_for_invalid_input()`

2. **Validate test patterns:**

   - Using `assert_eq!`, `assert_ne!`, `assert!` correctly
   - `#[should_panic]` for expected panics with expected message
   - Result return type for fallible tests: `fn test() -> Result<(), Box<dyn Error>>`

3. **Check for property tests (if applicable):**

   - Complex logic should have proptest invariants
   - Roundtrip tests for serialization

4. **Run the tests:**

   ```bash
   cargo test            # Standard runner
   cargo test --no-run   # Just compile, verify tests build
   ```

5. **Update log file with validation:**

   ```markdown
   ### Test Validation

   - Completed Rust testing checklist ✅
   - Unit tests in correct location ✅
   - Integration tests in tests/ directory ✅
   - Tests ready for implementation ✅
   ```

**🚨 DO NOT PROCEED TO IMPLEMENTATION IF ANY CHECKLIST ITEM FAILS 🚨**

Testing mistakes caught here save hours of debugging and rework later.

---

## Step 5: IMPLEMENTATION - Build to Pass Tests

**Purpose:** Let tests drive the implementation, ensuring you build exactly what's needed.

**Note:** Use activated skills (from Step 0) to guide implementation. Apply skill patterns throughout.

**Communicate to user via STDOUT before implementation:**
```text
⚙️  Implementing with guidance from skills: [skill-list-from-step-0]
```

**Actions:**

1. **Implement minimal code to pass each test:**
   - Start with one test or small group of related tests
   - Write the simplest code that makes tests pass
   - Don't over-engineer or add features not covered by tests

2. **Follow the plan's implementation details:**
   - Create files specified in the plan
   - Modify files specified in the plan
   - Implement key functions/structs as planned

3. **Iterate rapidly:**
   - Run tests frequently: `cargo test`
   - Fix failures immediately
   - Keep the feedback loop tight

4. **Continue until all phase tests pass:**
   - All tests must be green
   - No shortcuts - every test must pass

5. **Refactor with confidence:**
   - Once tests pass, improve code quality
   - Tests act as a safety net
   - Re-run tests after each refactor

6. **Update log file during implementation:**

   Add to "Work Log" section as you go:

   ```markdown
   ### Implementation Progress

   **[Timestamp]** - Created `lib/src/analysis/parser.rs`
   - Implemented `parse_source()`
   - Tests passing: X/Y

   **[Timestamp]** - Modified `lib/src/types.rs`
   - Added integration with new functionality
   - Tests passing: Y/Y ✅

   **[Timestamp]** - Refactored for better readability
   - All tests still passing ✅
   ```

## Step 6: CLOSE OUT - Verify and Document

**Purpose:** Ensure quality, prevent regressions, and properly document completion.

**Actions:**

1. **Run tests within blast radius:**

   ```bash
   # If blast radius is empty string, run all tests:
   cargo test

   # If blast radius is a pattern, run scoped tests:
   cargo test [blast-radius]
   ```

2. **Check for regressions within blast radius:**

   Compare ending test failures against starting failures:

   - **Capture ending failures:** Run tests and note all failures
   - **Compare against starting failures:** Identify NEW failures
   - **New regressions = ending failures - starting failures**

   If NEW regressions exist:

   - **STOP and think deeply** - understand WHY, not just the error message
   - Add a "Regressions Found" section to log file with test name, failure message, root cause analysis, and resolution
   - Determine root cause:
     - Is your implementation incorrect?
     - Does the existing test need updating? (only if requirements changed)
     - Is there a side effect you didn't anticipate?
   - Fix the root cause, not just the symptom
   - Re-run tests within blast radius to confirm fix

3. **Run quality checks:**

   ```bash
   cargo clippy -- -D warnings  # Linting
   cargo fmt --check            # Formatting
   cargo build                  # Compilation
   cargo test --doc             # Doc tests
   ```

4. **Build TypeScript handlers (if modified):**

   If you modified TypeScript handler types in `/ts`:
   ```bash
   cd ts && pnpm build
   ```

5. **Update log file with completion:**

   Add `## Phase Completion` section:

   ```markdown
   ## Phase Completion

   **Completed:** [Date and Time]
   **Duration:** [Time taken]
   **Blast Radius:** [test scope pattern or "all"]

   ### Final Test Results (within blast radius)

   - Unit tests: X/X passing ✅
   - Integration tests: Y/Y passing ✅
   - Doc tests: Z/Z passing ✅

   ### Regression Analysis

   **Starting failures:** [count] tests
   - [list from starting snapshot]

   **Ending failures:** [count] tests
   - [list from final run]

   **New regressions:** [None / list any new failures]

   ### Files Changed

   **Created:**
   - `path/to/new-file.rs`

   **Modified:**
   - `path/to/existing-file.rs`

   ### Quality Checks

   - Clippy: Pass (0 warnings)
   - Formatting: Pass
   - Doc tests: Pass
   - Build: Success
   ```

6. **Update plan status:**

   - Read the plan file
   - Mark this phase as complete
   - Update the plan's status section
   - Save the updated plan

7. **Report completion to user:**

   Provide a clear summary:

   ```text
   ✅ Phase [N] Complete: [Phase Name]

   **What was implemented:**
   - [Summary of implementation]

   **Test coverage added:**
   - [Number] tests written
   - All tests passing
   - No regressions

   **Quality checks:**
   - Clippy: Pass
   - Formatting: Pass
   - Build: Success

   **Next steps:**
   1. Review implementation
   2. Run `/execute-phase [N+1]` to continue to next phase

   **Log file:** `.ai/logs/YYYY-MM-DD.[planName]-phase[N]-log.md`
   ```

## Important Reminders

- **Skills activation** - Skills are activated in Step 0 and guide ALL subsequent steps
- **Detect phase type** - Design phases follow different workflow than implementation phases
- **Design phases** - Create design artifacts (ADRs, schemas, docs); no tests required
- **Implementation phases** - Follow TDD workflow with tests first
- **Tests FIRST** - Always write tests before implementation (implementation phases only)
- **Use activated skills** - Apply skill patterns in exploration, testing, implementation, and design
- **Log everything** - Keep the log file updated throughout
- **Understand failures** - Don't just fix symptoms, understand root causes
- **Blast radius testing** - Run tests within blast radius, not necessarily entire suite
- **Track regressions properly** - Compare ending failures against starting failures; only NEW failures are regressions
- **Rust: Test location** - Unit tests inline with `#[cfg(test)]`, integration tests in `tests/`
- **Rust: Property tests** - Use proptest for complex invariants and roundtrip testing
- **Quality checks** - Always run clippy, fmt, and build checks
- **TypeScript handlers** - If modifying `/ts`, run `pnpm build`

## Phase Execution Checklist

Use this checklist to ensure you don't miss any steps:

### Common Steps (All Phases)

- [ ] **Skills detected and activated** (Step 0)
- [ ] Skills communicated to user via STDOUT
- [ ] Plan and phase identified
- [ ] **Phase type detected** (DESIGN vs IMPLEMENTATION)
- [ ] LOG created in `.ai/logs/`
- [ ] Starting position documented

### Design Phase Checklist

- [ ] Existing architecture explored
- [ ] Design artifacts created (ADRs, schemas, API specs, etc.)
- [ ] Design validated against acceptance criteria
- [ ] Design consistency checked
- [ ] Skeleton code compiles (if applicable)
- [ ] Log file updated with completion
- [ ] Plan status updated
- [ ] User notified with summary

### Implementation Phase Checklist

- [ ] **Blast radius extracted from plan**
- [ ] SNAPSHOT captured (baseline test state within blast radius)
- [ ] **Starting failures documented**
- [ ] Tests written (unit and/or integration)
- [ ] Tests initially failed (proving validity)
- [ ] Implementation completed
- [ ] All tests passing
- [ ] **Blast radius tests run**
- [ ] **Ending failures documented**
- [ ] **No NEW regressions** (ending - starting = 0 new failures)
- [ ] Quality checks passed (clippy, fmt, build, doc tests)
- [ ] TypeScript handlers built (if modified)
- [ ] Log file updated with completion
- [ ] Plan status updated
- [ ] User notified with summary
