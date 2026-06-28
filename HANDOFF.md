# HANDOFF — `reedsolomon` crate

> Agent brief for building this crate. Self-contained: you do not need the
> `magi` / `cryptovault` repos to do this work.

## Goal
Implement a **native, pure-Rust, no-`unsafe`** Reed-Solomon codec over GF(2^8),
default **RS(255, 223)** (CCSDS), replacing any third-party FEC dependency.
The public API is already defined in `src/lib.rs` (`ReedSolomon::{new, encode,
decode}`, `RsError`) — keep it stable; fill in the `todo!()` bodies.

## Why / context
This is the FEC layer of the `cryptovault` crate. It wraps an
**already-encrypted** payload (`nonce || ciphertext || tag`) so it survives
bit-rot. It is **not** a security primitive: no secrets, no secret-dependent
branches/timing needed. Correctness and "never mis-correct" are what matter.

## Algorithm specification
- **Field:** GF(2^8) with primitive polynomial **0x11D**
  (`x^8 + x^4 + x^3 + x^2 + 1`) — the standard RS(255,223)/CCSDS field.
  Build log/antilog (exp) tables for fast multiply/inverse.
- **Code:** systematic RS(*n*=255, *k*=223), `2t = 32` parity, corrects `t = 16`
  byte errors per block. Generator polynomial `g(x) = Π (x - α^i)`,
  `i = 0..2t` (or `1..=2t`; pick one convention and KAT-pin it).
- **Encoder:** systematic — parity = remainder of `message(x)·x^(2t)` divided by
  `g(x)` over GF(2^8). Append parity to each `k`-byte chunk; last chunk is
  zero-padded to `k`, padding stripped on decode via `original_len`.
- **Decoder:** syndromes `S_j = R(α^j)` → **Berlekamp-Massey** (error-locator
  polynomial Λ) → **Chien search** (error positions) → **Forney algorithm**
  (error magnitudes) → correct. If syndromes are all zero → no errors. If the
  number of errors exceeds `t`, or Λ degree / Chien roots are inconsistent →
  return [`RsError::Uncorrectable`]. **Never return mis-corrected data.**

## Test bar (MUST, all green)
1. **KATs** — fixed known-answer vectors for encode and decode against an
   independent RS(255,223) reference; pin the generator-root convention.
2. **Round-trip** (property test, `proptest`): `decode(encode(x), x.len()) == x`
   for arbitrary `x`.
3. **Recovery:** corrupt **≤16** bytes/block → decode recovers exactly.
4. **Failure, not mis-correction:** corrupt **>16** bytes/block → `decode`
   returns `Uncorrectable` (assert it never returns wrong data).
5. **Fuzzing** (`cargo-fuzz`): `decode` never panics on arbitrary input.
6. Edge cases: empty input, single byte, exactly `k`, multi-block,
   `new()` precondition errors.

## Quality gates (every commit)
- `cargo nextest run` green · `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --check` clean · `cargo build --release` · `cargo doc --no-deps`
  (no warnings) · `cargo audit` clean.
- `#![forbid(unsafe_code)]` stays. Rustdoc on all public items. No magic
  numbers (named constants). File header (`// Author / Version / Date`).
- TDD: Red → Green → Refactor; atomic commits, English imperative messages,
  no AI mentions, no `Co-Authored-By`.

## Done =
Native encoder + decoder pass the full test bar, no third-party FEC dependency,
all gates green. Then bump to `0.1.0` and it can replace the FEC inside
`cryptovault`.
