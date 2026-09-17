<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Proof debt and the trusted boundary

This register distinguishes abstract interface obligations from global axioms.
It records the published source inspected on 2026-09-07; it does not promote
unpublished proof work or establish that every language feature is verified.

## (a) Discharged in concrete instances

The 15 `Axiom` declarations in
`proofs/verification/coq/solo-core/ResourceAlgebra.v` are fields of Coq module
types: ten `SEMIRING` laws, three additional `ORDERED_SEMIRING` order laws, and
two `RESIDUE_MEASURE` homomorphism laws. An abstract soundness functor is
conditional on these fields. A concrete implementation must supply proofs of
the fields when checked against its module signature.

The source identifies `Linear3`, `Tropical` and `EchoTraceTropical` as concrete
instances. Their instantiated theorem dependencies, rather than the spelling
`Axiom` inside a module type, determine the global trusted base. The existing
`proofs.yml` workflow checks concrete assumption closure. Passing the structural
trusted-base policy only establishes that these sites are documented; it is
not a new execution of the Coq checker or a proof of the Rust implementation.

## (b) Budgeted boundaries

No new refutation budget or runtime extraction guarantee is claimed by this
audit. See `proofs/STATUS.adoc` for the scope of existing checks.

## (c) Necessary global axioms

The interface fields above are not classified as necessary global axioms.
This review does not certify the dependency closure of every proof in the tree.

## (d) Open obligations

- **Owner:** repository maintainer, @hyperpolymath.
- **Scope:** the model/implementation correspondence and remaining obligations
  recorded in `proofs/STATUS.adoc` and `proofs/ALIGNMENT-PLAN.adoc`.
- **Plan:** keep each theorem tied to its actual language fragment, concrete
  resource algebra and executable checker; close the correspondence obligations
  with compiler/proof checks and counterexample tests before expanding claims.
- **Deadline:** INDEFINITE: these are separate research obligations, not
  discharged by the documentation and CI repairs in this PR.
