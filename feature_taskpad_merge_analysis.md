# Feature Taskpad: Merge Analysis Tool

## Feature Description
A comprehensive merge analysis system for autodebugger that provides LLM-friendly visibility into parallel development worktrees. The tool performs pre-merge checks, analyzes potential conflicts, and presents structured information to enable intelligent merge decisions by LLM agents or human developers.

## Specifications
- **Pre-merge CI/CD checks**: cargo check/test/clippy, code quality analysis
- **Conflict detection**: Identify overlapping file changes and potential merge issues
- **Diff analysis**: Structured summaries of changes across worktrees
- **Merge ordering suggestions**: Based on dependency analysis and conflict minimization
- **LLM-friendly output**: Structured JSON responses with detailed context
- **Integration with existing monitor**: Extend current worktree monitoring capabilities
- **Local-first approach**: No external CI/CD dependencies, immediate feedback

## Relevant Components

### Monitor Module
- `src/monitor/mod.rs`: Core monitoring infrastructure with worktree status tracking
- `src/monitor/worktree.rs`: Worktree scanning and status extraction
- `src/monitor/diff.rs`: Git diff analysis (needs enhancement for conflict detection)
- Current usage: Basic worktree monitoring and diff display

### CI Module (Stub Implementation)
- `src/ci/mod.rs`: CI framework with safety scoring and recommendation system
- `src/ci/checks.rs`: Pre-merge check runner (stub implementation)
- `src/ci/conflicts.rs`: Conflict prediction analyzer (stub implementation)
- Current usage: Basic structure exists, needs full implementation

### CLI Interface
- `src/main.rs`: Command dispatch with clap-based argument parsing
- Current usage: Basic commands (status, monitor, diff, context)

## Development Plan

### 1. Enhanced Diff Analysis
- [ ] Implement structured diff comparison between worktrees
- [ ] Add file overlap detection logic
- [ ] Create diff summary with categorization (added/modified/deleted)
- [ ] Build conflict severity scoring algorithm
- [ ] Add `diff-compare` command to CLI

### 2. Pre-Merge CI/CD Checks
- [ ] Implement cargo check execution in worktree contexts
- [ ] Add cargo test runner with timeout handling
- [ ] Integrate cargo clippy with configurable lint levels
- [ ] Create debug macro scanner for code quality
- [ ] Add TODO comment auditing with tracking
- [ ] Implement documentation freshness checking
- [ ] Add `check` command with `--pre-merge` flag

### 3. Conflict Prediction Engine
- [ ] Build file overlap analyzer using git diff
- [ ] Implement line-range conflict detection
- [ ] Add semantic conflict detection (same functions modified)
- [ ] Create severity classification (high/medium/low)
- [ ] Develop resolution strategy suggestions
- [ ] Add `analyze-conflicts` command for pairwise analysis

### 4. Merge Analysis Report
- [ ] Aggregate all check results into structured report
- [ ] List all potential conflicts with context
- [ ] Provide detailed diff summaries
- [ ] Build JSON output format for LLM consumption
- [ ] Include relevant code snippets for conflict areas
- [ ] Add `merge-analysis` command

### 5. Merge Order Analysis
- [ ] Build dependency graph visualization
- [ ] Identify which changes depend on others
- [ ] Show potential conflict cascades
- [ ] Present multiple ordering options with trade-offs
- [ ] Generate detailed context for each ordering
- [ ] Add `analyze-merge-order` command

### 6. Comprehensive Reporting
- [ ] Integrate all analysis components
- [ ] Create unified report generation
- [ ] Add export formats (JSON, markdown, text)
- [ ] Build summary statistics
- [ ] Add trend analysis over time
- [ ] Add `merge-report` command

### 7. CLI Integration and Polish
- [ ] Update main.rs with all new commands
- [ ] Add comprehensive help text and examples
- [ ] Implement progress indicators for long operations
- [ ] Add verbose/quiet output modes
- [ ] Create command aliases and shortcuts
- [ ] Update CLI_USAGE.md documentation

## Development Notes

### Technical Decisions Made:
- Using existing monitor infrastructure as foundation
- Building on clap CLI framework for consistency
- Leveraging git commands directly rather than libgit2 for simplicity
- JSON output format for LLM integration
- Local-first approach avoiding external dependencies

### Architecture Choices:
- Extending existing `src/ci/` stub implementation
- Reusing worktree scanning from monitor module
- Conflict analysis as separate module for modularity
- Safety scoring as pure function for testability

### Critical Dependencies:
- Git must be available in PATH for diff analysis
- Cargo required for Rust project checks
- File system access to worktree directories
- JSON serialization for structured output

## Future Tasks
- Performance benchmarking with large repositories
- Custom check plugin system for extensibility
- Git hooks integration for automated checks
- Web dashboard for visual merge analysis
- Remote CI/CD export functionality
- Machine learning conflict prediction
- Integration with GitHub Actions for hybrid workflows

## Final Implementation
*To be completed when feature is finished*