(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: separated QTT context (type ctx + usage)*)
(*                                                            *)
(* The standard QTT presentation, adopted for the F1.4        *)
(* preservation track (design decision, 2026-06-12): the      *)
(* TYPE context [tctx] (types only) is fixed across a         *)
(* derivation's splits, while the USAGE vector [uvec]         *)
(* (quantities only) is what gets added/scaled. Because       *)
(* addition and scaling now act purely on quantities, the     *)
(* algebra is genuinely clean — [uadd] is commutative and     *)
(* associative with no type-shape asymmetry (the wart of the  *)
(* old conflated `ctx`, where ctx_add took the type from its  *)
(* first argument).                                           *)
(* ========================================================== *)

Require Import Quantity.
Require Import Syntax.

(** * Type context — types only. *)
Inductive tctx : Type :=
  | TEmpty : tctx
  | TSnoc  : tctx -> ty -> tctx.

Fixpoint tlen (G : tctx) : nat :=
  match G with TEmpty => 0 | TSnoc G' _ => S (tlen G') end.

(** * Usage vector — quantities only, shape-matched to a [tctx]. *)
Inductive uvec : Type :=
  | UEmpty : uvec
  | USnoc  : uvec -> Q -> uvec.

Fixpoint ulen (D : uvec) : nat :=
  match D with UEmpty => 0 | USnoc D' _ => S (ulen D') end.

(** The all-zero usage of a type context's shape ("nothing is used"). *)
Fixpoint uzero (G : tctx) : uvec :=
  match G with TEmpty => UEmpty | TSnoc G' _ => USnoc (uzero G') Zero end.

(** Scalar multiplication of a usage vector. *)
Fixpoint uscale (q : Q) (D : uvec) : uvec :=
  match D with
  | UEmpty => UEmpty
  | USnoc D' qe => USnoc (uscale q D') (qmul q qe)
  end.

(** Pointwise addition of usage vectors — partial (defined on equal
    shapes; in a derivation both summands range over the same [tctx]). *)
Fixpoint uadd (D1 D2 : uvec) : option uvec :=
  match D1, D2 with
  | UEmpty, UEmpty => Some UEmpty
  | USnoc D1' q1, USnoc D2' q2 =>
      match uadd D1' D2' with
      | None => None
      | Some D => Some (USnoc D (qadd q1 q2))
      end
  | _, _ => None
  end.

(* ========================================================== *)
(* The clean algebra (the payoff of separation).              *)
(* ========================================================== *)

Lemma uzero_len : forall G, ulen (uzero G) = tlen G.
Proof. induction G as [| G IH a]; simpl; [reflexivity | rewrite IH; reflexivity]. Qed.

Lemma uscale_len : forall q D, ulen (uscale q D) = ulen D.
Proof. intros q. induction D as [| D IH qe]; simpl; [reflexivity | rewrite IH; reflexivity]. Qed.

(** Scaling is a monoid action of (Q, qmul, One). *)
Lemma uscale_one : forall D, uscale One D = D.
Proof. induction D as [| D IH qe]; simpl; [reflexivity | rewrite IH; reflexivity]. Qed.

Lemma uscale_compose : forall q1 q2 D,
  uscale q1 (uscale q2 D) = uscale (qmul q1 q2) D.
Proof.
  intros q1 q2. induction D as [| D IH qe]; simpl.
  - reflexivity.
  - rewrite IH, qmul_assoc. reflexivity.
Qed.

(** Scaling absorbs the zero usage. *)
Lemma uscale_zero : forall q G, uscale q (uzero G) = uzero G.
Proof.
  intros q. induction G as [| G IH a]; simpl.
  - reflexivity.
  - rewrite IH, qmul_zero_r. reflexivity.
Qed.

(** Addition is COMMUTATIVE — clean, no type-shape caveat (the win). *)
Lemma uadd_comm : forall D1 D2, uadd D1 D2 = uadd D2 D1.
Proof.
  induction D1 as [| D1 IH q1]; destruct D2 as [| D2 q2]; simpl; try reflexivity.
  rewrite IH. destruct (uadd D2 D1) as [D |]; [rewrite qadd_comm |]; reflexivity.
Qed.

(** Addition is ASSOCIATIVE (on the partial operation). *)
Lemma uadd_assoc : forall D1 D2 D3 D12 D123,
  uadd D1 D2 = Some D12 ->
  uadd D12 D3 = Some D123 ->
  exists D23, uadd D2 D3 = Some D23 /\ uadd D1 D23 = Some D123.
Proof.
  induction D1 as [| D1 IH q1]; intros D2 D3 D12 D123 H12 H123.
  - (* D1 = UEmpty *)
    destruct D2 as [| D2 q2]; simpl in H12; try discriminate.
    injection H12 as <-.
    destruct D3 as [| D3 q3]; simpl in H123; try discriminate.
    injection H123 as <-. exists UEmpty. split; reflexivity.
  - (* D1 = USnoc D1 q1 *)
    destruct D2 as [| D2 q2]; simpl in H12; try discriminate.
    destruct (uadd D1 D2) as [d12 |] eqn:E12; try discriminate.
    injection H12 as <-.
    destruct D3 as [| D3 q3]; simpl in H123; try discriminate.
    destruct (uadd d12 D3) as [d123 |] eqn:E123; try discriminate.
    injection H123 as <-.
    destruct (IH D2 D3 d12 d123 E12 E123) as [d23 [E23 E1_23]].
    exists (USnoc d23 (qadd q2 q3)). split.
    + simpl. rewrite E23. reflexivity.
    + simpl. rewrite E1_23, qadd_assoc. reflexivity.
Qed.

(** [uzero] is a left identity for addition (on matching shapes). *)
Lemma uadd_zero_l : forall G D, ulen D = tlen G -> uadd (uzero G) D = Some D.
Proof.
  induction G as [| G IH a]; intros [| D q] Hlen; simpl in *; try discriminate.
  - reflexivity.
  - (* simpl already reduces `qadd Zero q` to `q` *)
    injection Hlen as Hlen. rewrite (IH D Hlen). reflexivity.
Qed.

(** Scaling distributes over addition:  q·(D1+D2) = q·D1 + q·D2. *)
Lemma uscale_add : forall q D1 D2 D,
  uadd D1 D2 = Some D ->
  uadd (uscale q D1) (uscale q D2) = Some (uscale q D).
Proof.
  intros q. induction D1 as [| D1 IH q1]; intros [| D2 q2] D H; simpl in *; try discriminate.
  - injection H as <-. reflexivity.
  - destruct (uadd D1 D2) as [d |] eqn:E; try discriminate.
    injection H as <-. simpl. rewrite (IH D2 d E), qmul_distrib_l. reflexivity.
Qed.
