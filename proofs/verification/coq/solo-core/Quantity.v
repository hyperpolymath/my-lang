(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: QTT quantity semiring                   *)
(*                                                            *)
(* The three-point semiring {0, 1, omega} underpinning the    *)
(* solo dialect's affine/linear discipline. This is the Coq   *)
(* twin of proofs/verification/idris/solo-core/Quantity.idr;  *)
(* the two MUST stay in lock-step (see ALIGNMENT-PLAN.md).     *)
(*                                                            *)
(*    +  |  0   1   w               *  |  0   1   w           *)
(*   ----+-----------              ----+-----------           *)
(*    0  |  0   1   w               0  |  0   0   0           *)
(*    1  |  1   w   w               1  |  0   1   w           *)
(*    w  |  w   w   w               w  |  0   w   w           *)
(* ========================================================== *)

(** * Quantities *)

Inductive Q : Type :=
  | Zero  : Q
  | One   : Q
  | Omega : Q.

(** * Semiring operations *)

Definition qadd (a b : Q) : Q :=
  match a, b with
  | Zero,  q     => q
  | One,   Zero  => One
  | One,   One   => Omega
  | One,   Omega => Omega
  | Omega, _     => Omega
  end.

Definition qmul (a b : Q) : Q :=
  match a, b with
  | Zero,  _     => Zero
  | One,   q     => q
  | Omega, Zero  => Zero
  | Omega, One   => Omega
  | Omega, Omega => Omega
  end.

(** Subquantity ordering: [qle a b = true] when a value with
    quantity [a] may be used where [b] is expected. Affine
    weakening lives here: [qle Zero One = true]. *)
Definition qle (a b : Q) : bool :=
  match a, b with
  | Zero,  _     => true
  | _,     Omega => true
  | One,   One   => true
  | One,   Zero  => false
  | Omega, Zero  => false
  | Omega, One   => false
  end.

(** * Semiring laws (decidable: exhaustive case split) *)

Lemma qadd_zero_l : forall q, qadd Zero q = q.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qadd_zero_r : forall q, qadd q Zero = q.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qadd_comm : forall a b, qadd a b = qadd b a.
Proof. intros a b; destruct a, b; reflexivity. Qed.

Lemma qadd_assoc : forall a b c,
  qadd (qadd a b) c = qadd a (qadd b c).
Proof. intros a b c; destruct a, b, c; reflexivity. Qed.

Lemma qmul_zero_l : forall q, qmul Zero q = Zero.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qmul_zero_r : forall q, qmul q Zero = Zero.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qmul_one_l : forall q, qmul One q = q.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qmul_one_r : forall q, qmul q One = q.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qmul_comm : forall a b, qmul a b = qmul b a.
Proof. intros a b; destruct a, b; reflexivity. Qed.

Lemma qmul_assoc : forall a b c,
  qmul (qmul a b) c = qmul a (qmul b c).
Proof. intros a b c; destruct a, b, c; reflexivity. Qed.

Lemma qmul_distrib_l : forall a b c,
  qmul a (qadd b c) = qadd (qmul a b) (qmul a c).
Proof. intros a b c; destruct a, b, c; reflexivity. Qed.

Lemma qmul_distrib_r : forall a b c,
  qmul (qadd a b) c = qadd (qmul a c) (qmul b c).
Proof. intros a b c; destruct a, b, c; reflexivity. Qed.

(** * Ordering sanity *)

Lemma qle_zero : forall q, qle Zero q = true.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qle_omega : forall q, qle q Omega = true.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qle_refl : forall q, qle q q = true.
Proof. intros q; destruct q; reflexivity. Qed.

Lemma qle_trans : forall a b c, qle a b = true -> qle b c = true -> qle a c = true.
Proof. intros a b c Hab Hbc; destruct a, b, c; simpl in *; congruence. Qed.
