# Graph Report - .  (2026-06-28)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 34 nodes · 33 edges · 10 communities (6 shown, 4 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7fabfbd3`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Reed-Solomon Algorithms|Reed-Solomon Algorithms]]
- [[_COMMUNITY_Rust Error Handling|Rust Error Handling]]
- [[_COMMUNITY_Library Defaults|Library Defaults]]
- [[_COMMUNITY_Reed-Solomon Encoding|Reed-Solomon Encoding]]
- [[_COMMUNITY_Crate Dependencies|Crate Dependencies]]
- [[_COMMUNITY_Galois Field (GF)|Galois Field (GF)]]
- [[_COMMUNITY_Reed-Solomon (255,223)|Reed-Solomon (255,223)]]

## God Nodes (most connected - your core abstractions)
1. `RsError` - 6 edges
2. `ReedSolomon` - 6 edges
3. `decode method` - 4 edges
4. `ReedSolomon struct` - 3 edges
5. `default_is_rs_255_223()` - 2 edges
6. `reedsolomon crate` - 1 edges
7. `cryptovault crate` - 1 edges
8. `encode method` - 1 edges
9. `RsError enum` - 1 edges
10. `Berlekamp-Massey algorithm` - 1 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Test Bar Requirements** — reedsolomon_crate, rs_255_223, gf2_8, encode_method, decode_method [EXTRACTED 0.95]
- **Quality Gates** — reedsolomon_crate, cryptovault_crate, src_lib_rs [INFERRED 0.75]

## Communities (10 total, 4 thin omitted)

### Community 0 - "Reed-Solomon Algorithms"
Cohesion: 0.29
Nodes (7): Berlekamp-Massey algorithm, Chien search algorithm, decode method, encode method, Forney algorithm, ReedSolomon struct, RsError enum

### Community 1 - "Rust Error Handling"
Cohesion: 0.33
Nodes (5): Display, Error, Formatter, Result, RsError

### Community 3 - "Reed-Solomon Encoding"
Cohesion: 0.50
Nodes (3): Default, ReedSolomon, Vec

## Knowledge Gaps
- **9 isolated node(s):** `reedsolomon crate`, `cryptovault crate`, `GF(2^8)`, `RS(255,223)`, `encode method` (+4 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `RsError` connect `Rust Error Handling` to `Library Defaults`, `Reed-Solomon Encoding`?**
  _High betweenness centrality (0.107) - this node is a cross-community bridge._
- **Why does `ReedSolomon` connect `Reed-Solomon Encoding` to `Rust Error Handling`, `Library Defaults`?**
  _High betweenness centrality (0.090) - this node is a cross-community bridge._
- **What connects `reedsolomon crate`, `cryptovault crate`, `GF(2^8)` to the rest of the system?**
  _9 weakly-connected nodes found - possible documentation gaps or missing edges._