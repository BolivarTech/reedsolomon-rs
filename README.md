# reedsolomon

[![Crates.io](https://img.shields.io/crates/v/reedsolomon.svg)](https://crates.io/crates/reedsolomon)
[![Documentation](https://docs.rs/reedsolomon/badge.svg)](https://docs.rs/reedsolomon)
[![License](https://img.shields.io/crates/l/reedsolomon.svg)](https://github.com/BolivarTech/reedsolomon-rs#license)
[![CI](https://github.com/BolivarTech/reedsolomon-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/BolivarTech/reedsolomon-rs/actions/workflows/ci.yml)
[![Release](https://github.com/BolivarTech/reedsolomon-rs/actions/workflows/release.yml/badge.svg)](https://github.com/BolivarTech/reedsolomon-rs/actions/workflows/release.yml)

**Pure-Rust, `#![forbid(unsafe_code)]` Reed-Solomon forward error correction over
GF(2^8)**, defaulting to **RS(255, 223)** (CCSDS): 223 data + 32 parity bytes per
255-byte codeword, correcting up to **16 corrupted bytes per block**. Zero runtime
dependencies.

It wraps an already-encrypted, opaque payload so it survives bit-rot and byte-level
corruption in storage or transmission. It is the FEC layer extracted from
[`cryptovault`](https://crates.io/crates/cryptovault) so the vault has **zero
third-party FEC dependency**, but it stands alone and is reusable on its own.

> **Not a cryptographic primitive.** It protects already-encrypted bytes, carries no
> secrets, and has no secret-dependent timing. Its only goals are correctness and the
> **never-mis-correct** guarantee below.

## Features

| Capability | Detail |
|------------|--------|
| **Symbol-level correction** | Recovers up to `t = parity_len / 2` corrupted **bytes** per block (16 for the default code), regardless of how many bits within each byte are wrong |
| **Never mis-corrects** | Mandatory post-correction syndrome verification: any `Ok` is provably a valid codeword within distance `t`; beyond capacity it returns [`RsError::Uncorrectable`], never wrong-but-plausible data |
| **Generic geometry** | Any `(parity_len, data_len)` with `parity_len + data_len ≤ 255`; RS(255, 223) is the `Default` |
| **Self-describing framing** | Optional `encode_framed` / `decode_framed` with a CRC-checked header that carries the code parameters and original length |
| **No `unsafe`** | `#![forbid(unsafe_code)]` crate-wide |
| **No runtime deps** | std-only; `proptest` / `cargo-fuzz` are dev-only |
| **DoS-hardened allocation** | Input-scaled output uses `try_reserve` → `InvalidInput` instead of aborting on OOM |

## Installation

```toml
[dependencies]
reedsolomon = "0.2"
```

## Quick Start

### Raw path (zero overhead)

The caller tracks the code parameters and the original length.

```rust
use reedsolomon::ReedSolomon;

fn main() {
    let rs = ReedSolomon::default();            // RS(255, 223)
    let data: &[u8] = b"nonce || ciphertext || tag";

    let encoded = rs.encode(data).unwrap();     // data + parity codewords
    let decoded = rs.decode(&encoded, data.len()).unwrap();
    assert_eq!(decoded.as_slice(), data);
}
```

Corruption within capacity is recovered exactly:

```rust
use reedsolomon::ReedSolomon;

fn main() {
    let rs = ReedSolomon::default();
    let data: &[u8] = b"survives bit-rot";
    let mut encoded = rs.encode(data).unwrap();

    for i in 0..16 {                            // corrupt 16 bytes (= t)
        encoded[i * 7] ^= 0xFF;
    }
    assert_eq!(rs.decode(&encoded, data.len()).unwrap().as_slice(), data);
}
```

### Framed path (self-describing)

The 17-byte CRC-checked header carries `(version, parity_len, data_len, original_len)`,
so the decoder needs no out-of-band parameters and rejects a parameter mismatch instead
of silently mis-decoding.

```rust
use reedsolomon::ReedSolomon;

fn main() {
    let rs = ReedSolomon::default();
    let data: &[u8] = b"self-describing";

    let framed = rs.encode_framed(data).unwrap();
    let decoded = rs.decode_framed(&framed).unwrap();   // no original_len needed
    assert_eq!(decoded.as_slice(), data);
}
```

## How It Works

Systematic encoder — parity is the remainder of `message(x)·x^{parity_len}` mod `g(x)`,
appended per `data_len`-byte chunk (final chunk zero-padded). The decoder runs, per block:

```
syndromes ─▶ inversionless Berlekamp-Massey ─▶ Chien search ─▶ Forney ─▶ correct ─▶ verify
   S_j            Λ (error locator)             positions      magnitudes           post-check
```

The final **post-correction syndrome verification** recomputes the syndromes of the
corrected word and rejects the block unless all are zero — the primary defence against
mis-correction. Every inconsistency (errors > `t`, `deg(Λ)` ≠ Chien-root count, a root
outside `[0, n)`, a zero Forney magnitude, non-zero residual syndromes) yields
`RsError::Uncorrectable`.

| Module | Responsibility |
|--------|----------------|
| `gf256` | GF(2^8) field: compile-time `const` log/exp tables, `add`/`mul`/`inv`/`div`/`pow` |
| `poly` | Polynomial ops over GF(2^8): evaluate, multiply, remainder |
| `encode` | Generator `g(x)` construction and systematic encoding |
| `decode` | Syndromes → iBM → Chien → Forney → correction → verification |
| `crc` | Self-contained CRC-32/IEEE for the framed header |
| `frame` | Self-describing framed encode/decode |

## Correctness Guarantee

This is a **bounded-distance decoder**:

- For **≤ `t`** genuine byte errors per block, recovery is **exact**.
- Whenever `decode` returns `Ok`, the result is a valid codeword within Hamming distance
  `t` of the received block — never arbitrary bytes.
- For **> `t`** errors it returns `Uncorrectable`, **except** the mathematically inherent,
  negligible-probability case where the corruption lands within distance `t` of a
  *different* valid codeword (RS(255, 223) has minimum distance `d = 33`). This residual
  is shared by every RS decoder and cannot be removed by syndrome verification; callers
  needing detection beyond `t` should layer an integrity check (e.g. an AEAD tag) above
  the FEC, as `cryptovault` does.

## Convention

Fixed for every instance and pinned by Known-Answer Tests against an independently
configured reference:

| Parameter | Value |
|-----------|-------|
| Field polynomial | `0x187` (`x^8 + x^7 + x^2 + x + 1`, CCSDS) |
| Basis | Conventional (polynomial) — **not** the CCSDS dual basis |
| Primitive element | `α = 0x02` |
| First consecutive root | `FCR = 112` |

> **Caller responsibility (raw path).** The raw stream is not self-describing: decode with
> the same `(parity_len, data_len)` used to encode, and pass the true `original_len`. One
> parameter-mismatch family (same `n`, `parity_decode ≤ parity_encode`) is silent — see the
> `decode` rustdoc for the worked example. The **framed** methods eliminate both footguns.
> Dual-basis wire compatibility with CCSDS hardware is a non-goal.

## Testing

```bash
cargo nextest run                          # unit + KAT + property + edge + framed
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo doc --no-deps
cargo audit
```

- **KAT** — encode/decode/failure vectors pinned against the external `reedsolo`
  reference for RS(255, 223), the shortened RS(15, 11, 4), and odd-parity RS(13, 10, 3).
- **Property** (`proptest`) — round-trip, ≤ `t` recovery, and the no-garbage invariant.
- **Fuzz** (`cargo-fuzz`) — `decode`, `encode`, and `decode_framed` never panic on
  arbitrary input (`fuzz/`; requires a nightly toolchain and a libFuzzer-capable host).

## Minimum Supported Rust Version

Rust **1.96**. CI builds and tests on 1.96.

## License

Licensed under either of **[MIT](LICENSE-MIT)** or **[Apache-2.0](LICENSE-APACHE)** at
your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual-licensed
as above, without any additional terms or conditions.

## Credits

Developed by [BolivarTech](https://github.com/BolivarTech) — Julian Bolivar.
