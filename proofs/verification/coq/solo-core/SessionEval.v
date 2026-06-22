(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)
(*
 * A FUNCTIONAL session-configuration stepper `cstep1 : config -> option config`
 * for the binary fused `(νc)(P∣Q)` form of SessionPi.v, proved SOUND and
 * COMPLETE w.r.t. the reference `cstep` RELATION (S1.1b). The reference is a
 * relation (Prop) and so not extractable/runnable; `cstep1` is its executable
 * mirror — extracted (Extract.v) and differentially conformance-tested against
 * a Rust session runtime (`my_qtt::session`), closing impl ⇄ spec coupling #4
 * (session runtime refining `cstep`) for the binary fragment.
 *
 *   cstep1_sound    : cstep1 c = Some c' -> cstep c c'
 *   cstep1_complete : cstep c c' -> cstep1 c = Some c'
 *
 * In CI (`_CoqProject` lists this file) `coqc` checks both `Qed`.
 *)
From SoloCore Require Import SessionPi.

(* one synchronous communication step of the fused two-party config:
   a send meets the dual receive (payload substituted), or a select meets the
   dual branch (label looked up). Mirrors the 4 `cstep` constructors. *)
Definition cstep1 (c : config) : option config :=
  match c with
  | Conf (QSend v p) (QRecv q) => Some (Conf p (open_party v q))
  | Conf (QRecv q) (QSend v p) => Some (Conf (open_party v q) p)
  | Conf (QSel l p) (QBra bs)  =>
      match pget l bs with Some q => Some (Conf p q) | None => None end
  | Conf (QBra bs) (QSel l p)  =>
      match pget l bs with Some q => Some (Conf q p) | None => None end
  | _ => None
  end.

#[local] Hint Constructors cstep : sess.

Theorem cstep1_sound : forall c c', cstep1 c = Some c' -> cstep c c'.
Proof.
  intros [p1 p2] c' H; destruct p1; destruct p2; simpl in H;
    repeat (match type of H with
            | context[match pget ?l ?bs with _ => _ end] => destruct (pget l bs) eqn:?
            end);
    try discriminate; injection H as <-; eauto with sess.
Qed.

Theorem cstep1_complete : forall c c', cstep c c' -> cstep1 c = Some c'.
Proof.
  induction 1; simpl; try reflexivity;
    match goal with [ H : pget _ _ = _ |- _ ] => rewrite H end; reflexivity.
Qed.
