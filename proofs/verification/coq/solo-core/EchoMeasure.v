(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ============================================================ *)
(* my-lang Solo core: Echo residue MEASURE — the E4 SEAM witness *)
(*                                                              *)
(* The downstream half of the E4 seam capstone (the interface    *)
(* RESIDUE_MEASURE lives in ResourceAlgebra.v, which may not     *)
(* Require Tropical without a cycle — hence this separate file). *)
(*                                                              *)
(* It does two things, both axiom-free:                          *)
(*   (1) INHABITS the seam at the (DONE, R4) Tropical cost        *)
(*       carrier — a concrete RESIDUE_MEASURE Tropical instance   *)
(*       proving the seam is non-vacuous;                        *)
(*   (2) WITNESSES measure-INDEPENDENCE on the Coq side — the     *)
(*       measure is NON-INJECTIVE (echo_measure_not_injective),   *)
(*       the residue-level echo of the upstream Agda theorem      *)
(*       equal-measure-does-not-imply-equal-echo.                *)
(*                                                              *)
(* FAITHFULNESS FENCE (honest scope):                            *)
(*  * A residue is modelled as an echo REINDEXING TRACE — the     *)
(*    list of EchoMode.Mode's a value passes through (each kernel *)
(*    Weaken of EchoResidue.v appends one). combine = trace       *)
(*    concatenation, empty = the empty trace: a FREE monoid, the  *)
(*    minimal composition-bearing residue carrier. We do NOT      *)
(*    claim this IS the full echo residue object; it is the       *)
(*    composition skeleton the measure homomorphism needs.        *)
(*  * The measure is the accumulated COST: each Affine collapse   *)
(*    (an information-losing weakening) costs 1, each Linear step  *)
(*    costs 0 — exactly the min-plus reading of Tropical          *)
(*    (qmul = +, one = Fin 0). measure : Residue -> Tropical.Q is *)
(*    ONE-DIRECTIONAL (E -> R); there is no inverse (it is        *)
(*    non-injective). Echo is NOT encoded as a resource instance. *)
(*  * The full "equal measure does not imply equal ECHO MODALITY" *)
(*    theorem (over the degrade-compose modality) is proven       *)
(*    UPSTREAM (echo-types, --safe --without-K) and CITED here,   *)
(*    not re-derived; the Coq mirror carries the measure          *)
(*    HOMOMORPHISM + the residue-level non-injectivity only.      *)
(*  Echo-types audit: RELEVANT (axis-3 ECHO) — this rung mirrors  *)
(*  the audited upstream seam; all other ladders stay NOT-RELEVANT.*)
(* ============================================================ *)

Require Import List.
Import ListNotations.
Require Import PeanoNat.
Require Import EchoMode.
Require Import ResourceAlgebra.
Require Import Tropical.

(* ---- the residue carrier: an echo reindexing trace ---- *)

(* The cost of a single echo step: an Affine collapse loses one  *)
(* distinction (cost 1); a Linear step retains everything (0).   *)
Definition mode_cost (m : Mode) : nat :=
  match m with Linear => 0 | Affine => 1 end.

(* The accumulated cost of a whole trace. *)
Fixpoint trace_cost (l : list Mode) : nat :=
  match l with
  | []      => 0
  | m :: r  => mode_cost m + trace_cost r
  end.

(* Cost is additive over trace concatenation — this is exactly   *)
(* what makes [measure] a monoid homomorphism below.             *)
Lemma trace_cost_app : forall l1 l2,
  trace_cost (l1 ++ l2) = trace_cost l1 + trace_cost l2.
Proof.
  induction l1 as [| m r IH]; intro l2; simpl.
  - reflexivity.
  - rewrite IH. apply Nat.add_assoc.
Qed.

(* ---- (1) the seam INHABITED at the Tropical cost carrier ---- *)
(* A concrete RESIDUE_MEASURE over Tropical: residues are traces, *)
(* combine is concatenation, and the measure is the tropical Fin  *)
(* of the accumulated cost. The two homomorphism laws hold        *)
(* because Tropical.qmul on Fin is +, and trace cost is additive. *)
Module EchoTraceTropical <: RESIDUE_MEASURE Tropical.

  Definition Residue : Type := list Mode.
  Definition empty   : Residue := [].
  Definition combine (a b : Residue) : Residue := a ++ b.
  Definition measure (l : Residue) : Tropical.Q := Tropical.Fin (trace_cost l).

  (* empty trace -> zero cost -> Fin 0 = Tropical.one. *)
  Lemma measure_empty : measure empty = Tropical.one.
  Proof. reflexivity. Qed.

  (* concatenation -> additive cost -> tropical qmul (= +). *)
  Lemma measure_combine : forall a b,
    measure (combine a b) = Tropical.qmul (measure a) (measure b).
  Proof.
    intros a b. unfold measure, combine.
    rewrite trace_cost_app. reflexivity.
  Qed.

End EchoTraceTropical.

(* ---- (2) measure-INDEPENDENCE, Coq side ---- *)
(* The measure is NON-INJECTIVE: two DISTINCT residue traces can  *)
(* carry EQUAL cost ([Affine] and [Linear; Affine] both cost 1),  *)
(* so the measure cannot reconstruct the residue. This is the     *)
(* residue-level echo of the upstream Agda                        *)
(* equal-measure-does-not-imply-equal-echo: equal measure does    *)
(* NOT pin the residue, a fortiori not the Echo modality. The     *)
(* full modality-level statement is proven upstream and cited     *)
(* (see the FAITHFULNESS FENCE) — not re-derived here.            *)
Theorem echo_measure_not_injective :
  exists r1 r2 : EchoTraceTropical.Residue,
    r1 <> r2 /\ EchoTraceTropical.measure r1 = EchoTraceTropical.measure r2.
Proof.
  exists [Affine], [Linear; Affine]. split.
  - discriminate.
  - reflexivity.
Qed.
