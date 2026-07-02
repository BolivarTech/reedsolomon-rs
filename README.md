# reedsolomon

Pure-Rust, **no-`unsafe`** Reed-Solomon forward error correction (FEC) over
GF(2^8). Default code: **RS(255, 223)** (CCSDS) — 223 data + 32 parity bytes
per 255-byte codeword, correcting up to **16 corrupted bytes per block**.

> **Status: v0.1.0.** Native pure-Rust GF(2^8) codec — systematic encoder and a
> syndromes → inversionless Berlekamp-Massey → Chien → Forney decoder with
> mandatory post-correction verification. KAT-pinned against an independent
> reference; property-tested and fuzzed. No third-party FEC dependency.

## Why a dedicated crate

This is the FEC layer extracted from the [`cryptovault`](https://crates.io/crates/cryptovault)
module so it can be reused on its own and so the vault has **zero third-party
FEC dependency**. It is **not** a cryptographic primitive: it protects
already-encrypted bytes against bit-rot, carries no secrets, and has no
secret-dependent timing.

## Design invariant

The decoder **declares failure rather than mis-correct.** Returning
wrong-but-plausible data on an over-capacity block is the one outcome this crate
must never produce silently. (In `cryptovault` the AES-GCM-SIV tag is a final
backstop, but this crate must be correct on its own.)

## Example

```rust
use reedsolomon::ReedSolomon;

let rs = ReedSolomon::default();            // RS(255, 223)
let encoded = rs.encode(b"payload bytes").unwrap();
let decoded = rs.decode(&encoded, 13).unwrap();
assert_eq!(&decoded, b"payload bytes");
```

## License

Licensed under either of **MIT** or **Apache-2.0** at your option.
