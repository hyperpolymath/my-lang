# My-newsroom Examples

This directory contains examples for all four dialects in the My Language family.

## Directory Structure

```
examples/
├── julia/          # Dempster-Shafer belief fusion (Julia)
├── me/             # Me dialect playground (HTML/JS)
├── solo/           # Solo dialect examples (.solo)
├── duet/           # Duet dialect examples (.duet) - verification
└── ensemble/       # Ensemble dialect examples (.ens) - multi-agent
```

---

## Julia Examples (Dempster-Shafer)

### basic_fusion.jl
```bash
julia --project=. examples/julia/basic_fusion.jl
```

Demonstrates:
- Creating belief masses
- Fusing with Dempster and Yager rules
- Calculating uncertainty intervals

---

## Me Examples (Epistemic Types)

### playground.html
Open in browser:
```bash
open examples/me/playground.html
```

Interactive playground with:
- Belief state declarations
- Distribution assignments
- Fusion examples
- Bayesian updates

---

## Solo Examples (Systems Programming)

### hello_world.solo
```bash
cd solo-compiler
cargo build --release
./target/release/solo check ../examples/solo/hello_world.solo
```

### belief_example.solo
Demonstrates epistemic types with affine ownership.

---

## Duet Examples (Formal Verification)

> **Status:** Specification only - compiler not yet implemented

### verified_fusion.duet
Demonstrates:
- Formal verification of Dempster-Shafer fusion
- `@verify` contracts for correctness
- `@synth` holes for AI-assisted implementation

### verified_sort.duet
Demonstrates:
- Verified quicksort with proofs
- Loop invariants
- AI synthesis with hints

---

## Ensemble Examples (Multi-Agent)

> **Status:** Specification only - runtime not yet implemented

### simple_newsroom.ens
Demonstrates:
- 5-agent newsroom with reporters and editor
- Star topology orchestration
- Belief fusion across agents

### bft_consensus.ens
Demonstrates:
- Byzantine fault tolerance (33% malicious agents)
- Consensus despite adversarial behavior
- Malicious agent detection

---

## Running Examples

```bash
# Julia (Dempster-Shafer)
julia --project=. examples/julia/basic_fusion.jl

# Julia tests
julia --project=. -e 'using Pkg; Pkg.test()'

# Me (open in browser)
open examples/me/playground.html

# Solo (after building compiler)
cd solo-compiler && cargo build --release
./target/release/solo check ../examples/solo/hello_world.solo

# Duet/Ensemble (not yet implemented - examples are specifications)
```

---

## Contributing Examples

Want to add an example? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

Examples should:
- Include SPDX license header
- Be self-contained and runnable
- Include comments explaining key concepts
- Demonstrate one feature clearly
