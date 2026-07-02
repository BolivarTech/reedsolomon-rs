# Graph Report - reedsolomon  (2026-07-02)

## Corpus Check
- 49 files · ~29,060 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 271 nodes · 340 edges · 31 communities (20 shown, 11 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8ecc0223`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Crate Configuration|Crate Configuration]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]

## God Nodes (most connected - your core abstractions)
1. `RsError` - 16 edges
2. `ReedSolomon` - 13 edges
3. `encode_framed()` - 13 edges
4. `decode_block()` - 12 edges
5. `syndromes()` - 9 edges
6. `encode_blocks()` - 8 edges
7. `Task 1 Report — GF(2^8) field constants + EXP/LOG tables` - 8 edges
8. `Task 2 Report — GF(2^8) Arithmetic Operations` - 8 edges
9. `Task 5 Report: Systematic Encoder (`encode.rs`)` - 8 edges
10. `berlekamp_massey()` - 7 edges

## Surprising Connections (you probably didn't know these)
- `encode_framed()` --calls--> `crc32()`  [INFERRED]
  src/frame.rs → src/crc.rs
- `decode_block()` --references--> `RsError`  [EXTRACTED]
  src/decode.rs → src/lib.rs
- `decode_blocks()` --references--> `RsError`  [EXTRACTED]
  src/decode.rs → src/lib.rs
- `encode_blocks()` --references--> `RsError`  [EXTRACTED]
  src/encode.rs → src/lib.rs
- `decode_framed()` --references--> `RsError`  [EXTRACTED]
  src/frame.rs → src/lib.rs

## Import Cycles
- None detected.

## Communities (31 total, 11 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.15
Nodes (13): Default, Display, Error, Formatter, Result, Self, default_fails_loud_on_17_errors(), default_is_rs_255_223() (+5 more)

### Community 1 - "Community 1"
Cohesion: 0.09
Nodes (21): §0.1 Evidence (final state), C1 — `#![allow(dead_code)]` retained (expected, documented), C2 — TDD-Guard state synchronization (tdd-guard-rust --passthrough bug), C3 — Test structure differs from brief's 4-function spec, C4 — `inv(0)` without assert returns `EXP[255] = 1` (not caught without the guard), Commit SHAs, Concerns, Files Modified (+13 more)

### Community 2 - "Community 2"
Cohesion: 0.27
Nodes (7): Option, build_generator(), encode_blocks(), encode_is_systematic_and_right_length(), encode_multiblock_length(), encoded_len(), generator_has_expected_shape_and_roots()

### Community 3 - "Crate Configuration"
Cohesion: 0.33
Nodes (5): Design invariant, Example, License, reedsolomon, Why a dedicated crate

### Community 4 - "Community 4"
Cohesion: 0.11
Nodes (8): add(), div(), div_by_zero_panics(), inv(), inv_of_zero_panics(), mul(), mul_distributive_over_add(), pow_edge_cases_and_matches_repeated_mul()

### Community 5 - "Community 5"
Cohesion: 0.12
Nodes (16): §0.1 Evidence (final state), `#![allow(dead_code)]` Decision, Commit SHAs, Commits, Concerns, Iteration 1 — `encode_rejects_length_overflow` (tests `encoded_len`), Iteration 2 — `encode_empty_is_empty` (tests `encode_blocks` empty path), Iteration 3 — `encode_is_systematic_and_right_length` (single block) (+8 more)

### Community 6 - "Community 6"
Cohesion: 0.52
Nodes (6): eval(), eval_constant_poly_returns_constant(), mul(), mul_then_eval_is_pointwise_product(), remainder(), remainder_degree_is_below_divisor()

### Community 7 - "Community 7"
Cohesion: 0.14
Nodes (13): §0.1 evidence (final Green commit e556b2f), Commit sequence, CONCERN 1: tdd-guard-rust parser bug (medium severity), CONCERN 2: Brief's single-function test vs TDD-Guard (low severity), CONCERN 3: `#![allow(dead_code)]` at module level (low severity), Concerns, Deliverables, Files changed (+5 more)

### Community 8 - "Community 8"
Cohesion: 0.17
Nodes (11): §0.1 Evidence (final state, commit `761a1db`), 1. `tdd-guard-rust --passthrough` reporter bug (known, per task brief), 2. `remainder_degree_is_below_divisor` test is weak (verbatim from brief), Concerns, Files Created / Modified, Implementation Notes, Status: DONE_WITH_CONCERNS, Steps Executed (+3 more)

### Community 9 - "Community 9"
Cohesion: 0.20
Nodes (9): §0.1 Evidence (GREEN), Commit SHAs, Concerns, GREEN Phase, RED Phase, REFACTOR Phase, Status: COMPLETE, Steps Executed (+1 more)

### Community 10 - "Community 10"
Cohesion: 0.09
Nodes (9): ReedSolomon, crc32(), decode_framed(), encode_framed(), framed_rejects_bad_magic(), framed_rejects_corrupted_header(), framed_rejects_parameter_mismatch(), framed_rejects_unsupported_version() (+1 more)

### Community 11 - "Community 11"
Cohesion: 0.20
Nodes (9): Commit SHAs, Concerns / Notes, Cycle 1: `clean_codeword_has_zero_syndromes`, Cycle 2: `single_error_gives_nonzero_syndromes`, Files Modified, STATUS: COMPLETE, Summary, Task 6 Report — Syndromes (`src/decode.rs`) (+1 more)

### Community 12 - "Community 12"
Cohesion: 0.20
Nodes (9): §0.1 Evidence (Green), Commits, Concerns / Deviations, Green, Red, Refactor, Status, Steps (Red → Green → Refactor) (+1 more)

### Community 13 - "Community 13"
Cohesion: 0.20
Nodes (9): §0.1 evidence (at Green, before `feat:` commit), Concerns / deviations, Green — `feat:`, Red — `test:`, Refactor — none, Summary, Task 9 Report — Forney magnitudes with FCR=112 factor (`src/decode.rs`), TDD-Guard notes (+1 more)

### Community 14 - "Community 14"
Cohesion: 0.29
Nodes (6): §0.1 evidence (at Green, real output), Commit SHAs, Concerns / deviations from the reference draft, Function added, Task 7 Report — Inversionless Berlekamp-Massey (`decode.rs`), TDD steps

### Community 15 - "Community 15"
Cohesion: 0.33
Nodes (5): §6 pre-merge gate — execution plan (durable note), Aggregation & progression to §7, Budget & objective, Gate structure, Segments (self-contained; split further if a single one still overflows)

### Community 27 - "Community 27"
Cohesion: 0.23
Nodes (16): RsError, all_zero(), berlekamp_massey(), bm_locator_degree_matches_single_error(), chien_finds_the_injected_position(), chien_search(), clean_codeword_has_zero_syndromes(), decode_block() (+8 more)

## Knowledge Gaps
- **100 isolated node(s):** `Why a dedicated crate`, `Design invariant`, `Example`, `License`, `Gate structure` (+95 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `encode_framed()` connect `Community 10` to `Community 0`, `Community 27`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Why does `RsError` connect `Community 0` to `Community 10`, `Community 2`, `Community 27`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Why does `decode_framed()` connect `Community 10` to `Community 0`, `Community 27`?**
  _High betweenness centrality (0.021) - this node is a cross-community bridge._
- **What connects `Why a dedicated crate`, `Design invariant`, `Example` to the rest of the system?**
  _100 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.09090909090909091 - nodes in this community are weakly interconnected._
- **Should `Community 4` be split into smaller, more focused modules?**
  _Cohesion score 0.1067193675889328 - nodes in this community are weakly interconnected._
- **Should `Community 5` be split into smaller, more focused modules?**
  _Cohesion score 0.11764705882352941 - nodes in this community are weakly interconnected._