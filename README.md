# songline-math-wasm

> Australian Aboriginal songline navigation compiled to WebAssembly — graph pathfinding, hubs, and tradition evolution.

## What This Does

`songline-math-wasm` brings songline graph mathematics to the browser at near-native speed. Build knowledge graphs, pathfind with dreamtime fallback, find hubs and clusters, compute modularity, and evolve traditions. Use it for web-based knowledge graphs, semantic navigation, or interactive topological analysis.

## The Cultural Root

See `songline-math` (npm) for the full cultural background. Songlines encode navigation as songs through knowledge space.

## Install

```bash
npm install songline-math-wasm
```

## Quick Start

```typescript
import init, { SonglineGraph, pathfind, navigability_score, find_hubs, modularity, fitness } from "songline-math-wasm";

await init();

const graph = new SonglineGraph();
graph.add_waypoint(0, [0, 0]);
graph.add_waypoint(1, [1, 0]);
graph.add_waypoint(2, [2, 1]);
graph.add_verse(0, 1, 1.0);
graph.add_verse(1, 2, 1.5);

// Pathfinding
const path = pathfind(graph, 0, 2);
console.log(path);  // [0, 1, 2]

// Analysis
const nav = navigability_score(graph);
const hubs = find_hubs(graph);
const mod = modularity(graph);

// Evolution
const mutated = mutate(graph, 0.1);
const fit = fitness(graph);
```

## API Reference

### `SonglineGraph`
- `add_waypoint(id: number, coords: number[])`
- `add_verse(from: number, to: number, weight: number)`

### Functions
- `pathfind(graph, start, end) → number[]`
- `navigability_score(graph) → number`
- `find_hubs(graph) → number[]`
- `modularity(graph) → number`
- `mutate(graph, add_probability) → SonglineGraph`
- `fitness(graph) → number`

### Types
```typescript
interface Waypoint { id: number; coordinates: number[]; }
interface Verse { from: number; to: number; weight: number; }
```

## License

MIT
