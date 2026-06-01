#![deny(unsafe_code)]

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A node in the knowledge graph.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Waypoint {
    id: usize,
    weight: f64,
}

#[wasm_bindgen]
impl Waypoint {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, weight: f64) -> Self {
        Self { id, weight }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
}

/// A directed edge in the knowledge graph.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Verse {
    source: usize,
    target: usize,
    traversal_count: u32,
}

#[wasm_bindgen]
impl Verse {
    #[wasm_bindgen(constructor)]
    pub fn new(source: usize, target: usize, traversal_count: u32) -> Self {
        Self {
            source,
            target,
            traversal_count,
        }
    }

    pub fn source(&self) -> usize {
        self.source
    }

    pub fn target(&self) -> usize {
        self.target
    }

    pub fn traversal_count(&self) -> u32 {
        self.traversal_count
    }
}

// ---------------------------------------------------------------------------
// SonglineGraph
// ---------------------------------------------------------------------------

/// A navigable knowledge graph of waypoints and verses.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct SonglineGraph {
    waypoints: Vec<Waypoint>,
    verses: Vec<Verse>,
    adjacency: Vec<Vec<(usize, f64)>>, // target_index -> (neighbor_index, combined_weight)
}

#[wasm_bindgen]
impl SonglineGraph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            verses: Vec::new(),
            adjacency: Vec::new(),
        }
    }

    /// Add a waypoint. Returns its index.
    pub fn add_waypoint(&mut self, id: usize, weight: f64) -> usize {
        let idx = self.waypoints.len();
        self.waypoints.push(Waypoint::new(id, weight));
        self.adjacency.push(Vec::new());
        idx
    }

    /// Add a directed verse from waypoint at `source` to waypoint at `target`.
    pub fn add_verse(&mut self, source: usize, target: usize, weight: f64) {
        self.verses.push(Verse::new(source, target, 1));
        // Grow adjacency if needed
        while self.adjacency.len() <= source {
            self.adjacency.push(Vec::new());
        }
        while self.adjacency.len() <= target {
            self.adjacency.push(Vec::new());
        }
        self.adjacency[source].push((target, weight));
    }

    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }

    pub fn verse_count(&self) -> usize {
        self.verses.len()
    }

    /// Get waypoint weight by index.
    pub fn get_waypoint_weight(&self, index: usize) -> f64 {
        self.waypoints.get(index).map(|w| w.weight).unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

/// Dijkstra shortest-path using combined edge + target-node weights.
/// Higher-weight edges are preferred (we negate distances so weight acts as "benefit").
fn dijkstra_internal(
    graph: &SonglineGraph,
    start: usize,
    end: usize,
) -> Option<Vec<usize>> {
    use std::cmp::Ordering;

    let n = graph.adjacency.len();
    if start >= n || end >= n {
        return None;
    }

    // We want to maximise total weight → minimise negative weight.
    let mut dist = vec![f64::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    let mut visited = vec![false; n];

    dist[start] = 0.0;

    // Simple priority queue via sorted vec (fine for WASM, no alloc deps)
    // (negative accumulated distance, node)
    let mut heap: Vec<(f64, usize)> = vec![(0.0, start)];

    while let Some(pos) = heap.iter().enumerate().min_by(|a, b| {
        a.1 .0
            .partial_cmp(&b.1 .0)
            .unwrap_or(Ordering::Equal)
    }) {
        let (d, u) = heap.remove(pos.0);
        if visited[u] {
            continue;
        }
        visited[u] = true;

        if u == end {
            break;
        }

        for &(v, w) in &graph.adjacency[u] {
            if v >= n || visited[v] {
                continue;
            }
            // Use negative weight so higher weight = shorter distance
            let target_w = if v < graph.waypoints.len() {
                graph.waypoints[v].weight
            } else {
                0.0
            };
            let cost = -(w + target_w);
            let new_dist = d + cost;
            if new_dist < dist[v] {
                dist[v] = new_dist;
                prev[v] = Some(u);
                heap.push((new_dist, v));
            }
        }
    }

    if dist[end] == f64::INFINITY {
        return None;
    }

    // Reconstruct path
    let mut path = Vec::new();
    let mut cur = end;
    while let Some(p) = prev[cur] {
        path.push(cur);
        cur = p;
    }
    path.push(start);
    path.reverse();
    Some(path)
}

#[wasm_bindgen]
/// Find the highest-weight path from `start` to `end`.
pub fn pathfind(graph: &SonglineGraph, start: usize, end: usize) -> Vec<usize> {
    dijkstra_internal(graph, start, end).unwrap_or_default()
}

#[wasm_bindgen]
/// Returns a 0..1 score indicating how well-connected the graph is.
pub fn navigability_score(graph: &SonglineGraph) -> f64 {
    let n = graph.waypoint_count();
    if n <= 1 {
        return 1.0;
    }
    let max_pairs = n * (n - 1);
    let mut reachable = 0usize;
    for s in 0..n {
        for t in 0..n {
            if s != t && dijkstra_internal(graph, s, t).is_some() {
                reachable += 1;
            }
        }
    }
    reachable as f64 / max_pairs as f64
}

// ---------------------------------------------------------------------------
// Corroboree (graph analysis)
// ---------------------------------------------------------------------------

#[wasm_bindgen]
/// Find hub nodes — waypoints with degree >= median degree.
pub fn find_hubs(graph: &SonglineGraph) -> Vec<usize> {
    let n = graph.adjacency.len();
    if n == 0 {
        return Vec::new();
    }
    let mut degrees: Vec<(usize, usize)> = (0..n)
        .map(|i| {
            let out = graph.adjacency.get(i).map(|a| a.len()).unwrap_or(0);
            let in_deg = graph
                .adjacency
                .iter()
                .filter(|nbrs| nbrs.iter().any(|&(t, _)| t == i))
                .count();
            (i, out + in_deg)
        })
        .collect();

    degrees.sort_by_key(|&(_, d)| d);
    let median = degrees[n / 2].1;

    degrees
        .into_iter()
        .filter_map(|(i, d)| if d >= median && d > 0 { Some(i) } else { None })
        .collect()
}

#[wasm_bindgen]
/// Compute a simple modularity score (0..1) based on internal vs total edges.
pub fn modularity(graph: &SonglineGraph) -> f64 {
    let n = graph.adjacency.len();
    if n == 0 {
        return 0.0;
    }
    let total_edges: usize = graph.adjacency.iter().map(|a| a.len()).sum();
    if total_edges == 0 {
        return 0.0;
    }

    // Simple community split: first half vs second half
    let mid = n / 2;
    let mut internal = 0usize;
    for i in 0..n {
        let community_i = if i < mid { 0 } else { 1 };
        for &(j, _) in &graph.adjacency[i] {
            let community_j = if j < mid { 0 } else { 1 };
            if community_i == community_j {
                internal += 1;
            }
        }
    }
    internal as f64 / total_edges as f64
}

// ---------------------------------------------------------------------------
// Tradition (graph evolution)
// ---------------------------------------------------------------------------

#[wasm_bindgen]
/// Mutate a graph by potentially adding random verses.
pub fn mutate(graph: &SonglineGraph, add_probability: f64) -> SonglineGraph {
    let mut new = graph.clone();
    let n = new.waypoint_count();
    if n < 2 {
        return new;
    }

    // Simple pseudo-random using linear congruential
    let mut seed: u64 = 12345;
    for i in 0..n {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_val = (seed >> 33) as f64 / (1u64 << 31) as f64;
        if rand_val < add_probability {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let target = (seed as usize) % n;
            if target != i {
                let weight = 0.5 + rand_val;
                new.add_verse(i, target, weight);
            }
        }
    }
    new
}

#[wasm_bindgen]
/// Compute a fitness score (0..1) combining density and navigability.
pub fn fitness(graph: &SonglineGraph) -> f64 {
    let n = graph.waypoint_count();
    if n <= 1 {
        return 1.0;
    }
    let max_edges = n * (n - 1);
    let actual_edges: usize = graph.adjacency.iter().map(|a| a.len()).sum();
    let density = actual_edges as f64 / max_edges as f64;
    let nav = navigability_score(graph);
    // Weighted combination
    0.4 * density + 0.6 * nav
}

// ---------------------------------------------------------------------------
// Tests (30+)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Waypoint tests --

    #[test]
    fn test_waypoint_new() {
        let w = Waypoint::new(1, 3.5);
        assert_eq!(w.id(), 1);
        assert!((w.weight() - 3.5).abs() < 1e-9);
    }

    #[test]
    fn test_waypoint_zero_weight() {
        let w = Waypoint::new(0, 0.0);
        assert_eq!(w.weight(), 0.0);
    }

    #[test]
    fn test_waypoint_negative_weight() {
        let w = Waypoint::new(5, -1.0);
        assert_eq!(w.id(), 5);
        assert!(w.weight() < 0.0);
    }

    #[test]
    fn test_waypoint_large_weight() {
        let w = Waypoint::new(100, 1e15);
        assert!((w.weight() - 1e15).abs() < 1.0);
    }

    // -- Verse tests --

    #[test]
    fn test_verse_new() {
        let v = Verse::new(0, 1, 42);
        assert_eq!(v.source(), 0);
        assert_eq!(v.target(), 1);
        assert_eq!(v.traversal_count(), 42);
    }

    #[test]
    fn test_verse_zero_traversal() {
        let v = Verse::new(2, 3, 0);
        assert_eq!(v.traversal_count(), 0);
    }

    #[test]
    fn test_verse_self_loop() {
        let v = Verse::new(5, 5, 10);
        assert_eq!(v.source(), v.target());
    }

    // -- SonglineGraph tests --

    #[test]
    fn test_graph_new() {
        let g = SonglineGraph::new();
        assert_eq!(g.waypoint_count(), 0);
        assert_eq!(g.verse_count(), 0);
    }

    #[test]
    fn test_graph_add_waypoint() {
        let mut g = SonglineGraph::new();
        let idx = g.add_waypoint(1, 2.0);
        assert_eq!(idx, 0);
        assert_eq!(g.waypoint_count(), 1);
    }

    #[test]
    fn test_graph_add_multiple_waypoints() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(1, 1.0);
        g.add_waypoint(2, 2.0);
        g.add_waypoint(3, 3.0);
        assert_eq!(g.waypoint_count(), 3);
    }

    #[test]
    fn test_graph_add_verse() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 2.0);
        g.add_verse(0, 1, 1.5);
        assert_eq!(g.verse_count(), 1);
    }

    #[test]
    fn test_graph_get_waypoint_weight() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(10, 4.5);
        assert!((g.get_waypoint_weight(0) - 4.5).abs() < 1e-9);
    }

    #[test]
    fn test_graph_get_waypoint_weight_oob() {
        let g = SonglineGraph::new();
        assert_eq!(g.get_waypoint_weight(5), 0.0);
    }

    // -- Navigation / pathfind tests --

    #[test]
    fn test_pathfind_simple() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        g.add_verse(0, 1, 1.0);
        let path = pathfind(&g, 0, 1);
        assert_eq!(path, vec![0, 1]);
    }

    #[test]
    fn test_pathfind_no_path() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        let path = pathfind(&g, 0, 1);
        assert!(path.is_empty());
    }

    #[test]
    fn test_pathfind_three_nodes() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        g.add_waypoint(2, 1.0);
        g.add_verse(0, 1, 1.0);
        g.add_verse(1, 2, 1.0);
        let path = pathfind(&g, 0, 2);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn test_pathfind_prefers_high_weight() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0); // idx 0
        g.add_waypoint(1, 1.0); // idx 1
        g.add_waypoint(2, 1.0); // idx 2
        g.add_waypoint(3, 1.0); // idx 3
        g.add_verse(0, 3, 0.1); // direct but low weight
        g.add_verse(0, 1, 5.0); // via 1 → high weight
        g.add_verse(1, 2, 5.0);
        g.add_verse(2, 3, 5.0);
        let path = pathfind(&g, 0, 3);
        assert_eq!(path, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_pathfind_same_node() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        let path = pathfind(&g, 0, 0);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_pathfind_oob() {
        let g = SonglineGraph::new();
        let path = pathfind(&g, 0, 1);
        assert!(path.is_empty());
    }

    // -- Navigability tests --

    #[test]
    fn test_navigability_single_node() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        assert!((navigability_score(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_navigability_fully_connected() {
        let mut g = SonglineGraph::new();
        for i in 0..3 {
            g.add_waypoint(i, 1.0);
        }
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    g.add_verse(i, j, 1.0);
                }
            }
        }
        assert!((navigability_score(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_navigability_disconnected() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        assert!((navigability_score(&g) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_navigability_partial() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        g.add_waypoint(2, 1.0);
        g.add_verse(0, 1, 1.0);
        // 0→1 reachable, but 2 is isolated. 3 pairs, 1 reachable → 1/6 ≈ 0.1667
        let score = navigability_score(&g);
        assert!(score > 0.0 && score < 1.0);
    }

    // -- Corroboree / hubs tests --

    #[test]
    fn test_find_hubs_empty() {
        let g = SonglineGraph::new();
        assert!(find_hubs(&g).is_empty());
    }

    #[test]
    fn test_find_hubs_star() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0); // center
        g.add_waypoint(1, 1.0);
        g.add_waypoint(2, 1.0);
        g.add_verse(0, 1, 1.0);
        g.add_verse(0, 2, 1.0);
        g.add_verse(1, 0, 1.0);
        g.add_verse(2, 0, 1.0);
        let hubs = find_hubs(&g);
        assert!(hubs.contains(&0));
    }

    #[test]
    fn test_find_hubs_linear() {
        let mut g = SonglineGraph::new();
        for i in 0..4 {
            g.add_waypoint(i, 1.0);
        }
        g.add_verse(0, 1, 1.0);
        g.add_verse(1, 2, 1.0);
        g.add_verse(2, 3, 1.0);
        let hubs = find_hubs(&g);
        // Interior nodes (1,2) should be hubs
        assert!(hubs.contains(&1));
        assert!(hubs.contains(&2));
    }

    // -- Modularity tests --

    #[test]
    fn test_modularity_empty() {
        let g = SonglineGraph::new();
        assert!((modularity(&g) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_modularity_all_internal() {
        let mut g = SonglineGraph::new();
        for i in 0..4 {
            g.add_waypoint(i, 1.0);
        }
        // All edges within first half
        g.add_verse(0, 1, 1.0);
        g.add_verse(1, 0, 1.0);
        assert!((modularity(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_modularity_all_cross() {
        let mut g = SonglineGraph::new();
        for i in 0..4 {
            g.add_waypoint(i, 1.0);
        }
        // Edges cross communities: 0→2, 1→3
        g.add_verse(0, 2, 1.0);
        g.add_verse(1, 3, 1.0);
        assert!((modularity(&g) - 0.0).abs() < 1e-9);
    }

    // -- Tradition / mutate tests --

    #[test]
    fn test_mutate_zero_prob() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        let mutated = mutate(&g, 0.0);
        assert_eq!(mutated.verse_count(), g.verse_count());
    }

    #[test]
    fn test_mutate_high_prob() {
        let mut g = SonglineGraph::new();
        for i in 0..5 {
            g.add_waypoint(i, 1.0);
        }
        let mutated = mutate(&g, 1.0);
        assert!(mutated.verse_count() > 0);
    }

    #[test]
    fn test_mutate_preserves_waypoints() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        let mutated = mutate(&g, 0.5);
        assert_eq!(mutated.waypoint_count(), 2);
    }

    #[test]
    fn test_mutate_single_node() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        let mutated = mutate(&g, 1.0);
        assert_eq!(mutated.waypoint_count(), 1);
        assert_eq!(mutated.verse_count(), 0);
    }

    // -- Fitness tests --

    #[test]
    fn test_fitness_single_node() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        assert!((fitness(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_fitness_disconnected() {
        let mut g = SonglineGraph::new();
        g.add_waypoint(0, 1.0);
        g.add_waypoint(1, 1.0);
        let f = fitness(&g);
        assert!(f >= 0.0 && f < 0.5);
    }

    #[test]
    fn test_fitness_fully_connected() {
        let mut g = SonglineGraph::new();
        for i in 0..3 {
            g.add_waypoint(i, 1.0);
        }
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    g.add_verse(i, j, 1.0);
                }
            }
        }
        assert!((fitness(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_fitness_improves_with_edges() {
        let mut g1 = SonglineGraph::new();
        for i in 0..3 {
            g1.add_waypoint(i, 1.0);
        }
        g1.add_verse(0, 1, 1.0);

        let mut g2 = SonglineGraph::new();
        for i in 0..3 {
            g2.add_waypoint(i, 1.0);
        }
        g2.add_verse(0, 1, 1.0);
        g2.add_verse(1, 2, 1.0);
        g2.add_verse(2, 0, 1.0);

        assert!(fitness(&g2) > fitness(&g1));
    }
}
