# songline-math-wasm

Navigable knowledge graphs compiled to WebAssembly.

WASM bindings for graph structures inspired by Australian Aboriginal songlines — pathways of knowledge encoded as weighted, navigable graphs.

## Structures

- **Waypoint** — A node with an ID and weight
- **Verse** — A directed edge with traversal count
- **SonglineGraph** — The full graph container

## Navigation

- `pathfind(graph, start, end)` — High-weight Dijkstra pathfinding
- `navigability_score(graph)` — 0..1 connectivity score

## Corroboree (Analysis)

- `find_hubs(graph)` — High-connectivity nodes
- `modularity(graph)` — Community structure score

## Tradition (Evolution)

- `mutate(graph, add_probability)` — Probabilistic graph mutation
- `fitness(graph)` — Combined density + navigability score

## License

MIT
