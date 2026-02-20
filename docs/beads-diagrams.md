# Generating Project Plan Diagrams with Beads

This document explains how to generate Mermaid diagrams from beads issues to visualize project dependencies and execution order.

## Prerequisites

Install mermaid-cli for converting diagrams to images:

```bash
npm install -g @mermaid-js/mermaid-cli
```

## Basic Commands

### Show dependency tree for an issue

```bash
# Dependencies (what blocks this issue)
bd dep tree <issue-id> --format mermaid --direction down

# Dependents (what this issue blocks)
bd dep tree <issue-id> --format mermaid --direction up

# Full graph in both directions
bd dep tree <issue-id> --format mermaid --direction both
```

### Filtering options

```bash
# Only show open issues
bd dep tree <issue-id> --format mermaid --status open

# Limit depth
bd dep tree <issue-id> --format mermaid --max-depth 3
```

## Generate and View Diagrams

### One-liner to generate SVG and open in browser

```bash
bd dep tree <issue-id> --direction down --format mermaid | mmdc -i - -o /tmp/plan.svg -b white && open /tmp/plan.svg
```

### macOS with Chrome

```bash
bd dep tree <issue-id> --format mermaid --direction down > /tmp/diagram.mmd
mmdc -i /tmp/diagram.mmd -o /tmp/diagram.svg -b white
open -a "Google Chrome" /tmp/diagram.svg
```

### Save to project directory

```bash
bd dep tree <issue-id> --format mermaid --direction both > docs/project-plan.mmd
mmdc -i docs/project-plan.mmd -o docs/project-plan.svg -b white
mmdc -i docs/project-plan.mmd -o docs/project-plan.png -b white
```

## Output Format

The Mermaid output uses:
- `☐` for open issues
- `☑` for closed issues
- Arrows (`-->`) show dependency relationships (A --> B means A depends on B)

## Examples

### View the Parquet feature plan

```bash
# Show full dependency chain for the testing task (traces back to all prerequisites)
bd dep tree bytefreq-cpf.5 --format mermaid --direction down | mmdc -i - -o /tmp/parquet-plan.svg -b white && open /tmp/parquet-plan.svg

# Show parent feature and all children
bd dep tree bytefreq-cpf --format mermaid --direction both | mmdc -i - -o /tmp/parquet-feature.svg -b white && open /tmp/parquet-feature.svg
```

### View all blocked work

```bash
# List blocked issues first
bd blocked

# Then visualize a specific blocked issue's dependencies
bd dep tree <blocked-issue-id> --format mermaid --direction down
```

## Alternative Output Formats

Beads also supports other graph formats:

```bash
# Graphviz DOT format
bd dep tree <issue-id> --format dot

# Digraph format (for golang.org/x/tools/cmd/digraph)
bd list --format digraph
```

## Tips

1. Use `--direction down` to see what needs to be done before an issue
2. Use `--direction up` to see what will be unblocked when an issue completes
3. Start from the final deliverable task to see the full execution path
4. Use `--status open` to hide completed work and focus on remaining tasks
