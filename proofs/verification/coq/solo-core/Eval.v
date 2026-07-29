(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)
(*
 * A FUNCTIONAL one-step evaluator `step1 : tm -> option tm` for the solo core,
 * proved SOUND (and COMPLETE) with respect to the reference `step` RELATION of
 * SoloCore.v. The reference semantics is a relation (Prop), so it cannot be
 * extracted/run directly; `step1` is the executable mirror, and
 *
 *   step1_sound    : step1 t = Some t' -> step t t'     (every evaluator step
 *                                                        is sanctioned by spec)
 *   step1_complete : step t t' -> step1 t = Some t'      (the evaluator finds a
 *                                                        step whenever spec can)
 *
 * make it a faithful decision procedure for one CBV step. `step1` is extracted
 * (Extract.v) and differentially conformance-tested against the Rust evaluator
 * `my_qtt::step1` (conformance/eval/), closing impl ⇄ spec coupling #2
 * (interpreter adequacy vs Coq `step`) over the shared solo `tm` language.
 *
 * In CI (`_CoqProject` lists this file) `coqc` checks both theorems `Qed`.
 *)
From SoloCore Require Import SoloCore.
From SoloCore Require Import EchoMode.
Require Import Coq.Bool.Bool.

(* ---------- decidable value predicate ---------- *)

Fixpoint is_value (t : tm) : bool :=
  match t with
  | UnitT => true
  | Lam _ _ _ => true
  | With a b => is_value a && is_value b
  | Inl _ a => is_value a
  | Inr _ a => is_value a
  | MkEcho _ _ _ a => is_value a
  | Tensor a b => is_value a && is_value b
  | _ => false
  end.

Lemma is_value_value : forall t, is_value t = true -> value t.
Proof.
  induction t; simpl; intro H; try discriminate;
    try (apply andb_prop in H as [H1 H2]);
    constructor; auto.
Qed.

Lemma value_is_value : forall t, value t -> is_value t = true.
Proof.
  induction 1; simpl;
    repeat match goal with
           | [ H : is_value ?v = true |- context[is_value ?v] ] => rewrite H
           end;
    reflexivity.
Qed.

(* ---------- the functional one-step evaluator ---------- *)

Fixpoint step1 (t : tm) : option tm :=
  match t with
  | Var _ | UnitT | Lam _ _ _ => None
  | App t1 t2 =>
      match step1 t1 with
      | Some t1' => Some (App t1' t2)
      | None =>
          if is_value t1 then
            match step1 t2 with
            | Some t2' => Some (App t1 t2')
            | None =>
                if is_value t2
                then match t1 with
                     | Lam _ _ body => Some (subst0 t2 body)
                     | _ => None
                     end
                else None
            end
          else None
      end
  | With t1 t2 =>
      match step1 t1 with
      | Some t1' => Some (With t1' t2)
      | None =>
          if is_value t1
          then match step1 t2 with Some t2' => Some (With t1 t2') | None => None end
          else None
      end
  | Fst t0 =>
      match step1 t0 with
      | Some t0' => Some (Fst t0')
      | None => match t0 with
                | With v1 v2 => if is_value v1 && is_value v2 then Some v1 else None
                | _ => None
                end
      end
  | Snd t0 =>
      match step1 t0 with
      | Some t0' => Some (Snd t0')
      | None => match t0 with
                | With v1 v2 => if is_value v1 && is_value v2 then Some v2 else None
                | _ => None
                end
      end
  | Tensor t1 t2 =>
      match step1 t1 with
      | Some t1' => Some (Tensor t1' t2)
      | None =>
          if is_value t1
          then match step1 t2 with Some t2' => Some (Tensor t1 t2') | None => None end
          else None
      end
  | LetPair t1 t2 =>
      match step1 t1 with
      | Some t1' => Some (LetPair t1' t2)
      | None => match t1 with
                | Tensor v1 v2 =>
                    if is_value v1 && is_value v2 then Some (subst2 v1 v2 t2) else None
                | _ => None
                end
      end
  | Inl b t0 => match step1 t0 with Some t0' => Some (Inl b t0') | None => None end
  | Inr a t0 => match step1 t0 with Some t0' => Some (Inr a t0') | None => None end
  | Case s tL tR =>
      match step1 s with
      | Some s' => Some (Case s' tL tR)
      | None => match s with
                | Inl _ v => if is_value v then Some (subst0 v tL) else None
                | Inr _ v => if is_value v then Some (subst0 v tR) else None
                | _ => None
                end
      end
  | Let q t1 t2 =>
      match step1 t1 with
      | Some t1' => Some (Let q t1' t2)
      | None => if is_value t1 then Some (subst0 t1 t2) else None
      end
  | MkEcho m a b t0 =>
      match step1 t0 with Some t0' => Some (MkEcho m a b t0') | None => None end
  | Weaken t0 =>
      match step1 t0 with
      | Some t0' => Some (Weaken t0')
      | None => match t0 with
                | MkEcho Linear a b v => if is_value v then Some (MkEcho Affine a b v) else None
                | _ => None
                end
      end
  end.

(* values do not step (functionally) *)
Lemma value_no_step1 : forall t, value t -> step1 t = None.
Proof.
  induction 1; simpl;
    repeat match goal with
           | [ H : step1 ?t = None |- context[step1 ?t] ] => rewrite H
           | [ H : value ?v |- context[is_value ?v] ] => rewrite (value_is_value v H)
           end;
    reflexivity.
Qed.

(* ---------- SOUNDNESS: functional step is a relational step ---------- *)

(* Hint set: every `step` constructor + the value-decision soundness bridge.
   The proof below is name-INDEPENDENT (it never refers to a destruct-generated
   variable), so it is robust to Coq's argument-naming for `tm`. *)
#[local] Hint Constructors step : eval.
#[local] Hint Resolve is_value_value : eval.
(* `destruct (step1 s) eqn:` specialises the IH to `Some v = Some ? -> step s ?`;
   discharging the residual `Some v = Some v` needs reflexivity, which eauto does
   not try unaided. *)
#[local] Hint Extern 0 (@eq (option _) _ _) => reflexivity : eval.

Theorem step1_sound : forall t t', step1 t = Some t' -> step t t'.
Proof.
  induction t; simpl; intros t' H; try discriminate;
    (* peel every `match step1 _`, `if is_value _`, and redex `match _`
       scrutinee inside H, reducing the iota-redex each time so the loop
       makes progress and terminates *)
    repeat (match type of H with
            | context[match step1 ?s with _ => _ end] => destruct (step1 s) eqn:?; simpl in H
            | context[if ?b then _ else _]             => destruct b eqn:?; simpl in H
            | context[match ?x with _ => _ end]        => destruct x; simpl in H
            end);
    try discriminate; try (injection H as <-);
    repeat match goal with
           | [ E : is_value ?a && is_value ?b = true |- _ ] => apply andb_prop in E as [? ?]
           end;
    eauto 8 with eval.
Qed.

(* ---------- COMPLETENESS: relational step is found by the function ---------- *)

Theorem step1_complete : forall t t', step t t' -> step1 t = Some t'.
Proof.
  induction 1; simpl;
    repeat match goal with
           | [ IH : step1 ?t = Some _ |- context[step1 ?t] ] => rewrite IH
           | [ H : value ?v |- context[step1 ?v] ]  => rewrite (value_no_step1 v H)
           | [ H : value ?v |- context[is_value ?v] ] => rewrite (value_is_value v H)
           end;
    reflexivity.
Qed.
