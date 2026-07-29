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

Require Import Coq.Arith.PeanoNat.

(* ===== global-type stepper (coupling #4, global layer) =====

   `gstep` (S3c.3) is NONDETERMINISTIC at `GBra` (any offered label may be
   chosen), so the honest adequacy is SOUNDNESS + PROGRESS rather than a strong
   functional-determinism completeness: `gstep1` commits to the FIRST branch and
   finds a step exactly when the relation can step. *)

Definition gstep1 (G : gty) : option gty :=
  match G with
  | GMsg _ _ _ G'            => Some G'
  | GBra _ _ (GBcons _ G' _) => Some G'   (* commit to the head branch *)
  | _                        => None
  end.

#[local] Hint Constructors gstep : sess.

Theorem gstep1_sound : forall G G', gstep1 G = Some G' -> gstep G G'.
Proof.
  intros [| p q t G0 | p q bs | G0 | n] G' H; simpl in H; try discriminate.
  - injection H as <-; apply GStep_Msg.
  - destruct bs as [| l G0 rest]; simpl in H; try discriminate.
    injection H as <-. apply GStep_Bra with (l := l). simpl. rewrite Nat.eqb_refl. reflexivity.
Qed.

(* PROGRESS: whenever the relation can step, `gstep1` returns some reduct.
   (The strong form `gstep G G' -> gstep1 G = Some G'` is FALSE for `GBra` with
   more than one branch — the relation may pick a non-head label.) *)
Theorem gstep1_complete : forall G G', gstep G G' -> exists G'', gstep1 G = Some G''.
Proof.
  intros G G' H; destruct H as [p q t G0 | p q bs l G0 Hget].
  - exists G0; reflexivity.
  - destruct bs as [| k Gk rest]; simpl in Hget; try discriminate.
    exists Gk; reflexivity.
Qed.

Require Import Coq.Lists.List.
Import ListNotations.

(* ===== n-ary located stepper (coupling #4, multiparty layer) =====

   `nstep` (S2/S3) is NONDETERMINISTIC: any communicating (p,q) pair may fire.
   `nstep1` commits to the FIRST sender/selector role (left-to-right in the
   assignment) paired with its FIRST dual partner; adequacy is SOUNDNESS +
   PROGRESS. Parties are always re-fetched through `ra_get` (the sanctioned
   lookup), so duplicate role entries never make the stepper act on a stale
   party. *)

(* first q (with q <> r) whose canonical party is QRecv; yields (q, open_party v Qc) *)
Fixpoint find_recv (ra cands : role_assignment) (r : role) (v : val)
  : option (role * party) :=
  match cands with
  | [] => None
  | (q, _) :: rest =>
      if Nat.eqb q r then find_recv ra rest r v
      else match ra_get ra q with
           | Some (QRecv Qc) => Some (q, open_party v Qc)
           | _ => find_recv ra rest r v
           end
  end.

(* first q (with q <> r) whose canonical party is QBra offering label l; yields (q, Q) *)
Fixpoint find_bra (ra cands : role_assignment) (r : role) (l : nat)
  : option (role * party) :=
  match cands with
  | [] => None
  | (q, _) :: rest =>
      if Nat.eqb q r then find_bra ra rest r l
      else match ra_get ra q with
           | Some (QBra bs) =>
               match pget l bs with
               | Some Q => Some (q, Q)
               | None => find_bra ra rest r l
               end
           | _ => find_bra ra rest r l
           end
  end.

Definition try_role (ra : role_assignment) (r : role) (P : party)
  : option role_assignment :=
  match P with
  | QSend v cont =>
      match find_recv ra ra r v with
      | Some (q, qres) => Some (ra_set (ra_set ra r cont) q qres)
      | None => None
      end
  | QSel l cont =>
      match find_bra ra ra r l with
      | Some (q, qres) => Some (ra_set (ra_set ra r cont) q qres)
      | None => None
      end
  | _ => None
  end.

Fixpoint nstep1_go (ra cands : role_assignment) : option role_assignment :=
  match cands with
  | [] => None
  | (r, _) :: rest =>
      match ra_get ra r with
      | Some P => match try_role ra r P with
                  | Some ra' => Some ra'
                  | None => nstep1_go ra rest
                  end
      | None => nstep1_go ra rest
      end
  end.

Definition nstep1 (ra : role_assignment) : option role_assignment := nstep1_go ra ra.

#[local] Hint Constructors nstep : sess.

(* ----- soundness ----- *)

Lemma find_recv_sound : forall ra cands r v q qres,
  find_recv ra cands r v = Some (q, qres) ->
  r <> q /\ exists Qc, ra_get ra q = Some (QRecv Qc) /\ qres = open_party v Qc.
Proof.
  induction cands as [| [q0 P0] rest IH]; intros r v q qres H; simpl in H.
  - discriminate.
  - destruct (Nat.eqb q0 r) eqn:E.
    + apply IH in H; exact H.
    + destruct (ra_get ra q0) as [P|] eqn:Eg; [| apply IH in H; exact H].
      destruct P; try (apply IH in H; exact H).
      injection H as <- <-. split.
      * apply Nat.eqb_neq in E. congruence.
      * eexists; split; [ exact Eg | reflexivity ].
Qed.

Lemma find_bra_sound : forall ra cands r l q qres,
  find_bra ra cands r l = Some (q, qres) ->
  r <> q /\ exists bs, ra_get ra q = Some (QBra bs) /\ pget l bs = Some qres.
Proof.
  induction cands as [| [q0 P0] rest IH]; intros r l q qres H; simpl in H.
  - discriminate.
  - destruct (Nat.eqb q0 r) eqn:E.
    + apply IH in H; exact H.
    + destruct (ra_get ra q0) as [P|] eqn:Eg; [| apply IH in H; exact H].
      destruct P as [| ? ? | ? | ? ? | bs0]; try (apply IH in H; exact H).
      destruct (pget l bs0) as [Q|] eqn:Ep; [| apply IH in H; exact H].
      injection H as <- <-. split.
      * apply Nat.eqb_neq in E. congruence.
      * eexists; split; [ exact Eg | exact Ep ].
Qed.

Lemma try_role_sound : forall ra r P ra',
  ra_get ra r = Some P -> try_role ra r P = Some ra' -> nstep ra ra'.
Proof.
  intros ra r P ra' Hr H. destruct P as [| v cont | Qc | l cont | bs]; simpl in H; try discriminate.
  - destruct (find_recv ra ra r v) as [[q qres]|] eqn:Ef; try discriminate.
    injection H as <-. apply find_recv_sound in Ef.
    destruct Ef as (Hrq & Qc & Hq & ->). eapply NStep_Comm; eauto.
  - destruct (find_bra ra ra r l) as [[q qres]|] eqn:Ef; try discriminate.
    injection H as <-. apply find_bra_sound in Ef.
    destruct Ef as (Hrq & bs & Hq & Hpg). eapply NStep_Sel; eauto.
Qed.

Lemma nstep1_go_sound : forall ra cands ra',
  nstep1_go ra cands = Some ra' -> nstep ra ra'.
Proof.
  induction cands as [| [r P0] rest IH]; intros ra' H; simpl in H.
  - discriminate.
  - destruct (ra_get ra r) as [P|] eqn:Hr; [| apply IH; exact H].
    destruct (try_role ra r P) as [ra''|] eqn:Et; [| apply IH; exact H].
    injection H as <-. eapply try_role_sound; eauto.
Qed.

Theorem nstep1_sound : forall ra ra', nstep1 ra = Some ra' -> nstep ra ra'.
Proof. intros ra ra'. apply nstep1_go_sound. Qed.

(* ----- progress: nstep1 finds a step whenever the relation can step ----- *)

Lemma find_recv_steps : forall ra cands r v q Qc P,
  In (q, P) cands -> r <> q -> ra_get ra q = Some (QRecv Qc) ->
  find_recv ra cands r v <> None.
Proof.
  induction cands as [| [q0 P0] rest IH]; intros r v q Qc P Hin Hrq Hq; simpl.
  - inversion Hin.
  - destruct Hin as [Heq | Hin].
    + inversion Heq; subst q0 P0.
      destruct (Nat.eqb q r) eqn:E.
      * apply Nat.eqb_eq in E; subst; contradiction.
      * rewrite Hq; discriminate.
    + destruct (Nat.eqb q0 r) eqn:E; [ eapply IH; eauto |].
      destruct (ra_get ra q0) as [P1|] eqn:Eg; [| eapply IH; eauto].
      destruct P1; try (eapply IH; eauto); discriminate.
Qed.

Lemma find_bra_steps : forall ra cands r l q bs Q P,
  In (q, P) cands -> r <> q -> ra_get ra q = Some (QBra bs) -> pget l bs = Some Q ->
  find_bra ra cands r l <> None.
Proof.
  induction cands as [| [q0 P0] rest IH]; intros r l q bs Q P Hin Hrq Hq Hpg; simpl.
  - inversion Hin.
  - destruct Hin as [Heq | Hin].
    + inversion Heq; subst q0 P0.
      destruct (Nat.eqb q r) eqn:E.
      * apply Nat.eqb_eq in E; subst; contradiction.
      * rewrite Hq, Hpg; discriminate.
    + destruct (Nat.eqb q0 r) eqn:E; [ eapply IH; eauto |].
      destruct (ra_get ra q0) as [P1|] eqn:Eg; [| eapply IH; eauto].
      destruct P1 as [| ? ? | ? | ? ? | bs0]; try (eapply IH; eauto).
      destruct (pget l bs0) eqn:Ep; [ discriminate | eapply IH; eauto ].
Qed.

Lemma try_role_send_steps : forall ra p v P q Q,
  p <> q -> ra_get ra q = Some (QRecv Q) -> try_role ra p (QSend v P) <> None.
Proof.
  intros ra p v P q Q Hpq Hq. simpl.
  destruct (find_recv ra ra p v) as [[q' r']|] eqn:E; [ discriminate |].
  exfalso. revert E. eapply find_recv_steps with (q := q) (Qc := Q) (P := QRecv Q); eauto.
  apply ra_get_in; exact Hq.
Qed.

Lemma try_role_sel_steps : forall ra p l P q bs Q,
  p <> q -> ra_get ra q = Some (QBra bs) -> pget l bs = Some Q ->
  try_role ra p (QSel l P) <> None.
Proof.
  intros ra p l P q bs Q Hpq Hq Hpg. simpl.
  destruct (find_bra ra ra p l) as [[q' r']|] eqn:E; [ discriminate |].
  exfalso. revert E. eapply find_bra_steps with (q := q) (bs := bs) (Q := Q) (P := QBra bs); eauto.
  apply ra_get_in; exact Hq.
Qed.

Lemma nstep1_go_steps : forall ra cands r P,
  In r (map fst cands) -> ra_get ra r = Some P -> try_role ra r P <> None ->
  nstep1_go ra cands <> None.
Proof.
  induction cands as [| [r0 Q0] rest IH]; intros r P Hin Hr Hstep; simpl in Hin |- *.
  - contradiction.
  - destruct (ra_get ra r0) as [P0|] eqn:Hr0.
    + destruct (try_role ra r0 P0) eqn:Et0; [ discriminate |].
      destruct Hin as [Heq | Hin].
      * subst r0. rewrite Hr0 in Hr. injection Hr as Hpp. congruence.
      * eapply IH; eauto.
    + destruct Hin as [Heq | Hin].
      * subst r0. rewrite Hr0 in Hr. discriminate.
      * eapply IH; eauto.
Qed.

Theorem nstep1_complete : forall ra ra', nstep ra ra' -> exists ra'', nstep1 ra = Some ra''.
Proof.
  intros ra ra' H.
  assert (Hne : nstep1 ra <> None).
  { unfold nstep1.
    destruct H as [ra p q v P Q Hpq Hp Hq | ra p q l P bs Q Hpq Hp Hq Hpg].
    - eapply nstep1_go_steps with (r := p) (P := QSend v P); [| exact Hp |].
      + apply in_map_iff. exists (p, QSend v P); split; [reflexivity| apply ra_get_in; exact Hp].
      + eapply try_role_send_steps; eauto.
    - eapply nstep1_go_steps with (r := p) (P := QSel l P); [| exact Hp |].
      + apply in_map_iff. exists (p, QSel l P); split; [reflexivity| apply ra_get_in; exact Hp].
      + eapply try_role_sel_steps; eauto. }
  destruct (nstep1 ra) as [ra''|]; [ exists ra''; reflexivity | congruence ].
Qed.
