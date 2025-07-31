# Feature Taskpad: Autodebugger Agent Dashboard

## Feature Description
Transform Autodebugger into a cybernetic monitoring and coordination dashboard for LLM agents working on parallel development tasks. This tool provides real-time visibility into agent activities, enables intervention when needed, and serves as a local-first CI/CD system specifically designed for LLM-driven development workflows.

## Core Vision
- **Primary Users**: LLM agents (with human oversight)
- **Primary Function**: Monitor git worktrees and agent activities in real-time
- **Control Paradigm**: Cybernetic feedback loops with intervention capabilities
- **CI/CD Integration**: Local-first checks without external dependencies
- **Output Format**: Structured JSON/CLI optimized for LLM consumption

## Specifications
- Monitor multiple git worktrees simultaneously
- Track file changes, git diffs, and task progress
- Provide structured context aggregation from CLAUDE.local.md files
- Enable control commands (pause, reorient, backpedal)
- Calculate merge safety scores and predict conflicts
- Run local CI/CD checks without configuration files
- Support both CLI and future TUI interfaces

## Architecture Overview

```
autodebugger/
├── src/
│   ├── monitor/          # Worktree monitoring
│   │   ├── mod.rs       # Monitor trait and core logic
│   │   ├── worktree.rs  # Git worktree tracking
│   │   └── diff.rs      # Diff analysis and summarization
│   ├── ci/              # Local CI/CD functionality
│   │   ├── mod.rs       # CI framework
│   │   ├── checks.rs    # Individual check implementations
│   │   └── scoring.rs   # Safety score calculation
│   ├── analyze/         # Merge and conflict analysis
│   │   ├── mod.rs       # Analysis framework
│   │   ├── conflicts.rs # Conflict prediction
│   │   └── merge.rs     # Merge order calculation
│   ├── control/         # Agent control commands
│   │   ├── mod.rs       # Control framework
│   │   └── commands.rs  # Pause, reorient, backpedal
│   ├── parser/          # Context parsing
│   │   └── markdown.rs  # Task list parsing
│   └── main.rs          # CLI entry point
```

## Development Plan

### Phase 1: Enhanced Monitoring & Analysis (Current)

#### 1.1 Worktree Monitoring
- [x] Basic worktree status tracking
- [x] Git diff collection
- [x] JSON output format
- [ ] File system watcher for real-time updates
- [ ] Change rate tracking (commits/hour)

#### 1.2 Diff Analysis Features
- [x] Basic diff display
- [ ] Implement `--summary` mode
  ```bash
  autodebugger diff --summary
  # Output: Added: 10 files, Modified: 5 files, Deleted: 20 files
  ```
- [ ] Add `--categorize` flag to group changes
- [ ] Detect file overlap between worktrees
- [ ] Semantic diff analysis (function-level changes)

#### 1.3 Context Aggregation
- [x] Aggregate CLAUDE.local.md files
- [ ] Parse markdown task lists for completion tracking
- [ ] Generate progress reports per agent
- [ ] Track agent intent drift

### Phase 2: Merge Analysis Integration

See `feature_taskpad_merge_analysis.md` for comprehensive merge analysis features including:
- Pre-merge CI/CD checks (cargo check/test/clippy)
- Conflict detection and analysis
- Merge order recommendations
- LLM-friendly reporting

The merge analysis module extends the monitoring capabilities to provide detailed context for intelligent merge decisions.

### Phase 3: Control Commands

#### 3.1 Agent Control
- [ ] **Pause** - Halt agent activity
  ```bash
  autodebugger control pause <worktree-name>
  ```
- [ ] **Reorient** - Send guidance without stopping
  ```bash
  autodebugger control reorient <worktree-name> "message"
  ```
- [ ] **Backpedal** - Undo recent changes
  ```bash
  autodebugger control backpedal <worktree-name> --preserve-tests
  ```
- [ ] **Restore** - Reset to clean state
- [ ] **Terminate** - End agent instance

#### 3.2 Control Implementation
- [ ] File-based signaling (.pause, .reorient files)
- [ ] CLAUDE.local.md modification for guidance
- [ ] Git operations for backpedal/restore
- [ ] State persistence for recovery

### Phase 4: Task Analysis

#### 4.1 Task Completeness
- [ ] Implement `check-completeness` command
  ```bash
  autodebugger check-completeness <worktree> --taskpad <file>
  ```
- [ ] Parse [ ] vs [x] checkboxes
- [ ] Section-by-section analysis
- [ ] Progress visualization

### Phase 5: Future TUI Dashboard

#### 5.1 Terminal Interface
```
┌─────────────────────────────────────────────┐
│            Autodebugger TUI                 │
├─────────────────┬───────────────────────────┤
│ Worktree Status │ Active Agent Terminal     │
│ ┌─────────────┐ │                           │
│ │skeleton: 🟢  │ │ $ cargo build            │
│ │aichat:  🟡  │ │ Compiling cymbiont...    │
│ │logseq:  🟢  │ │                           │
│ │tui:     🔴  │ │                           │
│ └─────────────┘ │                           │
├─────────────────┴───────────────────────────┤
│ Control: [P]ause [R]eorient [B]ackpedal     │
└─────────────────────────────────────────────┘
```

#### 5.2 Features
- [ ] Real-time agent activity monitoring
- [ ] Quality scores and drift detection
- [ ] Inter-agent communication logs
- [ ] Terminal multiplexing
- [ ] MCP server integration

## Usage Examples

### Current CLI Usage
```bash
# Monitor all worktrees
autodebugger monitor /home/brandt/projects/cymbiont-workspace

# Get JSON status for LLM consumption
autodebugger status --path /path/to/workspace --json

# Check specific worktree changes
autodebugger diff skeleton-analysis --path /path/to/workspace

# Aggregate agent tasks
autodebugger context local-tasks --path /path/to/workspace
```

### Merge Analysis Usage
See `feature_taskpad_merge_analysis.md` for merge-related commands.

## Implementation Notes

### LLM-Friendly Design
1. Always provide structured output (JSON/YAML)
2. Use clear, predictable command patterns
3. Return actionable context, not raw data
4. Support piping and composition

### Monitoring Without Intrusion
1. Use filesystem watching for git changes
2. Parse git diffs for meaningful insights
3. Track file modifications without blocking
4. Aggregate changes into coherent narratives

### Local-First Advantages
1. **No Configuration Files** - No .github/workflows or .gitlab-ci.yml
2. **Immediate Feedback** - No waiting for runners or queuing
3. **LLM-Friendly Output** - Structured information for decision making
4. **Integrated Monitoring** - Unified view of development state

## Testing Strategy
- [ ] Create test worktrees with known conflicts
- [ ] Test merge analysis accuracy
- [ ] Benchmark performance on large repositories
- [ ] Integration tests with git operations
- [ ] LLM output format validation

## Future Extensions
1. **Git Hooks Integration** - Auto-run checks on commit/merge
2. **Custom Check Plugins** - Extensible check system
3. **Remote Runner Support** - Export to traditional CI/CD when needed
4. **WebSocket Server** - Real-time updates for web dashboards
5. **MCP Server** - Direct integration with Claude Desktop

## Success Metrics
- Agent activity visible within 2 seconds
- All relevant context aggregated in one place
- Pre-merge checks complete in <10 seconds
- Zero configuration required for new projects
- 100% local operation (no external dependencies)