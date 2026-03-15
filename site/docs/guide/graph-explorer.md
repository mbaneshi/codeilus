# Graph Explorer

The Graph Explorer (`/explore/graph`) provides an interactive visualization of the codebase's knowledge graph.

## What You See

- **Nodes** represent symbols (functions, classes, structs, methods)
- **Edges** represent relationships (calls, imports, extends, implements)
- **Colors** indicate community membership (modules detected by Louvain)
- **Size** reflects connectivity (more connections = larger node)

## Interactions

- **Click a node** to see its details: callers, callees, file location, signature
- **Filter by community** to focus on a specific module
- **Filter by edge type** to see only calls, imports, or heritage relationships
- **Search** to find specific symbols

## Communities

Communities are groups of symbols that are more connected to each other than to the rest of the graph. They typically correspond to modules or functional areas of the codebase.

Each community has:

- A **label** derived from the directory structure
- A **cohesion score** (0.0 to 1.0) measuring internal connectivity
- A list of **member symbols**

## Entry Points

Entry points are symbols identified as good starting places for understanding the codebase. Scoring heuristics include:

- `main` functions
- Route handlers and API endpoints
- CLI entry points
- Symbols with zero callers but many callees

## Related Pages

- [Metrics Dashboard](/explore/metrics) &mdash; complexity heatmap and hotspots
- [File Tree](/explore/tree) &mdash; browsable file hierarchy
- [Diagrams](/explore/diagrams) &mdash; architecture and flowchart diagrams
