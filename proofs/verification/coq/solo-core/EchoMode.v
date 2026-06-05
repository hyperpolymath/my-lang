(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: Echo linearity mode                     *)
(* (Coq twin of EchoMode.idr)                                 *)
(*                                                            *)
(* The two-point thin poset [Linear <= Affine] that decorates *)
(* echo residues. Mirrors `Mode` and `_<=m_` in the           *)
(* echo-types Agda library (EchoLinear.agda), and the Rust    *)
(* `EchoMode` in crates/my-lang/src/types.rs. The headline    *)
(* facts — reflexivity, transitivity, the weaken direction    *)
(* [Linear <= anything], and the *no-section* fact            *)
(* [~ (Affine <= Linear)] — are the type-algebra content      *)
(* behind the checker's Echo subtyping.                       *)
(* ========================================================== *)

(** * Linearity mode of an echo residue. *)

Inductive Mode : Type :=
  | Linear : Mode   (* full residue; distinctions retained *)
  | Affine : Mode.  (* collapsed residue; distinctions weakened away *)

(** The mode ordering [m1 <=m m2]: an echo at mode [m1] may be
    *weakened* to mode [m2]. [Linear] weakens to anything; [Affine]
    weakens only to itself. This is [EchoLinear._<=m_]. *)
Definition mle (m1 m2 : Mode) : bool :=
  match m1, m2 with
  | Linear, _      => true
  | Affine, Affine => true
  | Affine, Linear => false
  end.

(** ** Poset laws (exhaustive case split — no axioms). *)

(** Reflexivity. *)
Lemma mle_refl : forall m, mle m m = true.
Proof. destruct m; reflexivity. Qed.

(** Transitivity. *)
Lemma mle_trans : forall m1 m2 m3,
  mle m1 m2 = true -> mle m2 m3 = true -> mle m1 m3 = true.
Proof. destruct m1, m2, m3; simpl; intros; auto; discriminate. Qed.

(** Antisymmetry — the poset is *thin* (two-point). *)
Lemma mle_antisym : forall m1 m2,
  mle m1 m2 = true -> mle m2 m1 = true -> m1 = m2.
Proof. destruct m1, m2; simpl; intros H1 H2; auto; discriminate. Qed.

(** [EchoLinear.weaken] direction: [Linear] is the bottom — a linear
    residue weakens to any mode. *)
Lemma mle_linear_bot : forall m, mle Linear m = true.
Proof. destruct m; reflexivity. Qed.

(** Concretely, [Linear] weakens to [Affine]. *)
Lemma weaken_linear_affine : mle Linear Affine = true.
Proof. reflexivity. Qed.

(** [EchoLinear.no-section-weaken]: weakening is irreversible. An
    [Affine] residue can NOT be used where a [Linear] one is required —
    the weakening map has no section. *)
Lemma no_section_weaken : mle Affine Linear = false.
Proof. reflexivity. Qed.

(** The same fact as a negation, in the form the checker's Echo
    subtyping relies on: [Affine] does not weaken to [Linear]. *)
Lemma affine_not_below_linear : mle Affine Linear <> true.
Proof. discriminate. Qed.
