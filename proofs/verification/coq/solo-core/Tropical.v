(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ============================================================ *)
(* my-lang Solo core: TROPICAL (min-plus) resource instance.    *)
(*                                                              *)
(* R4 — the acceptance test for the R2 functorisation: an       *)
(* INFINITE carrier (ℕ ∪ {∞}). If [SoloCoreF] secretly relied   *)
(* on the finite three-point carrier, it could not be           *)
(* instantiated here. It can:                                   *)
(*                                                              *)
(*   Module SoloTropical := SoloCoreF Tropical.                 *)
(*                                                              *)
(* yields progress / preservation / affine_pres for the cost    *)
(* semiring with NO new soundness proof — only the algebra      *)
(* (the 10 SEMIRING laws + the qle preorder) is discharged here.*)
(*                                                              *)
(* Min-plus ("cost"): ⊕ = min, ⊗ = +, 0 = ∞ (Inf), 1 = Fin 0.   *)
(* The order [qle] is the bottom-anchored ordering with [zero]   *)
(* (= ∞) at the bottom — i.e. reverse cost order — so the affine *)
(* layer's [qle_zero] (zero is least) holds. *)
(* ============================================================ *)

Require Import PeanoNat.
Require Import Lia.
Require Import ResourceAlgebra.
Require Import SoloCore.

(* + distributes over min, both sides (the only non-mechanical laws). *)
Lemma add_min_distr_l : forall a b c, a + Nat.min b c = Nat.min (a + b) (a + c).
Proof.
  intros a b c. destruct (Nat.le_ge_cases b c) as [H | H].
  - rewrite (Nat.min_l b c H), Nat.min_l by lia. reflexivity.
  - rewrite (Nat.min_r b c H), Nat.min_r by lia. reflexivity.
Qed.

Lemma add_min_distr_r : forall a b c, Nat.min a b + c = Nat.min (a + c) (b + c).
Proof.
  intros a b c. destruct (Nat.le_ge_cases a b) as [H | H].
  - rewrite (Nat.min_l a b H), Nat.min_l by lia. reflexivity.
  - rewrite (Nat.min_r a b H), Nat.min_r by lia. reflexivity.
Qed.

Module Tropical <: ORDERED_SEMIRING.

  Inductive trop : Type := Fin (n : nat) | Inf.
  Definition Q : Type := trop.

  Definition zero : Q := Inf.
  Definition one  : Q := Fin 0.

  Definition qadd (a b : Q) : Q :=
    match a, b with
    | Inf,   _     => b
    | _,     Inf   => a
    | Fin x, Fin y => Fin (Nat.min x y)
    end.

  Definition qmul (a b : Q) : Q :=
    match a, b with
    | Inf,   _     => Inf
    | _,     Inf   => Inf
    | Fin x, Fin y => Fin (x + y)
    end.

  Definition qle (a b : Q) : bool :=
    match a, b with
    | Inf,   _     => true
    | Fin _, Inf   => false
    | Fin x, Fin y => Nat.leb y x
    end.

  (* --- additive (min) --- *)
  Lemma qadd_comm  : forall a b, qadd a b = qadd b a.
  Proof. intros [x|] [y|]; simpl; try reflexivity. f_equal; apply Nat.min_comm. Qed.

  Lemma qadd_assoc : forall a b c, qadd (qadd a b) c = qadd a (qadd b c).
  Proof. intros [x|] [y|] [z|]; simpl; try reflexivity. f_equal; symmetry; apply Nat.min_assoc. Qed.

  Lemma qadd_zero_l : forall q, qadd zero q = q.
  Proof. intros [x|]; reflexivity. Qed.

  Lemma qadd_zero_r : forall q, qadd q zero = q.
  Proof. intros [x|]; reflexivity. Qed.

  (* --- multiplicative (plus) --- *)
  Lemma qmul_assoc  : forall a b c, qmul (qmul a b) c = qmul a (qmul b c).
  Proof. intros [x|] [y|] [z|]; simpl; try reflexivity. f_equal; lia. Qed.

  Lemma qmul_one_l  : forall q, qmul one q = q.
  Proof. intros [x|]; simpl; [ f_equal; lia | reflexivity ]. Qed.

  Lemma qmul_zero_r : forall q, qmul q zero = zero.
  Proof. intros [x|]; reflexivity. Qed.

  Lemma qmul_zero_l : forall q, qmul zero q = zero.
  Proof. intros q; reflexivity. Qed.

  Lemma qmul_distrib_l : forall a b c, qmul a (qadd b c) = qadd (qmul a b) (qmul a c).
  Proof. intros [x|] [y|] [z|]; simpl; try reflexivity. f_equal; apply add_min_distr_l. Qed.

  Lemma qmul_distrib_r : forall a b c, qmul (qadd a b) c = qadd (qmul a c) (qmul b c).
  Proof. intros [x|] [y|] [z|]; simpl; try reflexivity. f_equal; apply add_min_distr_r. Qed.

  (* --- order (bottom-anchored: Inf = zero is least) --- *)
  Lemma qle_refl  : forall q, qle q q = true.
  Proof. intros [n|]; simpl; [ apply Nat.leb_refl | reflexivity ]. Qed.

  Lemma qle_trans : forall a b c, qle a b = true -> qle b c = true -> qle a c = true.
  Proof.
    intros [x|] [y|] [z|]; simpl; intros H1 H2; try reflexivity; try discriminate.
    apply Nat.leb_le in H1; apply Nat.leb_le in H2; apply Nat.leb_le; lia.
  Qed.

  Lemma qle_zero  : forall q, qle zero q = true.
  Proof. intros q; reflexivity. Qed.

End Tropical.

(* ============================================================ *)
(* The payoff: soundness at the tropical carrier, for free.     *)
(* ============================================================ *)

Module SoloTropical := SoloCoreF Tropical.
