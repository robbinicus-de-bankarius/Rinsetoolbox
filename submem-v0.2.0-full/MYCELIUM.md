# Mycelium for `submem`

Mycelium adds a deterministic symbolic cycle and SVG renderer to the existing nine-octave `submem` engine. It is an amalgamation of the earlier Mycelium Rust draft, the exact Veca sanctum geometry, and the LumiLuminal reservoir swirl.

## Honest scope

This is creative/simulation software. It does not claim cryptographic security, clinical value, physical antenna behaviour, or evidence of metaphysical effects. The names provide a symbolic structure; the tests check software and mathematical invariants.

## Integrated, not duplicated

- `veca_sanctum::MetatronSanctumProof` supplies the exact **13 centres, 78 chords, 294 raw crossings, 97 field nodes, and 4,656 connections**.
- `reservoir::PlanViewInterferenceSwirl` now exposes a pure `unveil_symbolic` sampler while preserving the existing live-bank `unveil_from_bank` path.
- Thirteen stable-indexed Mycelium nodes map directly onto the existing Metatron centres and cover all three roles, four bodies, and seven chakras.
- Seeded xorshift dynamics, non-deleting lateral inhibition, activity regulation, and an optional symbolic aperture lead into final-state validation.
- Via Negativa returns explained decisions and fails closed on invalid candidates or missing roles.

## Run

From this directory:

```sh
cargo test
cargo run --release -- mycelium --output mycelium.svg
```

Options:

```text
--output PATH   SVG destination (default: mycelium.svg)
--turns NUMBER  Symbolic swirl turns (default: 100)
--seed INTEGER  Decimal random seed; zero maps to the nonzero default
--closed        Leave the symbolic aperture closed
```

Example:

```sh
mkdir -p artifacts
cargo run --release -- mycelium --turns 100 --seed 42 --output artifacts/mycelium-42.svg
```

Open the generated SVG in a browser or vector editor. It displays the 13 centres and all 78 chords, all 4,096 swirl cells, final-state measurements, and the four candidate decisions.

## Module layout

- `src/mycelium.rs`: node model, dynamics, filter, cycle, SVG renderer, and ten module tests.
- `src/reservoir.rs`: shared pure/live swirl sampling.
- `src/main.rs`: `submem mycelium` CLI integration.
- `src/lib.rs`: exports the new module.

## Deliberate corrections to the earlier draft

- No hardcoded Metatron count stubs: the exact rational sanctum proof drives the cycle.
- No second swirl implementation: Mycelium and the reservoir share one sampler.
- No constant security or holography flags: final state and geometric invariants are measured instead.
- No thread-local random state: every run receives an explicit seed.
- No six-node Crown substitution: thirteen nodes represent every category.
- No canonical-vow verification claim: canonical vow text was not supplied.
