# Substitution Lemma (fixed)

This document records the substitution lemma proof that underpins Solo's affine-aware type system.

- **Source**: `proofs/shared/type-system/soundness.md` (see sections around the formal statement and proof for a step-by-step derivation).
- **Key points**:
  * Indexes each binding to a type, then shows that substituting a value of the declared type preserves typing in the surrounding environment.
  * Handles context splitting needed for affine/linear resources, ensuring resources are neither duplicated nor dropped inadvertently.
  * Supports downstream lemmas (progress/preservation) that are recorded in the same file and in `proofs/shared/operational-semantics/small-step.md`.

The original handover file `substitution_lemma_fixed.md` can be recreated from these sections; this document serves as the canonical repo copy.
