(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ============================================================ *)
(* my-lang ENSEMBLE session-pi core  (axis-2 STRUCTURE).         *)
(* Rung S1.0: definitions + executable witnesses.                *)
(*                                                              *)
(* This is a NEW development, deliberately NOT an extension of   *)
(* SoloCore.v: processes are a different term language from the  *)
(* lambda/QTT solo core, so nothing carrier-bearing is imported. *)
(* We reuse only the METHODOLOGY (Coq-canonical, axiom-free Qed, *)
(* Print Assumptions closed, executable Example/Compute          *)
(* witnesses), per proofs/ALIGNMENT-PLAN.md D1/D2.               *)
(*                                                              *)
(* Source-of-truth paper calculus:                               *)
(*   proofs/duet/session-types/formal-system.md  (binary)        *)
(*   proofs/ensemble/agent-calculus/pi-calculus-foundation.md.   *)
(*                                                              *)
(* FAITHFULNESS FENCE (honest scope — the M1 lesson).            *)
(*  IN  (S1.0): synchronous binary session pi over POLARISED     *)
(*       endpoints (Pos/Neg over a channel name), session types  *)
(*       send / recv / end, a computed duality involution, a     *)
(*       linear channel-typing judgement with context splitting, *)
(*       small-step reduction for the communication redex under  *)
(*       par/res contexts, and base-value payloads.              *)
(*  OUT (deferred): name-passing / channel mobility (payloads    *)
(*       are BASE values only -- so no scope extrusion and no     *)
(*       capture is possible), structural congruence as a        *)
(*       relation (S1.3c), internal/external choice select/branch*)
(*       (S1.3a), replication !P, mismatch, mu-recursion (S1.3b),*)
(*       bisimulation, the agent/delegate/await/orchestrate/     *)
(*       refine AI primitives, and multiparty global types G     *)
(*       with projection G|^p (the S2 = duet-by-projection hook). *)
(*  NOTE: because no name is ever substituted into a process     *)
(*       (only base payload VALUES are), explicit channel names   *)
(*       are capture-safe here; de Bruijn channels + alpha are    *)
(*       only needed once mobility lands, which is post-S1.       *)
(*                                                              *)
(* Echo-types: NOT-RELEVANT (audit). echo-types is axis-3         *)
(* MODALITY; this is axis-2 STRUCTURE. AXIS-ARCHITECTURE.md sec.3 *)
(* mandates the axes compose but must NOT be collapsed, so no     *)
(* EchoMode/EchoResidue artefact enters this file.               *)
(* ============================================================ *)

Require Import PeanoNat.
Require Import Lia.
Require Import List.
Import ListNotations.

(* ================= Payload value layer ================= *)
(* Payloads are BASE values only (no channels): this is what     *)
(* makes the whole development capture-safe without de Bruijn     *)
(* channel binders.                                              *)

Inductive vty : Type :=
| VTUnit : vty
| VTBool : vty
| VTNat  : vty.

Inductive val : Type :=
| VVar  : nat -> val          (* de Bruijn payload variable (receive binder) *)
| VUnit : val
| VBool : bool -> val
| VNat  : nat -> val.

(* Payload typing under a value context [G : list vty] (de Bruijn). *)
Inductive vtype : list vty -> val -> vty -> Prop :=
| VT_Var  : forall G k t, nth_error G k = Some t -> vtype G (VVar k) t
| VT_Unit : forall G, vtype G VUnit VTUnit
| VT_Bool : forall G b, vtype G (VBool b) VTBool
| VT_Nat  : forall G k, vtype G (VNat k) VTNat.

(* ================= Session type layer ================= *)

(* Labelled choice branches are a DEDICATED mutual inductive        *)
(* [sbranch] (not [list (nat*sty)]) so that [dual] below is a       *)
(* well-guarded mutual fixpoint: Coq's guard checker accepts a      *)
(* recursive call into the [sty] field of [SBcons] but NOT one      *)
(* buried under [List.map] / a list pair-projection.                *)
Inductive sty : Type :=
| SEnd    : sty                          (* end *)
| SSend   : vty -> sty -> sty            (* !t.s  : send a t, continue as s *)
| SRecv   : vty -> sty -> sty            (* ?t.s  : receive a t, continue as s *)
| SSelect : sbranch -> sty               (* +{l_i:s_i} : internal choice (S1.3a) *)
| SBranch : sbranch -> sty               (* &{l_i:s_i} : external choice (S1.3a) *)
| SVar    : nat -> sty                   (* mu-bound type variable (de Bruijn) (S1.3b) *)
| SMu     : sty -> sty                   (* mu a.s : equi-recursive session    (S1.3b) *)
with sbranch : Type :=
| SBnil  : sbranch
| SBcons : nat -> sty -> sbranch -> sbranch.

(* Duality.  Choice dualises pointwise over the branch list,       *)
(* preserving labels: select (internal) <-> branch (external),     *)
(* the paper's dual(+{l_i:S_i}) = &{l_i:dual S_i}.  Recursion       *)
(* variables are UNPOLARISED, so dual passes through SVar/SMu — a   *)
(* fence: this is correct ONLY because there are no polarised type  *)
(* variables (standard for binary session duality).                *)
Fixpoint dual (s : sty) : sty :=
  match s with
  | SEnd       => SEnd
  | SSend t k  => SRecv t (dual k)
  | SRecv t k  => SSend t (dual k)
  | SSelect bs => SBranch (dual_br bs)
  | SBranch bs => SSelect (dual_br bs)
  | SVar n     => SVar n
  | SMu s0     => SMu (dual s0)
  end
with dual_br (bs : sbranch) : sbranch :=
  match bs with
  | SBnil           => SBnil
  | SBcons l s rest => SBcons l (dual s) (dual_br rest)
  end.

Lemma dual_involutive : forall s, dual (dual s) = s.
Proof.
  fix IH 1.
  intros [ | t k | t k | bs | bs | n | s0 ]; simpl.
  - reflexivity.
  - rewrite IH; reflexivity.
  - rewrite IH; reflexivity.
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - reflexivity.
  - rewrite IH; reflexivity.
Qed.

(* Session-branch lookup (first match), used by the choice layer    *)
(* (S1.3a).  [bget] is FUNCTIONAL — a selected label has at most    *)
(* one continuation — so no NoDup side-condition is needed for      *)
(* subject reduction.                                               *)
Fixpoint bget (l : nat) (bs : sbranch) : option sty :=
  match bs with
  | SBnil           => None
  | SBcons k s rest => if Nat.eqb l k then Some s else bget l rest
  end.

(* bget commutes with the dual of a branch list (forward + back):  *)
(* the dual of the looked-up continuation is what you get by       *)
(* looking up in the dualised list.  Load-bearing for choice SR.   *)
Lemma bget_dual_br : forall l bs s,
  bget l bs = Some s -> bget l (dual_br bs) = Some (dual s).
Proof.
  intros l bs; induction bs as [ | k s0 rest IH ]; simpl; intros s H.
  - discriminate.
  - destruct (Nat.eqb l k) eqn:E.
    + injection H as ->. reflexivity.
    + apply IH. exact H.
Qed.

Lemma bget_dual_br_inv : forall l bs sl,
  bget l (dual_br bs) = Some sl -> exists sB, bget l bs = Some sB /\ sl = dual sB.
Proof.
  intros l bs; induction bs as [ | k s0 rest IH ]; simpl; intros sl H.
  - discriminate.
  - destruct (Nat.eqb l k) eqn:E.
    + injection H as <-. exists s0. split; reflexivity.
    + apply IH. exact H.
Qed.

(* ================= Endpoints (polarity) ================= *)

Inductive polarity : Type := Pos | Neg.

Definition co (p : polarity) : polarity :=
  match p with Pos => Neg | Neg => Pos end.

Lemma co_involutive : forall p, co (co p) = p.
Proof. destruct p; reflexivity. Qed.

(* ================= Process syntax ================= *)
(* Channel names are explicit [nat] (capture-safe, see fence).   *)
(* PRecv binds payload de Bruijn 0 in its body.                  *)

Inductive proc : Type :=
| PNil  : proc                                   (* 0 *)
| PSend : polarity -> nat -> val -> proc -> proc (* (p,n)!v . P *)
| PRecv : polarity -> nat -> proc -> proc        (* (p,n)?(x) . P *)
| PPar  : proc -> proc -> proc                   (* P | Q *)
| PRes  : nat -> sty -> proc -> proc.            (* (nu n : s) P, Pos:s Neg:dual s *)

(* ----- payload substitution (de Bruijn, capture-safe) ----- *)

Definition vlift (c : nat) (v : val) : val :=
  match v with
  | VVar k => if Nat.ltb k c then VVar k else VVar (S k)
  | _ => v
  end.

Definition vsubst (c : nat) (u : val) (v : val) : val :=
  match v with
  | VVar k =>
      match Nat.compare k c with
      | Lt => VVar k
      | Eq => u
      | Gt => VVar (Nat.pred k)
      end
  | _ => v
  end.

Fixpoint psubst (c : nat) (u : val) (P : proc) : proc :=
  match P with
  | PNil          => PNil
  | PSend p n v Q => PSend p n (vsubst c u v) (psubst c u Q)
  | PRecv p n Q   => PRecv p n (psubst (S c) (vlift 0 u) Q)
  | PPar Q R      => PPar (psubst c u Q) (psubst c u R)
  | PRes n s Q    => PRes n s (psubst c u Q)
  end.

Definition open (u : val) (P : proc) : proc := psubst 0 u P.

(* ================= Channel typing ================= *)
(* Context: a linear assignment of session types to endpoints.   *)
(* Each entry is (polarity, channel-name, session-type).         *)

Definition cctx := list (polarity * nat * sty).

(* A context is [ended] when every endpoint is finished.         *)
Definition ended (D : cctx) : Prop :=
  Forall (fun e => snd e = SEnd) D.

(* Linear context splitting: every endpoint goes to exactly one  *)
(* side. [csplit D D1 D2] reads D = D1 (+) D2.                    *)
Inductive csplit : cctx -> cctx -> cctx -> Prop :=
| Csp_nil : csplit [] [] []
| Csp_l   : forall e D D1 D2, csplit D D1 D2 -> csplit (e :: D) (e :: D1) D2
| Csp_r   : forall e D D1 D2, csplit D D1 D2 -> csplit (e :: D) D1 (e :: D2).

(* Process typing: [wt G D P] -- under payload context G, process *)
(* P uses exactly the channel endpoints in D, linearly.          *)
Inductive wt : list vty -> cctx -> proc -> Prop :=
| WT_Nil  : forall G D, ended D -> wt G D PNil
| WT_Par  : forall G D D1 D2 P Q,
    csplit D D1 D2 -> wt G D1 P -> wt G D2 Q -> wt G D (PPar P Q)
| WT_Send : forall G D Drest p n t v k P,
    csplit D [(p, n, SSend t k)] Drest ->
    vtype G v t ->
    wt G ((p, n, k) :: Drest) P ->
    wt G D (PSend p n v P)
| WT_Recv : forall G D Drest p n t k P,
    csplit D [(p, n, SRecv t k)] Drest ->
    wt (t :: G) ((p, n, k) :: Drest) P ->
    wt G D (PRecv p n P)
| WT_Res  : forall G D n s P,
    wt G ((Pos, n, s) :: (Neg, n, dual s) :: D) P ->
    wt G D (PRes n s P).

(* ================= Reduction (small-step) ================= *)
(* Communication is synchronous on co-endpoints (p,n)/(co p,n).  *)
(* Both adjacency orders are given directly since structural     *)
(* congruence (Par-Comm) is deferred to S1.3c.                   *)

Inductive step : proc -> proc -> Prop :=
| St_Comm : forall p n v P Q,
    step (PPar (PSend p n v P) (PRecv (co p) n Q))
         (PPar P (open v Q))
| St_CommR : forall p n v P Q,
    step (PPar (PRecv (co p) n Q) (PSend p n v P))
         (PPar (open v Q) P)
| St_Par  : forall P P' Q, step P P' -> step (PPar P Q) (PPar P' Q)
| St_ParR : forall P Q Q', step Q Q' -> step (PPar P Q) (PPar P Q')
| St_Res  : forall n s P P', step P P' -> step (PRes n s P) (PRes n s P').

(* ================= Executable witnesses ================= *)

(* (1) Duality computes, and is involutive on a 2-step protocol. *)
Example dual_protocol :
  dual (SSend VTNat (SRecv VTBool SEnd)) = SRecv VTNat (SSend VTBool SEnd).
Proof. reflexivity. Qed.

Example dual_protocol_invol :
  dual (dual (SSend VTNat (SRecv VTBool SEnd)))
  = SSend VTNat (SRecv VTBool SEnd).
Proof. apply dual_involutive. Qed.

(* (2) A well-typed ping-pong: channel 0 carries one Nat then     *)
(* both endpoints end. The whole system closes under nu.          *)
Definition sproto : sty := SSend VTNat SEnd.
Definition sP : proc := PSend Pos 0 (VNat 5) PNil.   (* sender side *)
Definition sQ : proc := PRecv Neg 0 PNil.            (* receiver side *)
Definition sys : proc := PRes 0 sproto (PPar sP sQ).

Example wt_pingpong : wt [] [] sys.
Proof.
  unfold sys, sproto, sP, sQ.
  apply WT_Res. simpl dual.
  eapply WT_Par.
  - apply Csp_l. apply Csp_r. apply Csp_nil.
  - eapply WT_Send.
    + apply Csp_l. apply Csp_nil.
    + apply VT_Nat.
    + apply WT_Nil. repeat constructor.
  - eapply WT_Recv.
    + apply Csp_l. apply Csp_nil.
    + apply WT_Nil. repeat constructor.
Qed.

(* (3) The ping-pong actually reduces: the Nat is communicated     *)
(* and both sides become 0.  open (VNat 5) PNil computes to PNil. *)
Example step_pingpong :
  step (PPar sP sQ) (PPar PNil PNil).
Proof. unfold sP, sQ. apply St_Comm. Qed.

(* And under the restriction, the whole system takes a step.       *)
Example step_sys :
  step sys (PRes 0 sproto (PPar PNil PNil)).
Proof. unfold sys, sP, sQ. apply St_Res. apply St_Comm. Qed.

(* ============================================================ *)
(* S1.1a — value-substitution + communication-redex preservation *)
(*                                                              *)
(* The load-bearing core of subject reduction, isolated as two   *)
(* axiom-free lemmas. These are architecture-INVARIANT (they act *)
(* at the payload / PPar level and never touch the ν binder), so *)
(* they stand regardless of how the full closed-system           *)
(* subject_reduction (S1.1b) resolves the ν re-typing question.  *)
(*                                                              *)
(* Because payloads are BASE values, the substitution lemma is   *)
(* FIRST-ORDER — strictly easier than the solo core's `ht_subst` *)
(* (no channel-context splitting inside the substitution).       *)
(* ============================================================ *)

(* Shift every payload variable up by [k]. Sound to apply with no *)
(* cutoff when the value's variables all point into the suffix    *)
(* context it is typed against (the only use below).              *)
Definition vshift (k : nat) (v : val) : val :=
  match v with VVar j => VVar (j + k) | _ => v end.

Lemma vshift_0 : forall v, vshift 0 v = v.
Proof. destruct v; simpl; try reflexivity. f_equal. apply Nat.add_0_r. Qed.

Lemma vlift0_vshift : forall k v, vlift 0 (vshift k v) = vshift (S k) v.
Proof.
  destruct v; simpl; try reflexivity.
  f_equal. rewrite Nat.add_succ_r. reflexivity.
Qed.

(* Value typing is stable under prefixing the context with G'     *)
(* (de Bruijn indices shift up by |G'|).                          *)
Lemma vtype_shift_app : forall G' G2 v t,
  vtype G2 v t -> vtype (G' ++ G2) (vshift (length G') v) t.
Proof.
  intros G' G2 v t H. inversion H; subst; simpl; try constructor.
  rewrite nth_error_app2 by apply Nat.le_add_l.
  rewrite Nat.add_sub. assumption.
Qed.

(* Inversion helper: a typed payload variable is a context lookup. *)
Lemma vtype_var_inv : forall G n t, vtype G (VVar n) t -> nth_error G n = Some t.
Proof. intros G n t H. inversion H; subst; assumption. Qed.

(* Substituting a value of the bound type into a value preserves   *)
(* value typing (the VVar bookkeeping under vsubst).               *)
Lemma vtype_subst : forall w G1 G2 t t' v,
  vtype (G1 ++ t :: G2) w t' ->
  vtype G2 v t ->
  vtype (G1 ++ G2) (vsubst (length G1) (vshift (length G1) v) w) t'.
Proof.
  intros w G1 G2 t t' v Hw Hv.
  destruct w; simpl; [ | inversion Hw; subst; constructor
                       | inversion Hw; subst; constructor
                       | inversion Hw; subst; constructor ].
  (* VVar n : decide n vs |G1| *)
  apply vtype_var_inv in Hw.
  destruct (Nat.compare n (length G1)) eqn:E.
  - (* Eq : n = |G1|, so t' = t; substitute v (shifted) *)
    apply Nat.compare_eq in E. subst n.
    rewrite nth_error_app2 in Hw by apply Nat.le_refl.
    rewrite Nat.sub_diag in Hw. simpl in Hw. inversion Hw; subst t'.
    apply vtype_shift_app. assumption.
  - (* Lt : n < |G1|, lookup stays in G1 *)
    apply Nat.compare_lt_iff in E.
    apply VT_Var. rewrite nth_error_app1 in Hw by assumption.
    rewrite nth_error_app1 by assumption. assumption.
  - (* Gt : n > |G1|, drop the removed slot *)
    apply Nat.compare_gt_iff in E.
    destruct n as [|n']; [ lia | ].
    apply VT_Var. simpl.
    rewrite nth_error_app2 in Hw by lia.
    rewrite nth_error_app2 by lia.
    replace (S n' - length G1) with (S (n' - length G1)) in Hw by lia.
    simpl in Hw. exact Hw.
Qed.

(* Per-constructor inversion lemmas (named cleanly, so the          *)
(* substitution induction never depends on auto-generated names).   *)
Lemma wt_nil_inv : forall G D, wt G D PNil -> ended D.
Proof. intros G D H; inversion H; subst; assumption. Qed.

Lemma wt_send_inv : forall G D p n v P, wt G D (PSend p n v P) ->
  exists Drest t k, csplit D [(p,n,SSend t k)] Drest
                 /\ vtype G v t /\ wt G ((p,n,k)::Drest) P.
Proof. intros G D p n v P H. inversion H; subst. do 3 eexists; repeat split; eassumption. Qed.

Lemma wt_recv_inv : forall G D p n P, wt G D (PRecv p n P) ->
  exists Drest t k, csplit D [(p,n,SRecv t k)] Drest
                 /\ wt (t::G) ((p,n,k)::Drest) P.
Proof. intros G D p n P H. inversion H; subst. do 3 eexists; repeat split; eassumption. Qed.

Lemma wt_par_inv : forall G D P Q, wt G D (PPar P Q) ->
  exists D1 D2, csplit D D1 D2 /\ wt G D1 P /\ wt G D2 Q.
Proof. intros G D P Q H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

Lemma wt_res_inv : forall G D n s P, wt G D (PRes n s P) ->
  wt G ((Pos,n,s)::(Neg,n,dual s)::D) P.
Proof. intros G D n s P H. inversion H; subst; assumption. Qed.

(* The value-substitution lemma: substituting a value of type [t]  *)
(* for the bound payload variable preserves process typing. The    *)
(* general (binder-depth |G1|) form is needed so the induction can *)
(* descend under PRecv.                                            *)
Lemma wt_subst : forall Q G1 G2 D t v,
  wt (G1 ++ t :: G2) D Q ->
  vtype G2 v t ->
  wt (G1 ++ G2) D (psubst (length G1) (vshift (length G1) v) Q).
Proof.
  induction Q as [ | pol ch w Qc IHQ | pol ch Qc IHQ | Qa IHa Qb IHb | ch s Qc IHQ ];
    intros G1 G2 D t v Hwt Hv; simpl.
  - (* PNil *)
    apply wt_nil_inv in Hwt. apply WT_Nil; assumption.
  - (* PSend *)
    apply wt_send_inv in Hwt. destruct Hwt as (Drest & ts & k & Hsp & Hvt & Hcont).
    eapply WT_Send.
    + eassumption.
    + eapply vtype_subst; eassumption.
    + apply IHQ with (t:=t); assumption.
  - (* PRecv : descend, |G1| grows by one *)
    apply wt_recv_inv in Hwt. destruct Hwt as (Drest & tr & k & Hsp & Hcont).
    eapply WT_Recv; [ eassumption | ].
    rewrite vlift0_vshift.
    apply (IHQ (tr :: G1) G2 ((pol, ch, k) :: Drest) t v); assumption.
  - (* PPar *)
    apply wt_par_inv in Hwt. destruct Hwt as (D1 & D2 & Hsp & HP & HQ).
    eapply WT_Par.
    + eassumption.
    + apply IHa with (t:=t); assumption.
    + apply IHb with (t:=t); assumption.
  - (* PRes : channel binder, value context untouched *)
    apply wt_res_inv in Hwt.
    apply WT_Res. apply IHQ with (t:=t); assumption.
Qed.

(* Substituting at the outermost binder (the comm-redex shape).     *)
Corollary wt_subst0 : forall G D Q t v,
  wt (t :: G) D Q -> vtype G v t -> wt G D (open v Q).
Proof.
  intros G D Q t v Hwt Hv.
  pose proof (wt_subst Q [] G D t v Hwt Hv) as HH.
  simpl in HH. rewrite vshift_0 in HH. unfold open. exact HH.
Qed.

(* csplit lemmas: a context is the disjoint append of its halves.   *)
Lemma csplit_nil_l : forall B, csplit B [] B.
Proof. induction B; [ apply Csp_nil | apply Csp_r; assumption ]. Qed.

Lemma csplit_app : forall A B, csplit (A ++ B) A B.
Proof. induction A; intros; simpl; [ apply csplit_nil_l | apply Csp_l; apply IHA ]. Qed.

(* ----- communication-redex preservation ----- *)
(* Given a sender continuing at [k] and a receiver that — by        *)
(* DUALITY (exactly what holds for a ν-bound channel) — expects the *)
(* sent type [t] and continues at [dual k], the reduct of the comm  *)
(* redex is well-typed at the session-ADVANCED context. This is the *)
(* heart of subject reduction for R-Comm.                           *)
Theorem sr_comm : forall G DP DQ p n t k v P Q,
  vtype G v t ->
  wt G ((p, n, k) :: DP) P ->
  wt (t :: G) ((co p, n, dual k) :: DQ) Q ->
  wt G (((p, n, k) :: DP) ++ ((co p, n, dual k) :: DQ))
       (PPar P (open v Q)).
Proof.
  intros G DP DQ p n t k v P Q Hv HP HQ.
  eapply WT_Par.
  - apply csplit_app.
  - exact HP.
  - eapply wt_subst0; eassumption.
Qed.

(* ============================================================ *)
(* S1.1b — FULL closed-system subject reduction, via the fused   *)
(* two-party (νc)(P∣Q) form (the duet T-Session primitive).      *)
(*                                                              *)
(* The general open `proc` calculus above pins a ν's protocol on *)
(* the `PRes n s` binder, which a communication under the ν must *)
(* advance — so the closed theorem cannot hold with Δ unchanged  *)
(* in that presentation. The standard fix (chosen by the owner)  *)
(* is the FUSED node `(νc)(P∣Q)`: a configuration of exactly the *)
(* two parties of ONE session, whose shared protocol advances    *)
(* LOCALLY on each communication. The free context never changes *)
(* (there is none — it is a closed two-party system), so subject *)
(* reduction is the clean `wf_config c → cstep c c' → wf_config  *)
(* c'`. This reuses the value-substitution machinery (vshift /    *)
(* vtype_subst / vlift0_vshift) proved for S1.1a verbatim.        *)
(*                                                              *)
(* Faithfulness fence (additional to the file header): this form *)
(* is the TWO-PARTY, SINGLE-SESSION, straight-line fragment —    *)
(* each party is a sequential behaviour (send/recv/end) on its   *)
(* one endpoint; no intra-party parallelism, no sub-session      *)
(* nesting, no free channels, no choice. It mechanises exactly   *)
(* the duet paper's `(νc)(P∣Q)` session initiation + R-Comm with *)
(* its subject reduction; richer process structure stays with    *)
(* the open `proc` calculus (whose R-Comm case is `sr_comm`).    *)
(* ============================================================ *)

(* A party = one endpoint's sequential session behaviour. *)
(* As with [sty]/[sbranch], party branches are a dedicated mutual   *)
(* inductive [pbranch] so [psubst_party] below is a well-guarded    *)
(* mutual fixpoint (recursion into the [party] field of [PBcons]).  *)
Inductive party : Type :=
| QEnd  : party                       (* close *)
| QSend : val -> party -> party       (* send v, continue *)
| QRecv : party -> party              (* receive (bind payload de Bruijn 0), continue *)
| QSel  : nat -> party -> party       (* select label l, continue        (S1.3a) *)
| QBra  : pbranch -> party            (* offer labelled continuations    (S1.3a) *)
with pbranch : Type :=
| PBnil  : pbranch
| PBcons : nat -> party -> pbranch -> pbranch.

(* Mutual induction principle (party + pbranch), needed for         *)
(* pty_subst's QBra case to recurse structurally into the branches. *)
Scheme party_mut := Induction for party Sort Prop
  with pbranch_mut := Induction for pbranch Sort Prop.

(* Party-branch lookup (first match); used in the typing rule below. *)
Fixpoint pget (l : nat) (bs : pbranch) : option party :=
  match bs with
  | PBnil           => None
  | PBcons k q rest => if Nat.eqb l k then Some q else pget l rest
  end.

(* Party typing: [pty G p s] — under payload context G, party p    *)
(* follows session type s exactly.                                 *)
Inductive pty : list vty -> party -> sty -> Prop :=
| PT_End  : forall G, pty G QEnd SEnd
| PT_Send : forall G v p t s, vtype G v t -> pty G p s -> pty G (QSend v p) (SSend t s)
| PT_Recv : forall G p t s, pty (t :: G) p s -> pty G (QRecv p) (SRecv t s)
| PT_Sel  : forall G l p s bs,                          (* S1.3a: select *)
    bget l bs = Some s -> pty G p s ->
    pty G (QSel l p) (SSelect bs)
| PT_Bra  : forall G bsP bs,                            (* S1.3a: branch *)
    (* every branch DECLARED in the type has a matching offered, *)
    (* correctly-typed continuation (label-coverage / mismatch   *)
    (* safety).                                                   *)
    (forall l sB, bget l bs = Some sB ->
                  exists q, pget l bsP = Some q /\ pty G q sB) ->
    pty G (QBra bsP) (SBranch bs).

Fixpoint psubst_party (c : nat) (u : val) (p : party) : party :=
  match p with
  | QEnd      => QEnd
  | QSend v q => QSend (vsubst c u v) (psubst_party c u q)
  | QRecv q   => QRecv (psubst_party (S c) (vlift 0 u) q)
  | QSel l q  => QSel l (psubst_party c u q)
  | QBra bs   => QBra (psubst_pbranch c u bs)
  end
with psubst_pbranch (c : nat) (u : val) (bs : pbranch) : pbranch :=
  match bs with
  | PBnil           => PBnil
  | PBcons l q rest => PBcons l (psubst_party c u q) (psubst_pbranch c u rest)
  end.

Definition open_party (u : val) (p : party) : party := psubst_party 0 u p.

(* [pget] (defined above, before [pty]) commutes with payload       *)
(* substitution into the branch list.                               *)
Lemma pget_psubst : forall l bs c u q,
  pget l bs = Some q ->
  pget l (psubst_pbranch c u bs) = Some (psubst_party c u q).
Proof.
  intros l bs; induction bs as [ | k q0 rest IH ]; simpl; intros c u q H.
  - discriminate.
  - destruct (Nat.eqb l k) eqn:E.
    + injection H as ->. reflexivity.
    + apply IH. exact H.
Qed.

(* Party inversion helpers (clean names). *)
Lemma pty_send_inv : forall G v p s, pty G (QSend v p) s ->
  exists t s', s = SSend t s' /\ vtype G v t /\ pty G p s'.
Proof. intros G v p s H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

Lemma pty_recv_inv : forall G p s, pty G (QRecv p) s ->
  exists t s', s = SRecv t s' /\ pty (t :: G) p s'.
Proof. intros G p s H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

Lemma pty_sel_inv : forall G l p s, pty G (QSel l p) s ->
  exists sl bs, s = SSelect bs /\ bget l bs = Some sl /\ pty G p sl.
Proof. intros G l p s H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

Lemma pty_bra_inv : forall G bsP s, pty G (QBra bsP) s ->
  exists bs, s = SBranch bs /\
    (forall l sB, bget l bs = Some sB -> exists q, pget l bsP = Some q /\ pty G q sB).
Proof. intros G bsP s H. inversion H; subst. eexists; split; [ reflexivity | eassumption ]. Qed.

(* Type-directed inversions: when only the TYPE is known to be a    *)
(* choice (the party is abstract — e.g. the partner in progress),   *)
(* the party MUST be the corresponding select/branch process.       *)
Lemma pty_select_ty_inv : forall G p bs, pty G p (SSelect bs) ->
  exists l p' sl, p = QSel l p' /\ bget l bs = Some sl /\ pty G p' sl.
Proof. intros G p bs H. inversion H; subst. do 3 eexists; repeat split; eassumption. Qed.

Lemma pty_branch_ty_inv : forall G p bs, pty G p (SBranch bs) ->
  exists bsP, p = QBra bsP /\
    (forall l sB, bget l bs = Some sB -> exists q, pget l bsP = Some q /\ pty G q sB).
Proof. intros G p bs H. inversion H; subst. eexists; split; [ reflexivity | eassumption ]. Qed.

(* Party value-substitution lemma (the party analogue of wt_subst, *)
(* reusing the same value-shift machinery).                        *)
Lemma pty_subst : forall p G1 G2 s t v,
  pty (G1 ++ t :: G2) p s ->
  vtype G2 v t ->
  pty (G1 ++ G2) (psubst_party (length G1) (vshift (length G1) v) p) s.
Proof.
  intro p.
  induction p as [ | v0 p0 IH | p0 IH | l0 p0 IH | b IH0 | | l0 p0 IHp rest IHrest ]
    using party_mut
    with (P0 := fun b => forall G1 G2 t v l q sB,
                 vtype G2 v t -> pget l b = Some q -> pty (G1 ++ t :: G2) q sB ->
                 pty (G1 ++ G2) (psubst_party (length G1) (vshift (length G1) v) q) sB).
  - (* QEnd *) intros G1 G2 s t v Hp Hv. simpl. inversion Hp; subst. constructor.
  - (* QSend v0 p0 *) intros G1 G2 s t v Hp Hv. simpl. inversion Hp; subst. constructor.
    + eapply vtype_subst; eassumption.
    + apply IH with (t:=t); assumption.
  - (* QRecv p0 *) intros G1 G2 s t v Hp Hv. simpl. inversion Hp; subst. constructor.
    rewrite vlift0_vshift. apply (IH (t0 :: G1) G2 s0 t v); assumption.
  - (* QSel l0 p0 *) intros G1 G2 s t v Hp Hv. simpl.
    apply pty_sel_inv in Hp. destruct Hp as (sl & bs & Es & Hb & Hq). subst s.
    eapply PT_Sel; [ exact Hb | apply IH with (t:=t); assumption ].
  - (* QBra b *) intros G1 G2 s t v Hp Hv. simpl.
    apply pty_bra_inv in Hp. destruct Hp as (bs & Es & Hcov). subst s.
    apply PT_Bra. intros l sB Hb.
    destruct (Hcov l sB Hb) as [q [Hq Htq]].
    exists (psubst_party (length G1) (vshift (length G1) v) q). split.
    + apply pget_psubst. exact Hq.
    + apply (IH0 G1 G2 t v l q sB); assumption.
  - (* PBnil *) intros G1 G2 t v l q sB Hv Hpg Hq. simpl in Hpg. discriminate.
  - (* PBcons l0 p0 rest *) intros G1 G2 t v l q sB Hv Hpg Hq. simpl in Hpg.
    destruct (Nat.eqb l l0) eqn:E.
    + inversion Hpg; subst. apply IHp with (t:=t); assumption.
    + apply (IHrest G1 G2 t v l q sB); assumption.
Qed.

Corollary pty_subst0 : forall G p s t v,
  pty (t :: G) p s -> vtype G v t -> pty G (open_party v p) s.
Proof.
  intros G p s t v Hp Hv.
  pose proof (pty_subst p [] G s t v Hp Hv) as HH.
  simpl in HH. rewrite vshift_0 in HH. unfold open_party. exact HH.
Qed.

(* A configuration = the two parties of one session, (νc)(P ∣ Q). *)
Inductive config : Type := Conf : party -> party -> config.

(* Well-formed: the two parties run dual protocols (the T-Session  *)
(* premise Δ₁(c) = dual(Δ₂(c))). The protocol s is existential —   *)
(* this is what lets the ν channel RE-TYPE across a communication. *)
Definition wf_config (c : config) : Prop :=
  match c with Conf P Q => exists s, pty [] P s /\ pty [] Q (dual s) end.

(* Synchronous communication; the session protocol advances        *)
(* locally to its continuation, the free context is unchanged.     *)
Inductive cstep : config -> config -> Prop :=
| CStep  : forall v P Q, cstep (Conf (QSend v P) (QRecv Q)) (Conf P (open_party v Q))
| CStepR : forall v P Q, cstep (Conf (QRecv Q) (QSend v P)) (Conf (open_party v Q) P)
| CSel   : forall l P bsP Q, pget l bsP = Some Q ->         (* S1.3a: R-Choice *)
    cstep (Conf (QSel l P) (QBra bsP)) (Conf P Q)
| CSelR  : forall l P bsP Q, pget l bsP = Some Q ->
    cstep (Conf (QBra bsP) (QSel l P)) (Conf Q P).

(* ----- the S1.1b headline: closed-system subject reduction ----- *)
(* Well-formedness (both parties dual) is preserved by reduction —  *)
(* the two endpoints stay dual as their shared protocol advances.   *)
Theorem config_subject_reduction : forall c c',
  wf_config c -> cstep c c' -> wf_config c'.
Proof.
  intros c c' Hwf Hstep.
  destruct Hstep as [v P Q | v P Q | l P bsP Q HIn | l P bsP Q HIn].
  - (* CStep : sender on the left *)
    destruct Hwf as [s [HP HQ]].
    apply pty_send_inv in HP. destruct HP as (t & s' & Es & Hv & HPc). subst s.
    simpl in HQ. apply pty_recv_inv in HQ. destruct HQ as (t2 & s2 & Es2 & HQc).
    inversion Es2; subst t2 s2.
    exists s'. split; [ exact HPc | eapply pty_subst0; eassumption ].
  - (* CStepR : sender on the right *)
    destruct Hwf as [s [HP HQ]].
    apply pty_recv_inv in HP. destruct HP as (t & s' & Es & HQc). subst s.
    simpl in HQ. apply pty_send_inv in HQ. destruct HQ as (t2 & s2 & Es2 & Hv & HPc).
    inversion Es2; subst t2 s2.
    exists s'. split; [ eapply pty_subst0; eassumption | exact HPc ].
  - (* CSel : selector (QSel) on the left, brancher (QBra) on the right *)
    destruct Hwf as [s [HP HQ]].
    apply pty_sel_inv in HP. destruct HP as (sl & bs & Es & Hasl & HPc). subst s.
    simpl in HQ. apply pty_bra_inv in HQ. destruct HQ as (B & EB & Hcov).
    injection EB as EB'. subst B.
    exists sl. split.
    + exact HPc.
    + assert (Hd : bget l (dual_br bs) = Some (dual sl)) by (apply bget_dual_br; exact Hasl).
      destruct (Hcov l (dual sl) Hd) as [q [Hq Htq]].
      assert (q = Q) by congruence. subst q. exact Htq.
  - (* CSelR : brancher (QBra) on the left, selector (QSel) on the right *)
    destruct Hwf as [s [HP HQ]].
    apply pty_bra_inv in HP. destruct HP as (bs & Es & Hcov). subst s.
    simpl in HQ. apply pty_sel_inv in HQ. destruct HQ as (sl & B & EB & Hasl & HPc).
    injection EB as EB'. subst B.
    apply bget_dual_br_inv in Hasl. destruct Hasl as [sB [HsB Esl]]. subst sl.
    destruct (Hcov l sB HsB) as [q [Hq Htq]].
    assert (q = Q) by congruence. subst q.
    exists sB. split; [ exact Htq | exact HPc ].
Qed.

(* ----- executable witnesses for the fused form ----- *)
Example wf_pingpong_config :
  wf_config (Conf (QSend (VNat 7) QEnd) (QRecv QEnd)).
Proof.
  exists (SSend VTNat SEnd). split.
  - apply PT_Send; [ apply VT_Nat | apply PT_End ].
  - simpl. apply PT_Recv. apply PT_End.
Qed.

Example cstep_pingpong_config :
  cstep (Conf (QSend (VNat 7) QEnd) (QRecv QEnd)) (Conf QEnd QEnd).
Proof. apply CStep. Qed.

(* The whole point, instantiated: the reduct is still well-formed.  *)
Example wf_preserved_pingpong :
  wf_config (Conf QEnd QEnd).
Proof.
  apply (config_subject_reduction
           (Conf (QSend (VNat 7) QEnd) (QRecv QEnd))).
  - apply wf_pingpong_config.
  - apply cstep_pingpong_config.
Qed.

(* ============================================================ *)
(* S1.2 — session fidelity + progress (deadlock freedom)         *)
(*                                                              *)
(* On the fused two-party form these are the duet paper's        *)
(* headline SAFETY (fidelity) and LIVENESS (progress / deadlock  *)
(* freedom) theorems, mechanised axiom-free for the send/recv/   *)
(* end fragment. They fall out cleanly because the two parties   *)
(* run dual protocols: every action of one is the dual action of *)
(* the other, so a well-formed non-ended config can ALWAYS step, *)
(* and each step advances the shared protocol by exactly its     *)
(* head action.                                                  *)
(*                                                              *)
(* Choice (select/branch) is now done — S1.3a, below, extends the *)
(* three theorems above with the n-ary labelled choice cases.     *)
(* Still OUT: μ-recursive sessions (S1.3b — type layer below,     *)
(* typing/SR deferred), structural congruence over the open      *)
(* `proc` calculus (S1.3c), and multiparty G / projection (S2,   *)
(* done separately above).                                       *)
(* ============================================================ *)

(* A single session-type transition: consume the head action. *)
Inductive sty_step : sty -> sty -> Prop :=
| SS_Send : forall t s, sty_step (SSend t s) s
| SS_Recv : forall t s, sty_step (SRecv t s) s
| SS_Sel  : forall l s bs, bget l bs = Some s -> sty_step (SSelect bs) s  (* S1.3a *)
| SS_Bra  : forall l s bs, bget l bs = Some s -> sty_step (SBranch bs) s. (* S1.3a *)

(* ----- session fidelity ----- *)
(* A communication of a well-formed two-party config advances the *)
(* shared protocol by EXACTLY one session step, and the reduct is *)
(* well-formed at that successor protocol. This both refines      *)
(* subject reduction (it pins WHICH protocol the reduct follows)  *)
(* and IS fidelity: the reduction follows the session type.       *)
Theorem session_fidelity : forall P Q c' s,
  pty [] P s -> pty [] Q (dual s) -> cstep (Conf P Q) c' ->
  exists s' P' Q', c' = Conf P' Q'
                /\ sty_step s s' /\ pty [] P' s' /\ pty [] Q' (dual s').
Proof.
  intros P Q c' s HP HQ Hstep. inversion Hstep; subst.
  - (* CStep : sender on the left *)
    apply pty_send_inv in HP. destruct HP as (t & s' & Es & Hv & HPc). subst s.
    simpl in HQ. apply pty_recv_inv in HQ. destruct HQ as (t2 & s2 & Es2 & HQc).
    inversion Es2; subst t2 s2.
    exists s', P0, (open_party v Q0). repeat split.
    + apply SS_Send.
    + exact HPc.
    + eapply pty_subst0; eassumption.
  - (* CStepR : sender on the right *)
    apply pty_recv_inv in HP. destruct HP as (t & s' & Es & HQc). subst s.
    simpl in HQ. apply pty_send_inv in HQ. destruct HQ as (t2 & s2 & Es2 & Hv & HPc).
    inversion Es2; subst t2 s2.
    exists s', (open_party v Q0), P0. repeat split.
    + apply SS_Recv.
    + eapply pty_subst0; eassumption.
    + exact HPc.
  - (* CSel : selector on the left; the chosen label drives SS_Sel *)
    apply pty_sel_inv in HP. destruct HP as (sl & bs & Es & Hasl & HPc). subst s.
    simpl in HQ. apply pty_bra_inv in HQ. destruct HQ as (Bb & EB & Hcov).
    injection EB as EB'. subst Bb.
    assert (Hd : bget _ (dual_br bs) = Some (dual sl)) by (apply bget_dual_br; exact Hasl).
    destruct (Hcov _ (dual sl) Hd) as [q0 [Hq Htq]].
    exists sl, P0, Q0. repeat split.
    + eapply SS_Sel; exact Hasl.
    + exact HPc.
    + assert (Q0 = q0) by congruence. subst q0. exact Htq.
  - (* CSelR : brancher on the left; the chosen label drives SS_Bra *)
    apply pty_bra_inv in HP. destruct HP as (bs & Es & Hcov). subst s.
    simpl in HQ. apply pty_sel_inv in HQ. destruct HQ as (sl & Bb & EB & Hasl & HPc).
    injection EB as EB'. subst Bb.
    apply bget_dual_br_inv in Hasl. destruct Hasl as [sB [HsB Esl]]. subst sl.
    destruct (Hcov _ sB HsB) as [q0 [Hq Htq]].
    exists sB, Q0, P0. repeat split.
    + eapply SS_Bra; exact HsB.
    + assert (Q0 = q0) by congruence. subst q0. exact Htq.
    + exact HPc.
Qed.

(* ----- progress / deadlock freedom ----- *)
(* A well-formed config is either fully ENDED (both parties done) *)
(* or it can take a step. There is no stuck (deadlocked) state:   *)
(* duality guarantees one party's pending action is matched by    *)
(* the other's dual action.                                       *)
Theorem config_progress : forall c,
  wf_config c -> c = Conf QEnd QEnd \/ exists c', cstep c c'.
Proof.
  intros [P Q] [s [HP HQ]].
  destruct P as [ | v P0 | P0 | l P0 | bsP ].
  - (* P = QEnd : s = SEnd, so dual s = SEnd, so Q = QEnd *)
    inversion HP; subst. simpl in HQ. inversion HQ; subst.
    left. reflexivity.
  - (* P = QSend : Q must be QRecv (dual), so CStep fires *)
    inversion HP; subst. simpl in HQ. inversion HQ; subst.
    right. eexists. apply CStep.
  - (* P = QRecv : Q must be QSend (dual), so CStepR fires *)
    inversion HP; subst. simpl in HQ. inversion HQ; subst.
    right. eexists. apply CStepR.
  - (* P = QSel l P0 : Q must be QBra (dual), and l is offered, so CSel fires *)
    apply pty_sel_inv in HP. destruct HP as (sl & bs & Es & Hasl & HPc). subst s.
    simpl in HQ. apply pty_branch_ty_inv in HQ. destruct HQ as (bsP & EQ & Hcov). subst Q.
    assert (Hd : bget l (dual_br bs) = Some (dual sl)) by (apply bget_dual_br; exact Hasl).
    destruct (Hcov l (dual sl) Hd) as [q0 [Hq _]].
    right. eexists. apply CSel. exact Hq.
  - (* P = QBra bsP : Q must be QSel (dual), which selects an offered label, so CSelR fires *)
    apply pty_bra_inv in HP. destruct HP as (bs & Es & Hcov). subst s.
    simpl in HQ. apply pty_select_ty_inv in HQ.
    destruct HQ as (l & P1 & sl & EQ & Hasl & HPc). subst Q.
    apply bget_dual_br_inv in Hasl. destruct Hasl as [sB [HsB Esl]]. subst sl.
    destruct (Hcov l sB HsB) as [q0 [Hq _]].
    right. eexists. apply CSelR. exact Hq.
Qed.

(* Witness: the ping-pong config is not stuck — it can step. *)
Example progress_pingpong :
  (Conf (QSend (VNat 7) QEnd) (QRecv QEnd)) = Conf QEnd QEnd
  \/ exists c', cstep (Conf (QSend (VNat 7) QEnd) (QRecv QEnd)) c'.
Proof. apply config_progress. apply wf_pingpong_config. Qed.

(* ----- executable witnesses for the S1.3a CHOICE layer ----- *)
(* A 2-branch labelled choice: the selector picks label 0, the     *)
(* brancher offers {0:end, 1:end}.  All real Qed.                  *)
Definition choice_sty : sty := SSelect (SBcons 0 SEnd (SBcons 1 SEnd SBnil)).
Definition choice_sel : party := QSel 0 QEnd.
Definition choice_bra : party := QBra (PBcons 0 QEnd (PBcons 1 QEnd PBnil)).

Example wf_choice_config : wf_config (Conf choice_sel choice_bra).
Proof.
  exists choice_sty. unfold choice_sel, choice_bra, choice_sty. split.
  - eapply PT_Sel; [ reflexivity | apply PT_End ].
  - simpl. apply PT_Bra. intros l sB Hb. simpl in Hb.
    destruct (Nat.eqb l 0) eqn:E0.
    + injection Hb as <-. exists QEnd. simpl. rewrite E0. split; [ reflexivity | apply PT_End ].
    + destruct (Nat.eqb l 1) eqn:E1.
      * injection Hb as <-. exists QEnd. simpl. rewrite E0, E1.
        split; [ reflexivity | apply PT_End ].
      * discriminate.
Qed.

(* The choice config steps (label 0 selected) to the ended config. *)
Example cstep_choice_config :
  cstep (Conf choice_sel choice_bra) (Conf QEnd QEnd).
Proof. unfold choice_sel, choice_bra. apply CSel. reflexivity. Qed.

(* ...and it is deadlock-free (a one-line instance of progress).   *)
Example progress_choice :
  Conf choice_sel choice_bra = Conf QEnd QEnd
  \/ exists c', cstep (Conf choice_sel choice_bra) c'.
Proof. apply config_progress. apply wf_choice_config. Qed.

(* ============================================================ *)
(* S2 — duet by projection (axis-2 STRUCTURE).                  *)
(*                                                              *)
(* The duet thesis "duet = ensemble | 2-party" rendered as a    *)
(* theorem: a global choreography, PROJECTED onto its two roles,*)
(* yields DUAL binary local session types, so the two projected *)
(* endpoints compose into a well-formed fused config and thereby*)
(* INHERIT the entire S1.1b/S1.2 metatheory (subject reduction, *)
(* fidelity, progress / deadlock-freedom) for free.             *)
(*                                                              *)
(* It reuses the existing session-type layer verbatim (sty,     *)
(* dual, dual_involutive) and the fused two-party config        *)
(* metatheory (party, pty, config, wf_config,                   *)
(* config_subject_reduction, session_fidelity, config_progress).*)
(*                                                              *)
(* ----- FAITHFULNESS FENCE (S2 — honest scope). --------------- *)
(* WHAT IS PROVED. For a TWO-PARTY choreography — a [gty] all of *)
(*   whose messages are between the SAME fixed pair p,q          *)
(*   (predicate [two_party p q G]) — projection yields DUAL      *)
(*   binary local types (projection_duality), the two role-      *)
(*   projections form one dual channel                           *)
(*   (global_projects_to_dual_channel), and composing the two    *)
(*   projected endpoints transports the full S1.1b/S1.2          *)
(*   guarantee: well-formedness (projected_config_wf), subject   *)
(*   reduction (projected_config_subject_reduction), session     *)
(*   fidelity (projected_session_fidelity), and progress /       *)
(*   deadlock-freedom (projected_config_progress). This          *)
(*   INSTANTIATES the duet thesis on two-party choreographies;   *)
(*   it does NOT establish a general n-party -> 2-party result.  *)
(* GLOBAL-TYPE LANGUAGE (S2.0 + S2.2). Message + end (GMsg|GEnd) *)
(*   PLUS (S2.2) labelled choice GBra p q {l_i:G_i} and equi-    *)
(*   recursive GMu/GVar. [role := nat] so uninvolved-party       *)
(*   branches are SYNTACTICALLY reachable, but NO theorem        *)
(*   quantifies over a genuine n>=3 system: every theorem's      *)
(*   [two_party] hypothesis re-collapses the nat role space to   *)
(*   {p,q}, under which every uninvolved branch is DEAD.         *)
(* PROJECTION EQUATIONS. All are DEFINED in [proj] (send/recv,   *)
(*   select/branch, mu/var, and the uninvolved passthru/merge).  *)
(*   The p- and q-cases are LOAD-BEARING in projection_duality;  *)
(*   the uninvolved cases are exercised by executable witnesses  *)
(*   (3-party gty projected on each role) and are provably       *)
(*   unreachable inside the theorems: defined-and-witnessed, NOT *)
(*   all-verified-in-a-theorem.                                  *)
(* COHERENCE / MERGE (S2.2, FENCED). With choice, [proj] is now  *)
(*   PARTIAL (option sty): an uninvolved role merges the         *)
(*   branches via [merge].  merge is the PLAIN / identity-MEET   *)
(*   merge (keystone merge_idem) — NOT the full label-set-UNION  *)
(*   merge; the n>=3 coherence/projection-EXISTENCE it unlocks   *)
(*   is deferred to S3.  Still NO well-formedness predicate on   *)
(*   [gty]; an arbitrary node with roles outside {p,q} is a      *)
(*   valid gty that NO theorem constrains.                       *)
(* MU PROJECTION (S2.2, FENCED). proj (GMu G) r = mu.(G|>r) is   *)
(*   UNPRUNED: a non-participating role yields mu X.X, a non-    *)
(*   contractive artefact (rejected by [guarded]/                *)
(*   not_guarded_muvar), occurring only OUTSIDE the two_party    *)
(*   {p,q} collapse (non-theorematic).  No witness displays it   *)
(*   as a role's session type.  Participation-pruning deferred.  *)
(* THE p<>q SIDE-CONDITION is load-bearing, not decorative: at   *)
(*   p=q the duality theorem is FALSE (a self-send GMsg 0 0 t    *)
(*   GEnd projects to SSend, whose dual is SRecv). p<>q is what  *)
(*   puts self-communication (p->p) out of scope.                *)
(* NO GLOBAL-LEVEL METATHEORY. Subject reduction / fidelity /    *)
(*   progress are NOT re-proved for GBra/GMu at the global       *)
(*   level; the duet-collapse bridge transports the BINARY       *)
(*   metatheory only where BOTH projections are Some.            *)
(* OUT (deferred). S3: n>=3 coherence / full-UNION merge /       *)
(*   projection-EXISTENCE — the boundary where "duet" stops and  *)
(*   "ensemble" begins.  S1.3b-meta: mu TYPING up-to-unfolding.  *)
(*   No payload-binding in global types (base value types only). *)
(* ECHO-TYPES AUDIT (performed, not assumed). Projection emits   *)
(*   no obligation/residue/attestation; it is pure axis-2        *)
(*   STRUCTURE. No axis-3 (L3 echo) obligation arises ->         *)
(*   recorded NOT-RELEVANT (AXIS-ARCHITECTURE.md sec.3 — axes    *)
(*   compose but must not collapse), same outcome as S1,         *)
(*   re-checked for the multiparty-syntax layer specifically.    *)
(* ============================================================ *)

(* ================= S2.0: global types + projection ========== *)

(* Participants. [nat] (not a 2-element type) so the paper's     *)
(* third projection equation  (p->q:t.G)|>r = G|>r  (r<>p,q) is  *)
(* SYNTACTICALLY reachable — witness-enabling + future-proofing  *)
(* only; no theorem quantifies over a genuine n-party system.    *)
Definition role := nat.

(* Global type (choreography). S2.0 shipped the MESSAGE fragment;  *)
(* S2.2 adds labelled CHOICE  p->q:{l_i:G_i}  (GBra, with a        *)
(* DEDICATED mutual gbranch — NOT list, to keep the projection     *)
(* fixpoints' guard checker happy: the S1.3a lesson) and equi-     *)
(* recursive  mu X.G / X  (GMu / GVar, de Bruijn aligned with the  *)
(* local SMu/SVar layer).                                          *)
(*   GMsg p q t G  ==  p -> q : t . G        (p sends a t to q)    *)
(*   GBra p q bs   ==  p -> q : {l_i : G_i}  (p picks a label)     *)
(*   GMu  G        ==  mu X. G               (recursive protocol)  *)
(*   GVar n        ==  X                     (recursion variable)  *)
Inductive gty : Type :=
| GEnd : gty
| GMsg : role -> role -> vty -> gty -> gty
| GBra : role -> role -> gbranch -> gty
| GMu  : gty -> gty
| GVar : nat -> gty
with gbranch : Type :=
| GBnil  : gbranch
| GBcons : nat -> gty -> gbranch -> gbranch.

(* ---- merge: the uninvolved-role projection's combinator ----    *)
(* Decidable payload equality, to reject mismatched message        *)
(* payloads inside merge.                                          *)
Definition vty_eqb (a b : vty) : bool :=
  match a, b with
  | VTUnit, VTUnit => true
  | VTBool, VTBool => true
  | VTNat , VTNat  => true
  | _, _ => false
  end.
Lemma vty_eqb_refl : forall t, vty_eqb t t = true.
Proof. destruct t; reflexivity. Qed.

(* [merge s1 s2] : the local type a role uninvolved in a choice    *)
(* must follow when the choice's branches demand s1 vs s2.         *)
(*                                                                 *)
(* FENCE (honest scope — the "merge" name is only PARTIALLY        *)
(* earned).  This is the PLAIN / identity-MEET merge.  On the      *)
(* message fragment (SEnd/SSend/SRecv/SVar/SMu) it succeeds IFF    *)
(* the two types are equal (merge s1 s2 = Some s -> s1 = s2 = s);  *)
(* the only structural give is that branch CONTINUATIONS merge     *)
(* pointwise under IDENTICAL label structure.  It is NOT the full  *)
(* label-set-UNION merge of Honda-Yoshida-Carbone (where a merged  *)
(* role may offer the UNION of the branches' labels); that, and    *)
(* the n>=3 coherence it unlocks, is deferred to S3.  merge        *)
(* recurses STRUCTURALLY on s1 (principal arg); it NEVER unfolds   *)
(* an SMu — that would break the guard — it recurses on the body.  *)
Fixpoint merge (s1 s2 : sty) {struct s1} : option sty :=
  match s1, s2 with
  | SEnd, SEnd => Some SEnd
  | SSend t1 k1, SSend t2 k2 =>
      if vty_eqb t1 t2
      then match merge k1 k2 with Some k => Some (SSend t1 k) | None => None end
      else None
  | SRecv t1 k1, SRecv t2 k2 =>
      if vty_eqb t1 t2
      then match merge k1 k2 with Some k => Some (SRecv t1 k) | None => None end
      else None
  | SSelect b1, SSelect b2 =>
      match merge_br b1 b2 with Some b => Some (SSelect b) | None => None end
  | SBranch b1, SBranch b2 =>
      match merge_br b1 b2 with Some b => Some (SBranch b) | None => None end
  | SVar n1, SVar n2 => if Nat.eqb n1 n2 then Some (SVar n1) else None
  | SMu a1, SMu a2 =>
      match merge a1 a2 with Some a => Some (SMu a) | None => None end
  | _, _ => None
  end
with merge_br (b1 b2 : sbranch) {struct b1} : option sbranch :=
  match b1, b2 with
  | SBnil, SBnil => Some SBnil
  | SBcons l1 s1 r1, SBcons l2 s2 r2 =>
      if Nat.eqb l1 l2
      then match merge s1 s2, merge_br r1 r2 with
           | Some s, Some r => Some (SBcons l1 s r)
           | _, _ => None
           end
      else None
  | _, _ => None
  end.

(* Mutual induction principle over the local types, for merge_idem.*)
Scheme sty_mut := Induction for sty Sort Prop
  with sbranch_mut := Induction for sbranch Sort Prop.

(* merge KEYSTONE: merge is reflexive — every type merges with     *)
(* itself, to itself.  This is exactly what makes an uninvolved    *)
(* role's projection DEFINED whenever the choice's branches AGREE  *)
(* on that role (the plain-merge projectability condition).        *)
Lemma merge_idem : forall s, merge s s = Some s.
Proof.
  intro s.
  induction s using sty_mut
    with (P0 := fun bs => merge_br bs bs = Some bs); simpl; try reflexivity.
  - (* SSend   *) rewrite vty_eqb_refl, IHs. reflexivity.
  - (* SRecv   *) rewrite vty_eqb_refl, IHs. reflexivity.
  - (* SSelect *) rewrite IHs. reflexivity.
  - (* SBranch *) rewrite IHs. reflexivity.
  - (* SVar    *) rewrite Nat.eqb_refl. reflexivity.
  - (* SMu     *) rewrite IHs. reflexivity.
  - (* SBcons  *) rewrite Nat.eqb_refl, IHs, IHs0. reflexivity.
Qed.

(* Projection  G|>r, now PARTIAL (option sty): an uninvolved role  *)
(* must MERGE the choice branches, which can fail.  Three-way      *)
(* mutual (proj / proj_br / proj_uninv), all structural on the     *)
(* gty/gbranch tree.                                               *)
(*   (p->q:t.G)|>p     = !t.(G|>p)            sender   -> SSend     *)
(*   (p->q:t.G)|>q     = ?t.(G|>q)            receiver -> SRecv     *)
(*   (p->q:t.G)|>r     = G|>r                 uninvolved passthru   *)
(*   (p->q:{l:G})|>p   = +{l:(G|>p)}          sender   -> SSelect   *)
(*   (p->q:{l:G})|>q   = &{l:(G|>q)}          receiver -> SBranch   *)
(*   (p->q:{l:G})|>r   = merge_i (G_i|>r)     uninvolved -> MERGE   *)
(*   (mu X.G)|>r       = mu X.(G|>r)          (UNPRUNED — fence b)  *)
(*   X|>r              = X                                          *)
(* proj_uninv folds the branches with merge; EMPTY = None, the     *)
(* SINGLETON case is the body alone, and ANY None (unprojectable   *)
(* body OR a merge conflict, including a TAIL conflict) propagates  *)
(* — never silently erased (the S2.2 panel's None-erasure fix).    *)
Fixpoint proj (G : gty) (r : role) {struct G} : option sty :=
  match G with
  | GEnd          => Some SEnd
  | GMsg p q t G' =>
      if Nat.eqb r p then option_map (SSend t) (proj G' r)
      else if Nat.eqb r q then option_map (SRecv t) (proj G' r)
      else proj G' r
  | GBra p q bs   =>
      if Nat.eqb r p then option_map SSelect (proj_br bs r)
      else if Nat.eqb r q then option_map SBranch (proj_br bs r)
      else proj_uninv bs r
  | GMu G'        => option_map SMu (proj G' r)
  | GVar n        => Some (SVar n)
  end
with proj_br (bs : gbranch) (r : role) {struct bs} : option sbranch :=
  match bs with
  | GBnil           => Some SBnil
  | GBcons l G rest =>
      match proj G r, proj_br rest r with
      | Some s, Some sb => Some (SBcons l s sb)
      | _, _ => None
      end
  end
with proj_uninv (bs : gbranch) (r : role) {struct bs} : option sty :=
  match bs with
  | GBnil           => None
  | GBcons l G rest =>
      match rest with
      | GBnil => proj G r
      | _     => match proj G r, proj_uninv rest r with
                 | Some s, Some s' => merge s s'
                 | _, _ => None
                 end
      end
  end.

(* [two_party p q G] : every message/choice node in G is between *)
(* exactly the fixed pair p,q (either direction). The binary      *)
(* restriction is enforced at EVERY GMsg/GBra node (branches via  *)
(* the mutual two_party_br); GVar/GMu carry it transparently — a  *)
(* third role is unrepresentable except at a message/choice node. *)
(* NB it does NOT by itself entail p<>q (TP_PQ/TP_QP coincide and *)
(* admit a self-send when p=q); distinctness is carried as a      *)
(* theorem hypothesis where needed — see the p<>q fence note.     *)
Inductive two_party (p q : role) : gty -> Prop :=
| TP_End   : two_party p q GEnd
| TP_PQ    : forall t G, two_party p q G -> two_party p q (GMsg p q t G)
| TP_QP    : forall t G, two_party p q G -> two_party p q (GMsg q p t G)
| TP_BraPQ : forall bs, two_party_br p q bs -> two_party p q (GBra p q bs)
| TP_BraQP : forall bs, two_party_br p q bs -> two_party p q (GBra q p bs)
| TP_Mu    : forall G, two_party p q G -> two_party p q (GMu G)
| TP_Var   : forall n, two_party p q (GVar n)
with two_party_br (p q : role) : gbranch -> Prop :=
| TPB_nil  : two_party_br p q GBnil
| TPB_cons : forall l G rest,
    two_party p q G -> two_party_br p q rest ->
    two_party_br p q (GBcons l G rest).

(* Mutual induction principle: the GBra duality case needs the    *)
(* per-branch (P0) motive AND the head/rest IHs.                  *)
Scheme two_party_mut := Induction for two_party Sort Prop
  with two_party_br_mut := Induction for two_party_br Sort Prop.

(* ================= S2.1: projection duality + bridge ======== *)

(* option_map glue: dual passes through each projection wrapper.   *)
(* Each is one [destruct] — they let every duality case collapse   *)
(* to the IH after a single rewrite (the option-form enabler).     *)
Lemma option_map_dual_SSend : forall t y,
  option_map (SSend t) (option_map dual y)
  = option_map dual (option_map (SRecv t) y).
Proof. intros t [s|]; reflexivity. Qed.

Lemma option_map_dual_SRecv : forall t y,
  option_map (SRecv t) (option_map dual y)
  = option_map dual (option_map (SSend t) y).
Proof. intros t [s|]; reflexivity. Qed.

Lemma option_map_dual_SSelect : forall y,
  option_map SSelect (option_map dual_br y)
  = option_map dual (option_map SBranch y).
Proof. intros [b|]; reflexivity. Qed.

Lemma option_map_dual_SBranch : forall y,
  option_map SBranch (option_map dual_br y)
  = option_map dual (option_map SSelect y).
Proof. intros [b|]; reflexivity. Qed.

Lemma option_map_dual_SMu : forall y,
  option_map SMu (option_map dual y) = option_map dual (option_map SMu y).
Proof. intros [s|]; reflexivity. Qed.

(* The duet collapse (S2.0 + S2.2): a two-party choreography       *)
(* projects onto its two roles to give DUAL local types. Now       *)
(* OPTION-valued (projection is partial), so duality is stated     *)
(* with option_map; both projections are in fact Some on the       *)
(* two-party fragment, but the total statement is cleaner and      *)
(* makes each case  option_map f (proj ..) = option_map g (proj..) *)
(* collapse to the IH.  Induction is on the [two_party] DERIVATION *)
(* via the MUTUAL scheme (the GBra case needs the per-branch P0    *)
(* motive proj_br p = option_map dual_br (proj_br q)).  The p<>q    *)
(* hypothesis is load-bearing: it makes the receiver's (q =? p)    *)
(* test false at every GMsg/GBra node.  GMu uses only structural   *)
(* dual(SMu x)=SMu(dual x) — NOT dual_unfold.                       *)
Theorem projection_duality :
  forall p q G, p <> q -> two_party p q G ->
    proj G p = option_map dual (proj G q).
Proof.
  intros p q G Hpq Htp.
  induction Htp as
    [ | t G0 Hsub IH | t G0 Hsub IH | bs Hsub IHbr | bs Hsub IHbr
    | G0 Hsub IH | n | | l G0 rest Hsg IHg Hsr IHr ]
    using two_party_mut
    with (P0 := fun bs (_ : two_party_br p q bs) =>
                  proj_br bs p = option_map dual_br (proj_br bs q)).
  - (* TP_End *) reflexivity.
  - (* TP_PQ : node GMsg p q t G0 *)
    assert (Hqp : (q =? p) = false) by (apply Nat.eqb_neq; congruence).
    cbn [proj]. rewrite Nat.eqb_refl, Hqp, Nat.eqb_refl. cbv beta iota.
    rewrite IH. apply option_map_dual_SSend.
  - (* TP_QP : node GMsg q p t G0 *)
    assert (Hpq' : (p =? q) = false) by (apply Nat.eqb_neq; exact Hpq).
    cbn [proj]. rewrite Hpq', Nat.eqb_refl, Nat.eqb_refl. cbv beta iota.
    rewrite IH. apply option_map_dual_SRecv.
  - (* TP_BraPQ : node GBra p q bs *)
    assert (Hqp : (q =? p) = false) by (apply Nat.eqb_neq; congruence).
    cbn [proj]. rewrite Nat.eqb_refl, Hqp, Nat.eqb_refl. cbv beta iota.
    rewrite IHbr. apply option_map_dual_SSelect.
  - (* TP_BraQP : node GBra q p bs *)
    assert (Hpq' : (p =? q) = false) by (apply Nat.eqb_neq; exact Hpq).
    cbn [proj]. rewrite Hpq', Nat.eqb_refl, Nat.eqb_refl. cbv beta iota.
    rewrite IHbr. apply option_map_dual_SBranch.
  - (* TP_Mu : node GMu G0 *)
    cbn [proj]. rewrite IH. apply option_map_dual_SMu.
  - (* TP_Var : node GVar n *) reflexivity.
  - (* TPB_nil *) reflexivity.
  - (* TPB_cons : node GBcons l G0 rest *)
    cbn [proj_br]. rewrite IHg, IHr.
    destruct (proj G0 q) as [sq|], (proj_br rest q) as [sbq|]; reflexivity.
Qed.

(* Glue lemma (option form): the q-projection is the dual of the *)
(* p-projection.                                                 *)
Lemma proj_q_dual_p :
  forall p q G, p <> q -> two_party p q G ->
    proj G q = option_map dual (proj G p).
Proof.
  intros p q G Hpq Htp.
  rewrite (projection_duality p q G Hpq Htp).
  destruct (proj G q) as [s|]; cbn [option_map].
  - rewrite dual_involutive. reflexivity.
  - reflexivity.
Qed.

(* Some-threaded helper (B8): the bridge + fidelity corollaries   *)
(* consume the projections as concrete Some-values, so the dual   *)
(* relation is funnelled through this single lemma to keep them   *)
(* one-rewrite thin and drift-free.                               *)
Lemma proj_q_dual_p_some :
  forall p q G sp sq, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some sq -> sq = dual sp.
Proof.
  intros p q G sp sq Hpq Htp Hp Hq.
  pose proof (projection_duality p q G Hpq Htp) as Hd.
  rewrite Hp, Hq in Hd. cbn [option_map] in Hd.
  injection Hd as Hd. subst sp. rewrite dual_involutive. reflexivity.
Qed.

(* The bridge: a config built from the two role-projections is   *)
(* well-formed in the fused two-party metatheory — well-formed   *)
(* BY CONSTRUCTION, no separate duality obligation.  Projections  *)
(* are now PARTIAL, so the bridge is conditioned on both being    *)
(* Some (the projectable case); duality is funnelled through      *)
(* proj_q_dual_p_some.                                            *)
Theorem projected_config_wf :
  forall p q G P Q sp sq, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some sq ->
    pty [] P sp -> pty [] Q sq -> wf_config (Conf P Q).
Proof.
  intros p q G P Q sp sq Hpq Htp Hp Hq HP HQ.
  exists sp. split.
  - exact HP.
  - rewrite (proj_q_dual_p_some p q G sp sq Hpq Htp Hp Hq) in HQ. exact HQ.
Qed.

(* The thesis sentence: G IS a duet — its two role-projections   *)
(* form one dual channel (projection_duality, repackaged).       *)
Corollary global_projects_to_dual_channel :
  forall p q G sp, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some (dual sp).
Proof.
  intros p q G sp Hpq Htp Hp.
  rewrite (proj_q_dual_p p q G Hpq Htp), Hp. reflexivity.
Qed.

(* ---- the whole S1.1b/S1.2 guarantee, transported across the   *)
(*      projection (each is one `apply` off wf_config). -------- *)

(* SAFETY: subject reduction for any projected two-party config. *)
Corollary projected_config_subject_reduction :
  forall p q G P Q c' sp sq, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some sq ->
    pty [] P sp -> pty [] Q sq ->
    cstep (Conf P Q) c' -> wf_config c'.
Proof.
  intros p q G P Q c' sp sq Hpq Htp Hp Hq HP HQ Hstep.
  apply (config_subject_reduction (Conf P Q) c').
  - apply (projected_config_wf p q G P Q sp sq Hpq Htp Hp Hq HP HQ).
  - exact Hstep.
Qed.

(* FIDELITY: a step of a projected config advances the SHARED    *)
(* protocol (= sp) by exactly one head action.                   *)
Corollary projected_session_fidelity :
  forall p q G P Q c' sp sq, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some sq ->
    pty [] P sp -> pty [] Q sq ->
    cstep (Conf P Q) c' ->
    exists s' P' Q', c' = Conf P' Q'
      /\ sty_step sp s' /\ pty [] P' s' /\ pty [] Q' (dual s').
Proof.
  intros p q G P Q c' sp sq Hpq Htp Hp Hq HP HQ Hstep.
  apply (session_fidelity P Q c' sp).
  - exact HP.
  - rewrite (proj_q_dual_p_some p q G sp sq Hpq Htp Hp Hq) in HQ. exact HQ.
  - exact Hstep.
Qed.

(* LIVENESS: a projected two-party config is never stuck — every *)
(* projectable choreography is deadlock-free BY CONSTRUCTION.    *)
Corollary projected_config_progress :
  forall p q G P Q sp sq, p <> q -> two_party p q G ->
    proj G p = Some sp -> proj G q = Some sq ->
    pty [] P sp -> pty [] Q sq ->
    Conf P Q = Conf QEnd QEnd \/ exists c', cstep (Conf P Q) c'.
Proof.
  intros p q G P Q sp sq Hpq Htp Hp Hq HP HQ.
  apply config_progress. apply (projected_config_wf p q G P Q sp sq Hpq Htp Hp Hq HP HQ).
Qed.

(* ================= executable witnesses ===================== *)

(* The two-party ping-pong choreography:  0 -> 1 : Nat . end.    *)
Definition gpingpong : gty := GMsg 0 1 VTNat GEnd.

Example proj_ping_0 : proj gpingpong 0 = Some (SSend VTNat SEnd). Proof. reflexivity. Qed.
Example proj_ping_1 : proj gpingpong 1 = Some (SRecv VTNat SEnd). Proof. reflexivity. Qed.
Example two_party_ping : two_party 0 1 gpingpong.
Proof. apply TP_PQ. apply TP_End. Qed.

(* A 3-party choreography:  0 -> 1 : Nat . 1 -> 2 : Bool . end.  *)
(* Projecting it on EACH role fires a DIFFERENT projection       *)
(* branch on one term — role 0 hits the uninvolved (case 3)      *)
(* branch on the 1->2 node, the one a purely-binary gty could    *)
(* never reach.                                                  *)
Definition g3 : gty := GMsg 0 1 VTNat (GMsg 1 2 VTBool GEnd).
Example proj_g3_role0 : proj g3 0 = Some (SSend VTNat SEnd). Proof. reflexivity. Qed.
Example proj_g3_role1 : proj g3 1 = Some (SRecv VTNat (SSend VTBool SEnd)). Proof. reflexivity. Qed.
Example proj_g3_role2 : proj g3 2 = Some (SRecv VTBool SEnd). Proof. reflexivity. Qed.

(* ...and g3 is genuinely OUTSIDE the two-party theorems: it is  *)
(* not two_party 0 1 (the inner 1->2 node refutes it), so the    *)
(* uninvolved branch is witnessed without contaminating duality. *)
Example g3_not_two_party_01 : ~ two_party 0 1 g3.
Proof.
  unfold g3. intro H. inversion H; subst.
  match goal with Hi : two_party 0 1 (GMsg 1 2 _ _) |- _ => inversion Hi end.
Qed.

(* The keystone, instantiated: the config obtained by PROJECTING *)
(* gpingpong onto its two roles is deadlock-free — a one-line    *)
(* instance of the general projected_config_progress.           *)
Example projected_pingpong_deadlock_free :
  Conf (QSend (VNat 7) QEnd) (QRecv QEnd) = Conf QEnd QEnd
  \/ exists c', cstep (Conf (QSend (VNat 7) QEnd) (QRecv QEnd)) c'.
Proof.
  eapply (projected_config_progress 0 1 gpingpong).
  - discriminate.
  - apply two_party_ping.
  - reflexivity.   (* proj gpingpong 0 = Some (SSend VTNat SEnd) *)
  - reflexivity.   (* proj gpingpong 1 = Some (SRecv VTNat SEnd) *)
  - apply PT_Send; [ apply VT_Nat | apply PT_End ].
  - apply PT_Recv. apply PT_End.
Qed.

(* ================= S2.2 witnesses ============================ *)
(* (a) The merge name, EARNED and FENCED: idempotence keystone,  *)
(*     a non-trivial-continuation success, and BOTH failure      *)
(*     modes (payload mismatch, constructor mismatch).           *)
Example merge_idem_witness :
  merge (SSend VTNat SEnd) (SSend VTNat SEnd) = Some (SSend VTNat SEnd).
Proof. reflexivity. Qed.
Example merge_succ_branch :
  merge (SBranch (SBcons 0 (SRecv VTNat SEnd) SBnil))
        (SBranch (SBcons 0 (SRecv VTNat SEnd) SBnil))
  = Some (SBranch (SBcons 0 (SRecv VTNat SEnd) SBnil)).
Proof. reflexivity. Qed.
Example merge_fail_payload :
  merge (SSend VTNat SEnd) (SSend VTBool SEnd) = None.
Proof. reflexivity. Qed.
Example merge_fail_ctor :
  merge (SSend VTNat SEnd) (SRecv VTNat SEnd) = None.
Proof. reflexivity. Qed.

(* (b) PARTIAL projection, witnessed in BOTH directions on an    *)
(*     uninvolved role (2):  AGREE -> Some,  DISAGREE -> None,    *)
(*     plus the panel's TAIL-conflict regression (was Some SEnd  *)
(*     under the buggy fold; must be None) and an unprojectable  *)
(*     body.                                                      *)
Definition gchoice_agree : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 1 VTNat GEnd)
           (GBcons 1 (GMsg 0 1 VTNat GEnd) GBnil)).
Example proj_agree_uninvolved : proj gchoice_agree 2 = Some SEnd.
Proof. reflexivity. Qed.

Definition gchoice_disagree : gty :=
  GBra 0 1 (GBcons 0 (GMsg 2 0 VTNat GEnd)
           (GBcons 1 (GMsg 0 2 VTNat GEnd) GBnil)).
Example proj_disagree_uninvolved_none : proj gchoice_disagree 2 = None.
Proof. reflexivity. Qed.

(* Tail-conflict regression (LOCKS the B1 None-propagation fix):  *)
(* branch 0 leaves role 2 idle (SEnd), branch 1 has role 2 SEND,  *)
(* branch 2 has role 2 RECV — the tail conflict must NOT be       *)
(* erased to Some SEnd; the honest answer is None.                *)
Definition gtail_conflict : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 1 VTNat GEnd)
           (GBcons 1 (GMsg 2 0 VTNat GEnd)
           (GBcons 2 (GMsg 0 2 VTNat GEnd) GBnil))).
Example proj_tail_conflict_none : proj gtail_conflict 2 = None.
Proof. reflexivity. Qed.

Example proj_unprojectable_body_none :
  proj (GBra 0 1 (GBcons 0 gchoice_disagree GBnil)) 2 = None.
Proof. reflexivity. Qed.

(* (c) Choice + recursion choreography DUALITY (projection_duality*)
(*     instantiated). Both participants act in each, so no        *)
(*     uninvolved mu X.X artefact is ever displayed (fence b).    *)
Definition gchoice2 : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 1 VTNat GEnd)
           (GBcons 1 (GMsg 1 0 VTBool GEnd) GBnil)).
Example proj_gchoice2_0 :
  proj gchoice2 0
  = Some (SSelect (SBcons 0 (SSend VTNat SEnd)
                  (SBcons 1 (SRecv VTBool SEnd) SBnil))).
Proof. reflexivity. Qed.
Example proj_gchoice2_1 :
  proj gchoice2 1
  = Some (SBranch (SBcons 0 (SRecv VTNat SEnd)
                  (SBcons 1 (SSend VTBool SEnd) SBnil))).
Proof. reflexivity. Qed.
Example two_party_gchoice2 : two_party 0 1 gchoice2.
Proof.
  apply TP_BraPQ. apply TPB_cons; [ apply TP_PQ; apply TP_End | ].
  apply TPB_cons; [ apply TP_QP; apply TP_End | apply TPB_nil ].
Qed.
Example gchoice2_dual : proj gchoice2 0 = option_map dual (proj gchoice2 1).
Proof. apply (projection_duality 0 1 gchoice2); [ discriminate | apply two_party_gchoice2 ]. Qed.

(* The two-party RECURSIVE choreography  mu X. 0 -> 1 : Nat . X.  *)
Definition grec : gty := GMu (GMsg 0 1 VTNat (GVar 0)).
Example proj_grec_0 : proj grec 0 = Some (SMu (SSend VTNat (SVar 0))).
Proof. reflexivity. Qed.
Example proj_grec_1 : proj grec 1 = Some (SMu (SRecv VTNat (SVar 0))).
Proof. reflexivity. Qed.
Example two_party_grec : two_party 0 1 grec.
Proof. apply TP_Mu. apply TP_PQ. apply TP_Var. Qed.
Example grec_dual : proj grec 0 = option_map dual (proj grec 1).
Proof. apply (projection_duality 0 1 grec); [ discriminate | apply two_party_grec ]. Qed.

(* ============================================================ *)
(* S1.3c — structural congruence preserves typing (open proc).  *)
(*                                                              *)
(* On the OPEN `proc` calculus we prove that the structural     *)
(* congruence laws PRESERVE the linear channel-typing `wt`:     *)
(*   wt G D P  ->  cong P Q  ->  wt G D Q   (wt_congr).          *)
(*                                                              *)
(* Faithfulness fence: this is TYPING-PRESERVATION under ≡, NOT *)
(* subject-reduction-up-to-≡ (there is no reduction in the      *)
(* statement) — the open `proc` form has no closed SR for the   *)
(* S1.1b reason (the `PRes` ν binder pins the protocol). `cong` *)
(* contains ONLY the par laws P|0≡P, P|Q≡Q|P, (P|Q)|R≡P|(Q|R)   *)
(* (each preserved in BOTH directions; full equivalence). OUT   *)
(* (named, with reasons): ν-extrusion (νc)(P|Q)≡(νc)P|Q needs   *)
(* a free-name function `fn` that does not exist here, so its   *)
(* side-condition is unstatable; ν-swap needs an absent context *)
(* permutation lemma; replication *P≡P|*P has no `!P`/`*P`      *)
(* constructor — vacuously absent.  Echo-types: NOT-RELEVANT.   *)
(* ============================================================ *)

(* ----- context-splitting algebra ----- *)

Lemma csplit_comm : forall D D1 D2, csplit D D1 D2 -> csplit D D2 D1.
Proof.
  intros D D1 D2 H; induction H.
  - apply Csp_nil.
  - apply Csp_r; assumption.
  - apply Csp_l; assumption.
Qed.

Lemma csplit_nil_r : forall D, csplit D D [].
Proof. induction D; [ apply Csp_nil | apply Csp_l; assumption ]. Qed.

Lemma csplit_ended : forall D D1 D2,
  csplit D D1 D2 -> ended D1 -> ended D2 -> ended D.
Proof.
  intros D D1 D2 H; induction H; intros He1 He2.
  - exact He1.
  - inversion He1; subst. constructor; [ assumption | apply IHcsplit; assumption ].
  - inversion He2; subst. constructor; [ assumption | apply IHcsplit; assumption ].
Qed.

(* Reassociation of the linear split: (D1 (+) D2) (+) D3 regroups   *)
(* to D1 (+) (D2 (+) D3).  The load-bearing algebra lemma.          *)
Lemma csplit_assoc : forall D D12 D3, csplit D D12 D3 ->
  forall D1 D2, csplit D12 D1 D2 ->
  exists D23, csplit D D1 D23 /\ csplit D23 D2 D3.
Proof.
  intros D D12 D3 H.
  induction H as [ | e D' D12' D3' Hrec IH | e D' D12' D3' Hrec IH ];
    intros D1 D2 H12.
  - inversion H12; subst. exists []. split; apply Csp_nil.
  - inversion H12; subst.
    + destruct (IH _ _ ltac:(eassumption)) as [D23 [Ha Hb]].
      exists D23. split; [ apply Csp_l; exact Ha | exact Hb ].
    + destruct (IH _ _ ltac:(eassumption)) as [D23 [Ha Hb]].
      exists (e :: D23). split; [ apply Csp_r; exact Ha | apply Csp_l; exact Hb ].
  - destruct (IH _ _ H12) as [D23 [Ha Hb]].
    exists (e :: D23). split; [ apply Csp_r; exact Ha | apply Csp_r; exact Hb ].
Qed.

(* The par-nil engine: a process well-typed at D1 stays well-typed *)
(* when D1 is enlarged by an ENDED context D2 — the extra ended    *)
(* endpoints thread through the derivation to the PNil leaves.     *)
Lemma wt_absorb_ended : forall G D1 P, wt G D1 P ->
  forall D D2, csplit D D1 D2 -> ended D2 -> wt G D P.
Proof.
  intros G D1 P H.
  induction H as [ G DA Hend
                 | G DA Da Db P1 P2 Hsp HP1 IHP1 HP2 IHP2
                 | G DA Drest p n t v k P0 Hsp Hv HP0 IHP0
                 | G DA Drest p n t k P0 Hsp HP0 IHP0
                 | G DA n s P0 HP0 IHP0 ];
    intros D D2 Hcsp Hend2.
  - apply WT_Nil. eapply csplit_ended; eassumption.
  - destruct (csplit_assoc _ _ _ Hcsp _ _ Hsp) as [Dbd [HaD Hb]].
    eapply WT_Par; [ exact HaD | exact HP1 | apply (IHP2 Dbd D2); assumption ].
  - destruct (csplit_assoc _ _ _ Hcsp _ _ Hsp) as [Drest' [HaD Hb]].
    eapply WT_Send;
      [ exact HaD | exact Hv
      | apply (IHP0 ((p,n,k)::Drest') D2); [ apply Csp_l; exact Hb | exact Hend2 ] ].
  - destruct (csplit_assoc _ _ _ Hcsp _ _ Hsp) as [Drest' [HaD Hb]].
    eapply WT_Recv;
      [ exact HaD
      | apply (IHP0 ((p,n,k)::Drest') D2); [ apply Csp_l; exact Hb | exact Hend2 ] ].
  - apply WT_Res. apply (IHP0 ((Pos,n,s)::(Neg,n,dual s)::D) D2).
    + apply Csp_l. apply Csp_l. exact Hcsp.
    + exact Hend2.
Qed.

(* ----- structural congruence (the typing-preserving par laws) ----- *)

Inductive cong : proc -> proc -> Prop :=
| Cg_refl     : forall P, cong P P
| Cg_sym      : forall P Q, cong P Q -> cong Q P
| Cg_trans    : forall P Q R, cong P Q -> cong Q R -> cong P R
| Cg_par      : forall P P' Q, cong P P' -> cong (PPar P Q) (PPar P' Q)
| Cg_parnil   : forall P, cong (PPar P PNil) P
| Cg_parcomm  : forall P Q, cong (PPar P Q) (PPar Q P)
| Cg_parassoc : forall P Q R, cong (PPar (PPar P Q) R) (PPar P (PPar Q R)).

(* Subject congruence: structurally-congruent processes are typed   *)
(* at the SAME context (proved as an iff so the symmetric closure    *)
(* Cg_sym goes through in one induction).                            *)
Lemma wt_congr_iff : forall P Q, cong P Q -> forall G D, wt G D P <-> wt G D Q.
Proof.
  intros P Q H; induction H; intros G D.
  - (* refl *) split; intro Hwt; exact Hwt.
  - (* sym *) split; intro Hwt;
      [ apply (proj2 (IHcong G D)) | apply (proj1 (IHcong G D)) ]; exact Hwt.
  - (* trans *) split; intro Hwt.
    + apply (proj1 (IHcong2 G D)). apply (proj1 (IHcong1 G D)). exact Hwt.
    + apply (proj2 (IHcong1 G D)). apply (proj2 (IHcong2 G D)). exact Hwt.
  - (* Cg_par : congruence closure on the left of a PPar *)
    split; intro Hwt; apply wt_par_inv in Hwt;
      destruct Hwt as (D1 & D2 & Hsp & HP & HQ).
    + eapply WT_Par; [ exact Hsp | apply (proj1 (IHcong G D1)); exact HP | exact HQ ].
    + eapply WT_Par; [ exact Hsp | apply (proj2 (IHcong G D1)); exact HP | exact HQ ].
  - (* Cg_parnil : P|0 ≡ P *)
    split; intro Hwt.
    + apply wt_par_inv in Hwt. destruct Hwt as (D1 & D2 & Hsp & HP & HQ).
      apply wt_nil_inv in HQ. eapply wt_absorb_ended; eassumption.
    + eapply WT_Par; [ apply csplit_nil_r | exact Hwt | apply WT_Nil; constructor ].
  - (* Cg_parcomm : P|Q ≡ Q|P *)
    split; intro Hwt; apply wt_par_inv in Hwt;
      destruct Hwt as (D1 & D2 & Hsp & HP & HQ);
      apply csplit_comm in Hsp; eapply WT_Par; eassumption.
  - (* Cg_parassoc : (P|Q)|R ≡ P|(Q|R) *)
    split; intro Hwt.
    + apply wt_par_inv in Hwt. destruct Hwt as (Dpq & Dr & Hsp1 & Hpq & HR).
      apply wt_par_inv in Hpq. destruct Hpq as (Dp & Dq & Hsp2 & HP & HQ).
      destruct (csplit_assoc _ _ _ Hsp1 _ _ Hsp2) as [Dqr [Ha Hb]].
      eapply WT_Par; [ exact Ha | exact HP | eapply WT_Par; [ exact Hb | exact HQ | exact HR ] ].
    + apply wt_par_inv in Hwt. destruct Hwt as (Dp & Dqr & Hsp1 & HP & Hqr).
      apply wt_par_inv in Hqr. destruct Hqr as (Dq & Dr & Hsp2 & HQ & HR).
      apply csplit_comm in Hsp1. apply csplit_comm in Hsp2.
      destruct (csplit_assoc _ _ _ Hsp1 _ _ Hsp2) as [Dpq [Ha Hb]].
      apply csplit_comm in Ha. apply csplit_comm in Hb.
      eapply WT_Par;
        [ exact Ha | eapply WT_Par; [ exact Hb | exact HP | exact HQ ] | exact HR ].
Qed.

Theorem wt_congr : forall G D P Q, wt G D P -> cong P Q -> wt G D Q.
Proof. intros G D P Q HP Hc. apply (proj1 (wt_congr_iff P Q Hc G D)). exact HP. Qed.

(* Witness: a well-typed parallel composition stays well-typed       *)
(* after commuting its two sides (a one-line use of wt_congr).        *)
Example wt_congr_comm_witness : forall G D P Q,
  wt G D (PPar P Q) -> wt G D (PPar Q P).
Proof. intros G D P Q H. eapply wt_congr; [ exact H | apply Cg_parcomm ]. Qed.

(* ============================================================ *)
(* S1.3b-core — equi-recursive mu, the TYPE layer.              *)
(*                                                              *)
(* The session-type infrastructure for mu-recursion: type-var   *)
(* shifting/substitution, one-step unfolding, dual passing      *)
(* through mu (dual_unfold — future duality-up-to-unfolding     *)
(* infrastructure for S1.3b-meta; NOT consumed by S2.2's        *)
(* projection_duality, which uses only structural               *)
(* dual(SMu x)=SMu(dual x)), and a depth-indexed `guarded`      *)
(* contractiveness predicate.  All structural / axiom-free.     *)
(*                                                              *)
(* DEFERRED (S1.3b-meta, NOT done here): the equi-recursive     *)
(* TYPING / subject-reduction layer (typing up to unfolding).   *)
(* Naive inversion is unprovable under a `PT_Unfold` rule; it    *)
(* needs every inversion lemma rewritten to an up-to-unfolding   *)
(* relation plus a PT_Unfold-soundness theorem — a separate,     *)
(* larger unit.  S2.2 (DONE) consumed only this type layer       *)
(* (SMu / SVar / guarded for projectability).                    *)
(* No "mu supported / recursive sessions done" claim until       *)
(* S1.3b-meta lands.  Echo-types: NOT-RELEVANT (axis-2).         *)
(* ============================================================ *)

(* Type-variable shift (de Bruijn), mutual over sty/sbranch. *)
Fixpoint tlift (c : nat) (s : sty) : sty :=
  match s with
  | SEnd       => SEnd
  | SSend t k  => SSend t (tlift c k)
  | SRecv t k  => SRecv t (tlift c k)
  | SSelect bs => SSelect (tlift_br c bs)
  | SBranch bs => SBranch (tlift_br c bs)
  | SVar k     => if Nat.ltb k c then SVar k else SVar (S k)
  | SMu s0     => SMu (tlift (S c) s0)
  end
with tlift_br (c : nat) (bs : sbranch) : sbranch :=
  match bs with
  | SBnil           => SBnil
  | SBcons l s rest => SBcons l (tlift c s) (tlift_br c rest)
  end.

(* Type-variable substitution [c := u], mutual over sty/sbranch. *)
Fixpoint tsubst (c : nat) (u : sty) (s : sty) : sty :=
  match s with
  | SEnd       => SEnd
  | SSend t k  => SSend t (tsubst c u k)
  | SRecv t k  => SRecv t (tsubst c u k)
  | SSelect bs => SSelect (tsubst_br c u bs)
  | SBranch bs => SBranch (tsubst_br c u bs)
  | SVar k     => match Nat.compare k c with
                  | Lt => SVar k
                  | Eq => u
                  | Gt => SVar (Nat.pred k)
                  end
  | SMu s0     => SMu (tsubst (S c) (tlift 0 u) s0)
  end
with tsubst_br (c : nat) (u : sty) (bs : sbranch) : sbranch :=
  match bs with
  | SBnil           => SBnil
  | SBcons l s rest => SBcons l (tsubst c u s) (tsubst_br c u rest)
  end.

(* Equi-recursive one-step unfolding: mu a.s  ==  s[mu a.s / a].   *)
(* A plain Definition (one tsubst pass): NO fuel/measure — it      *)
(* terminates even on mu a.a (tsubst is structural on s).          *)
Definition unfold_mu (s : sty) : sty := tsubst 0 (SMu s) s.

(* dual commutes with shift and substitution... *)
Lemma dual_tlift : forall s c, dual (tlift c s) = tlift c (dual s).
Proof.
  fix IH 1.
  intros [ | t k | t k | bs | bs | n | s0 ] c; simpl.
  - reflexivity.
  - rewrite IH; reflexivity.
  - rewrite IH; reflexivity.
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - destruct (Nat.ltb n c); reflexivity.
  - rewrite IH; reflexivity.
Qed.

Lemma dual_tsubst : forall s c u, dual (tsubst c u s) = tsubst c (dual u) (dual s).
Proof.
  fix IH 1.
  intros [ | t k | t k | bs | bs | n | s0 ] c u; simpl.
  - reflexivity.
  - rewrite IH; reflexivity.
  - rewrite IH; reflexivity.
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - f_equal. induction bs as [ | l s' rest IHrest ]; simpl;
      [ reflexivity | rewrite IH, IHrest; reflexivity ].
  - destruct (Nat.compare n c); reflexivity.
  - rewrite IH. rewrite dual_tlift. reflexivity.
Qed.

(* dual passes cleanly through unfolding.  Infrastructure for      *)
(* future duality-UP-TO-UNFOLDING (S1.3b-meta); NOT consumed by    *)
(* S2.2's projection_duality (that uses only structural            *)
(* dual(SMu x)=SMu(dual x), no unfolding).                          *)
Lemma dual_unfold : forall s, dual (unfold_mu s) = unfold_mu (dual s).
Proof. intro s. unfold unfold_mu. rewrite dual_tsubst. reflexivity. Qed.

(* Depth-indexed contractiveness: [guarded d s] holds when every   *)
(* SVar referring to one of the innermost d mu-binders occurs      *)
(* UNDER a message/choice prefix.  A message prefix "resets" d to  *)
(* 0 (everything below it is guarded); a mu adds an unguarded      *)
(* binder.  This is the projectability side-condition S2.2 needs;  *)
(* it is carried in WELL-FORMEDNESS, never in the typing rules.    *)
Inductive guarded : nat -> sty -> Prop :=
| Gd_end    : forall d, guarded d SEnd
| Gd_var    : forall d k, k >= d -> guarded d (SVar k)
| Gd_send   : forall d t s, guarded 0 s -> guarded d (SSend t s)
| Gd_recv   : forall d t s, guarded 0 s -> guarded d (SRecv t s)
| Gd_select : forall d bs, guarded_br bs -> guarded d (SSelect bs)
| Gd_branch : forall d bs, guarded_br bs -> guarded d (SBranch bs)
| Gd_mu     : forall d s, guarded (S d) s -> guarded d (SMu s)
with guarded_br : sbranch -> Prop :=
| Gdb_nil  : guarded_br SBnil
| Gdb_cons : forall l s rest, guarded 0 s -> guarded_br rest -> guarded_br (SBcons l s rest).

(* ----- executable witnesses for the mu type layer ----- *)

(* Unfolding the recursive ping-pong type mu a.!Nat.a yields       *)
(* !Nat.(mu a.!Nat.a) — by reflexivity.                            *)
Example unfold_mu_compute :
  unfold_mu (SSend VTNat (SVar 0)) = SSend VTNat (SMu (SSend VTNat (SVar 0))).
Proof. reflexivity. Qed.

(* dual_unfold instantiated on that type. *)
Example dual_unfold_pingpong :
  dual (unfold_mu (SSend VTNat (SVar 0))) = unfold_mu (SRecv VTNat (SVar 0)).
Proof. apply dual_unfold. Qed.

(* mu a.!Nat.a is guarded (contractive); mu a.a is not. *)
Example guarded_recursive : guarded 0 (SMu (SSend VTNat (SVar 0))).
Proof. apply Gd_mu. apply Gd_send. apply Gd_var. apply Nat.le_0_l. Qed.

Example not_guarded_muvar : ~ guarded 0 (SMu (SVar 0)).
Proof.
  intro H. inversion H; subst.
  match goal with G : guarded _ (SVar _) |- _ => inversion G; subst end.
  lia.
Qed.

(* ============================================================ *)
(* S3a — n-party PROJECTION TOTALITY (restricted / plain merge). *)
(*                                                              *)
(* The FIRST theorem that genuinely quantifies over an n>=3     *)
(* role space, breaking the standing fence that no prior        *)
(* theorem quantified over a genuine n-party system. The        *)
(* keystone `projection_total` says: every role of a well-      *)
(* branched global type projects to SOME local type.            *)
(*                                                              *)
(* HONEST NAME: the predicate is `projectable_wf` — it asserts  *)
(* projection EXISTENCE only, NOT session safety. It is NOT     *)
(* MPST "coherence" (which would imply n-party subject          *)
(* reduction + progress); no such safety theorem attaches.      *)
(*                                                              *)
(* FENCES (S3a — each literally true vs the code):              *)
(*  (1) projectable_wf = every-role-projects, NOT session-      *)
(*      safety. No n-party SR / progress / fidelity.            *)
(*  (2) The uninvolved-role MERGE is the PLAIN / identity-meet  *)
(*      merge: choices where an uninvolved role behaves         *)
(*      DIFFERENTLY across branches are EXCLUDED (the SAFE MPST  *)
(*      pattern admitted by full label-UNION merge — deferred   *)
(*      to S3c). Witnessed both ways below.                     *)
(*  (3) Non-empty choice REQUIRED: projectable_wf forbids       *)
(*      `GBra _ _ GBnil` (proj_uninv GBnil = None).             *)
(*  (4) mu IS allowed in the predicate (PW_Mu) and T1 handles   *)
(*      it, but projection stays UNPRUNED — see the S2.2        *)
(*      fence (b); guardedness-of-projection is NOT imposed.    *)
(*  (5) The general `two_party p q G -> projectable_wf G` is    *)
(*      NOT proved — and is in fact FALSE: two_party is         *)
(*      strictly MORE permissive (it admits empty choices and   *)
(*      branches with DIVERGENT mu-structure, on which an       *)
(*      absent role's unpruned projections (mu X.end vs end)    *)
(*      fail to merge). The n=2 collapse is witnessed BY        *)
(*      EXAMPLE instead (gpingpong is projectable_wf).          *)
(*  (6) No n-party config / no operational semantics (S3b/S3c). *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE).          *)
(* ============================================================ *)

(* projectable_wf : the restricted (plain-merge) projectability *)
(* well-formedness. mutual with projectable_wf_br over branches.*)
Inductive projectable_wf : gty -> Prop :=
| PW_End : projectable_wf GEnd
| PW_Var : forall n, projectable_wf (GVar n)
| PW_Msg : forall p q t G,
    p <> q -> projectable_wf G -> projectable_wf (GMsg p q t G)
| PW_Bra : forall p q bs,
    p <> q ->
    bs <> GBnil ->                                   (* non-empty choice *)
    projectable_wf_br bs ->                           (* body coherence (inductive) *)
    (forall r, r <> p -> r <> q ->                    (* MERGE-EXISTENCE clause: *)
       exists s, proj_uninv bs r = Some s) ->         (* every uninvolved role merges *)
    projectable_wf (GBra p q bs)
| PW_Mu : forall G, projectable_wf G -> projectable_wf (GMu G)
with projectable_wf_br : gbranch -> Prop :=
| PWb_nil  : projectable_wf_br GBnil
| PWb_cons : forall l G rest,
    projectable_wf G -> projectable_wf_br rest ->
    projectable_wf_br (GBcons l G rest).

Scheme projectable_wf_mut := Induction for projectable_wf Sort Prop
  with projectable_wf_br_mut := Induction for projectable_wf_br Sort Prop.

(* A message node is ALWAYS projectable when its tail is        *)
(* (used to discharge the merge-existence clause in witnesses). *)
Lemma proj_msg_total : forall p q t G r,
  (exists s, proj G r = Some s) -> exists s, proj (GMsg p q t G) r = Some s.
Proof.
  intros p q t G r [s Hs]. cbn [proj].
  destruct (r =? p). { exists (SSend t s). rewrite Hs. reflexivity. }
  destruct (r =? q). { exists (SRecv t s). rewrite Hs. reflexivity. }
  exists s. exact Hs.
Qed.

Lemma proj_msg_end_total : forall p q t r,
  exists s, proj (GMsg p q t GEnd) r = Some s.
Proof. intros. apply proj_msg_total. exists SEnd. reflexivity. Qed.

(* ===== T1 — THE KEYSTONE: n-party projection totality ===== *)
(* Every role of a projectable_wf global type projects. The    *)
(* `forall r : role` genuinely quantifies over ALL roles —     *)
(* this is the first n>=3 statement in the development.        *)
Theorem projection_total :
  forall G, projectable_wf G -> forall r : role, exists s, proj G r = Some s.
Proof.
  intros G Hwf.
  induction Hwf as
    [ | n | p q t G0 Hpq Hsub IH | p q bs Hpq Hne Hsub IHbr Hmerge
    | G0 Hsub IH | | l G0 rest Hsg IHg Hsr IHr ]
    using projectable_wf_mut
    with (P0 := fun bs (_ : projectable_wf_br bs) =>
                  forall r, exists sb, proj_br bs r = Some sb).
  - (* PW_End *) intro r. exists SEnd. reflexivity.
  - (* PW_Var *) intro r. exists (SVar n). reflexivity.
  - (* PW_Msg *) intro r. destruct (IH r) as [s' Hs'].
    cbn [proj]. destruct (r =? p).
    + exists (SSend t s'). rewrite Hs'. reflexivity.
    + destruct (r =? q).
      * exists (SRecv t s'). rewrite Hs'. reflexivity.
      * exists s'. exact Hs'.
  - (* PW_Bra *) intro r. cbn [proj]. destruct (r =? p) eqn:Ep.
    + destruct (IHbr r) as [sb Hsb]. exists (SSelect sb). rewrite Hsb. reflexivity.
    + destruct (r =? q) eqn:Eq.
      * destruct (IHbr r) as [sb Hsb]. exists (SBranch sb). rewrite Hsb. reflexivity.
      * apply Nat.eqb_neq in Ep. apply Nat.eqb_neq in Eq.
        exact (Hmerge r Ep Eq).
  - (* PW_Mu *) intro r. destruct (IH r) as [s' Hs'].
    exists (SMu s'). cbn [proj]. rewrite Hs'. reflexivity.
  - (* PWb_nil *) intro r. exists SBnil. reflexivity.
  - (* PWb_cons *) intro r.
    destruct (IHg r) as [s Hs]. destruct (IHr r) as [sb Hsb].
    exists (SBcons l s sb). cbn [proj_br]. rewrite Hs, Hsb. reflexivity.
Qed.

(* ===== witnesses: NON-VACUITY at n>=3 + the merge boundary ===== *)

(* n=2 collapse BY EXAMPLE: a binary choreography is in-domain.  *)
Example projectable_gpingpong : projectable_wf gpingpong.
Proof. apply PW_Msg; [ discriminate | apply PW_End ]. Qed.

(* W1 — a genuine 3-party RING: projects on all three roles and  *)
(* is provably NOT two_party (three distinct directed pairs).    *)
Definition g_ring : gty :=
  GMsg 0 1 VTNat (GMsg 1 2 VTNat (GMsg 2 0 VTNat GEnd)).
Example projectable_g_ring : projectable_wf g_ring.
Proof.
  apply PW_Msg; [ discriminate | ].
  apply PW_Msg; [ discriminate | ].
  apply PW_Msg; [ discriminate | apply PW_End ].
Qed.
Example g_ring_proj0 : proj g_ring 0 = Some (SSend VTNat (SRecv VTNat SEnd)).
Proof. reflexivity. Qed.
(* T1 INSTANTIATED at n=3: every role of the ring projects.      *)
Example g_ring_projects_all : forall r, exists s, proj g_ring r = Some s.
Proof. apply projection_total. apply projectable_g_ring. Qed.
Example g_ring_not_two_party : ~ two_party 0 1 g_ring.
Proof.
  unfold g_ring. intro H. inversion H; subst.
  match goal with Hi : two_party _ _ (GMsg 1 2 _ _) |- _ => inversion Hi end.
Qed.

(* W2 — the REAL non-vacuity witness: a 3-party CHOICE whose     *)
(* uninvolved role 2 AGREES across branches, so it projects via  *)
(* a NON-trivial (non-SEnd) merge (merge_idem on identical       *)
(* branches). Provably not two_party (role 2 is in the bodies).  *)
Definition g_choice3 : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 2 VTNat GEnd)
           (GBcons 1 (GMsg 0 2 VTNat GEnd) GBnil)).
Example projectable_g_choice3 : projectable_wf g_choice3.
Proof.
  apply PW_Bra.
  - discriminate.
  - discriminate.
  - apply PWb_cons; [ apply PW_Msg; [ discriminate | apply PW_End ] | ].
    apply PWb_cons; [ apply PW_Msg; [ discriminate | apply PW_End ] | apply PWb_nil ].
  - intros r Hr0 Hr1.
    destruct (proj_msg_end_total 0 2 VTNat r) as [v Hv].
    exists v. cbn [proj_uninv]. rewrite Hv. apply merge_idem.
Qed.
Example g_choice3_proj2 : proj g_choice3 2 = Some (SRecv VTNat SEnd).
Proof. reflexivity. Qed.
Example g_choice3_not_two_party : ~ two_party 0 1 g_choice3.
Proof.
  unfold g_choice3. intro H. inversion H; subst.
  match goal with Hb : two_party_br 0 1 _ |- _ => inversion Hb; subst end.
  match goal with Hm : two_party 0 1 (GMsg 0 2 _ _) |- _ => inversion Hm end.
Qed.

(* W3 — EXCLUDED-but-LEGITIMATE (the honesty witness): SAME      *)
(* direction, DIFFERENT PAYLOAD across branches — a SAFE         *)
(* protocol that full-UNION merge admits but plain merge         *)
(* EXCLUDES. proj fails (None) and it is NOT projectable_wf.     *)
Definition g_excluded : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 2 VTNat  GEnd)
           (GBcons 1 (GMsg 0 2 VTBool GEnd) GBnil)).
Example g_excluded_proj2_none : proj g_excluded 2 = None.
Proof. reflexivity. Qed.
Example g_excluded_not_projectable : ~ projectable_wf g_excluded.
Proof.
  intro H. inversion H; subst.
  match goal with Hm : forall r, r <> 0 -> r <> 1 -> _ |- _ =>
    destruct (Hm 2 ltac:(congruence) ltac:(congruence)) as [s Hs] end.
  cbn in Hs. discriminate.
Qed.

(* The S2.2 disagree witness, recast from below the new          *)
(* predicate: a direction-divergent uninvolved role => excluded. *)
Example gchoice_disagree_not_projectable : ~ projectable_wf gchoice_disagree.
Proof.
  intro H. inversion H; subst.
  match goal with Hm : forall r, r <> 0 -> r <> 1 -> _ |- _ =>
    destruct (Hm 2 ltac:(congruence) ltac:(congruence)) as [s Hs] end.
  cbn in Hs. discriminate.
Qed.

(* ============================================================ *)
(* S3b — STATIC n-party CONFIGURATION (axis-2 STRUCTURE).        *)
(*                                                              *)
(* The FIRST n-party CONFIGURATION form: a static role->endpoint *)
(* assignment.  S3a gave projection TOTALITY (the type side is   *)
(* defined on every role); S3b gives the CONTAINER (an actual    *)
(* assembly of endpoint PROCESSES).  There is NO operational     *)
(* semantics here — the dynamics over an n-party assembly is S3c.*)
(*                                                              *)
(* FENCES (each literally true vs the code):                    *)
(*  (1) wf_assignment asserts ONLY that every LISTED endpoint   *)
(*      is typed at G's projection on its role.  It does NOT     *)
(*      assert n-party safety: no n-party subject reduction, no  *)
(*      n-party progress, no n-party fidelity, no deadlock-      *)
(*      freedom for n>=3, and no global compatibility / coherence*)
(*      among the listed endpoints.  (Despite the wf_ prefix —   *)
(*      same convention as projectable_wf: existence / typing,   *)
(*      NOT safety.)                                            *)
(*  (2) conf_is_role_assignment2 recovers the binary Conf /      *)
(*      wf_config as the n=2 INSTANCE; it still requires         *)
(*      two_party p q G and p<>q, so it fires only on binary     *)
(*      choreographies.  A SLICE, NOT a general n->2 collapse,   *)
(*      transporting NO metatheory to n>=3.  The only            *)
(*      operational metatheory reached is the BINARY             *)
(*      config_subject_reduction, only via projected_config_wf   *)
(*      inside the n=2 collapse, never over the n-ary list.      *)
(*  (3) STATIC only: NO cstep / no n-party operational semantics *)
(*      / no n-party metatheory in this rung (= S3c).  'config'  *)
(*      here = a static role->endpoint assignment, never an      *)
(*      operationally-stepping system.                          *)
(*  (4) The uninvolved-role merge is the PLAIN / identity-meet   *)
(*      merge inherited from S2.2/S3a (NOT label-UNION).  Since  *)
(*      wf_assignment ranges only over roles whose proj is Some, *)
(*      g_excluded-class (same-direction different-payload)      *)
(*      protocols are silently excluded (their proj is None, so  *)
(*      the exists-clause is unprovable).  Full-union = S3c.     *)
(*  (5) mu is UNPRUNED (S2.2 fence b / S3a fence 3 inherited).   *)
(*      Combined with (1), an open GVar / GMu role projects to   *)
(*      an SVar / SMu type that NO closed party inhabits — so    *)
(*      there is NO general 'projectable_wf G -> exists covering *)
(*      wf_assignment' theorem (it is FALSE — witnessed by       *)
(*      projectable_but_uncoverable).  Coverage / inhabitation   *)
(*      is witnessed BY EXAMPLE only.                            *)
(*  (6) role_assignment is an association LIST, NOT a verified    *)
(*      map: the obligation is POINTWISE via In (NOT nth / NOT   *)
(*      length), is DUPLICATE-TOLERANT (witnessed:               *)
(*      wf_assignment_admits_duplicate_keys), and is vacuously   *)
(*      true on []; non-vacuity is carried by the non-empty      *)
(*      g_ring witness.  ra_get is a convenience whose only      *)
(*      sanctioned bridge to truth is ra_get_in.  No coverage    *)
(*      of all of G's roles is claimed.                          *)
(*  (7) The S3a-trap analogue is avoided BY EXAMPLE, not by a    *)
(*      false theorem: NO 'wf_assignment -> (any global safety)' *)
(*      and NO uniqueness / functionality theorem is stated —    *)
(*      both are refuted by the witnesses below.                *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE — the       *)
(*  assignment layer emits no obligation / residue / attestation)*)
(* ============================================================ *)

(* A finite map role -> endpoint process, as an association list. *)
Definition role_assignment := list (role * party).

(* wf_assignment G ra : every listed endpoint is typed at G's     *)
(* projection on its role.  RELATIONAL, via In-membership.  This  *)
(* asserts ONLY per-endpoint local typing-at-projection; it does  *)
(* NOT assert n-party safety / progress / subject reduction.  A   *)
(* plain Definition over a Prop: NOT a Fixpoint (it would have to *)
(* decide pty, an undecidable Inductive Prop) and NOT an Inductive*)
(* (a derived In-obligation is cleaner and needs no Scheme), so   *)
(* the S1.3a list-mutual guard/positivity wall does not apply.    *)
Definition wf_assignment (G : gty) (ra : role_assignment) : Prop :=
  forall r P, In (r, P) ra ->
    exists s, proj G r = Some s /\ pty [] P s.

(* KEYSTONE — the n=2 INSTANCE: the binary Conf / wf_config is    *)
(* recovered from a two_party choreography + p<>q + the two role  *)
(* entries in a wf_assignment, reusing projected_config_wf.  The  *)
(* two destructs yield the SAME sp / sq that feed                 *)
(* projected_config_wf, which re-derives sq = dual sp internally  *)
(* (proj_q_dual_p_some) — no duality re-asserted, no mismatch.    *)
Theorem conf_is_role_assignment2 :
  forall p q G P Q ra,
    p <> q -> two_party p q G ->
    In (p, P) ra -> In (q, Q) ra ->
    wf_assignment G ra ->
    wf_config (Conf P Q).
Proof.
  intros p q G P Q ra Hpq Htp HinP HinQ Hwf.
  destruct (Hwf p P HinP) as [sp [Hpp Htp_p]].
  destruct (Hwf q Q HinQ) as [sq [Hpq2 Htp_q]].
  apply (projected_config_wf p q G P Q sp sq Hpq Htp Hpp Hpq2 Htp_p Htp_q).
Qed.

(* CONVERSE embed: from the binary data, build a 2-element         *)
(* wf_assignment.  p<>q is NOT needed (In is duplicate-tolerant —  *)
(* the two clauses are independent).  Together with the keystone   *)
(* this makes the binary metatheory exactly the n=2 SLICE.         *)
Theorem role_assignment2_of_conf :
  forall p q G P Q sp sq,
    proj G p = Some sp -> proj G q = Some sq ->
    pty [] P sp -> pty [] Q sq ->
    wf_assignment G [(p, P); (q, Q)].
Proof.
  intros p q G P Q sp sq Hpp Hqq HP HQ.
  intros r R Hin. cbn [In] in Hin.
  destruct Hin as [E | [E | F]].
  - injection E as Er ER. subst r R. exists sp. split; assumption.
  - injection E as Er ER. subst r R. exists sq. split; assumption.
  - contradiction.
Qed.

(* ---- functional lookup, related to In WITHOUT no-duplicates ---- *)
Fixpoint ra_get (ra : role_assignment) (r : role) : option party :=
  match ra with
  | [] => None
  | (r', P) :: rest => if Nat.eqb r r' then Some P else ra_get rest r
  end.

(* ra_get found => In.  (No duplicate-freeness required.)  This is *)
(* the ONLY sanctioned bridge from the convenience lookup to truth.*)
Lemma ra_get_in : forall ra r P,
  ra_get ra r = Some P -> In (r, P) ra.
Proof.
  induction ra as [| [r' P'] rest IH]; intros r P H.
  - cbn in H. discriminate.
  - cbn in H. destruct (Nat.eqb r r') eqn:E.
    + apply Nat.eqb_eq in E. subst r'. injection H as H. subst P'. left. reflexivity.
    + right. apply IH. exact H.
Qed.

(* Well-formedness transfers to any successful lookup. *)
Lemma wf_assignment_get : forall G ra r P,
  wf_assignment G ra -> ra_get ra r = Some P ->
  exists s, proj G r = Some s /\ pty [] P s.
Proof.
  intros G ra r P Hwf Hget.
  apply Hwf. apply ra_get_in. exact Hget.
Qed.

(* ---- n=3 witnesses: genuine 3-party static configs ---- *)
(* Endpoints for the g_ring roles, typed at the ring projections. *)
Definition ring_P0 : party := QSend (VNat 1) (QRecv QEnd).  (* !Nat.?Nat.end *)
Definition ring_P1 : party := QRecv (QSend (VNat 2) QEnd).  (* ?Nat.!Nat.end *)
Definition ring_P2 : party := QRecv (QSend (VNat 3) QEnd).  (* ?Nat.!Nat.end *)

Definition ra_ring : role_assignment := [(0, ring_P0); (1, ring_P1); (2, ring_P2)].

Example ring_P0_typed : pty [] ring_P0 (SSend VTNat (SRecv VTNat SEnd)).
Proof. apply PT_Send; [ apply VT_Nat | apply PT_Recv; apply PT_End ]. Qed.

Example proj_ring_1 : proj g_ring 1 = Some (SRecv VTNat (SSend VTNat SEnd)).
Proof. reflexivity. Qed.
(* role 2: receives Nat from 1, then sends Nat to 0 (GMsg 2 0). *)
Example proj_ring_2 : proj g_ring 2 = Some (SRecv VTNat (SSend VTNat SEnd)).
Proof. reflexivity. Qed.

(* A genuine 3-party STATIC config: every role's endpoint typed   *)
(* at its projection of g_ring.  Three In-cases + the [] tail.    *)
Example wf_ra_ring : wf_assignment g_ring ra_ring.
Proof.
  intros r P Hin. cbn [In ra_ring] in Hin.
  destruct Hin as [E | [E | [E | F]]].
  - injection E as Er EP. subst r P. exists (SSend VTNat (SRecv VTNat SEnd)).
    split; [ reflexivity | apply ring_P0_typed ].
  - injection E as Er EP. subst r P. exists (SRecv VTNat (SSend VTNat SEnd)).
    split; [ reflexivity | apply PT_Recv; apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
  - injection E as Er EP. subst r P. exists (SRecv VTNat (SSend VTNat SEnd)).
    split; [ reflexivity | apply PT_Recv; apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
  - contradiction.
Qed.

(* This 3-party config is NOT a single Conf: it has three roles,  *)
(* and g_ring is provably not two_party 0 1 (g_ring_not_two_party)*)
(* — shown BY EXAMPLE (length + the existing refutation), never   *)
(* via an over-conditioned 'wf with >2 entries -> not two_party'. *)
Example ra_ring_three_entries : length ra_ring = 3.
Proof. reflexivity. Qed.

(* ---- honesty witnesses: the soundness boundary, by example ---- *)

(* wf_assignment is DUPLICATE-TOLERANT (In, not a map): two        *)
(* DISTINCT endpoints at role 0, both typed at proj gpingpong 0.   *)
(* Ship this REFUTATION instead of any (false) functionality /     *)
(* config-determined-by-G theorem.                                 *)
Example wf_assignment_admits_duplicate_keys :
  wf_assignment gpingpong [(0, QSend (VNat 7) QEnd); (0, QSend (VNat 9) QEnd)]
  /\ QSend (VNat 7) QEnd <> QSend (VNat 9) QEnd.
Proof.
  split.
  - intros r P Hin. cbn [In] in Hin. destruct Hin as [E | [E | F]].
    + injection E as Er EP. subst r P. exists (SSend VTNat SEnd).
      split; [ reflexivity | apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
    + injection E as Er EP. subst r P. exists (SSend VTNat SEnd).
      split; [ reflexivity | apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
    + contradiction.
  - intro E. injection E as E. discriminate.
Qed.

(* The S3a-trap analogue, refuted: projectable does NOT imply      *)
(* coverable.  GVar 3 is projectable_wf, yet proj (GVar 3) 0 =     *)
(* Some (SVar 3), and NO closed party inhabits SVar n.  Ship this, *)
(* NOT a general 'projectable_wf G -> exists covering ra'.         *)
Lemma svar_uninhabited : forall P n, ~ pty [] P (SVar n).
Proof. intros P n H. inversion H. Qed.

Example projectable_but_uncoverable :
  projectable_wf (GVar 3) /\ proj (GVar 3) 0 = Some (SVar 3)
  /\ (forall P, ~ pty [] P (SVar 3)).
Proof.
  split; [ apply PW_Var | ].
  split; [ reflexivity | ].
  intro P. apply svar_uninhabited.
Qed.

(* ============================================================ *)
(* S3c.0 — full label-UNION merge.  ADD ALONGSIDE plain merge.   *)
(* The Honda-Yoshida-Carbone label-union: at an EXTERNAL choice  *)
(* (SBranch &), a merged uninvolved role may offer the UNION of  *)
(* the branches' labels — a SAFE protocol the plain identity-    *)
(* meet `merge` REJECTS (witnessed). Adds NOTHING to proj /      *)
(* projectable_wf / cstep — wiring umerge into projection is the *)
(* SEPARATE S3c.1.  Reuses sty_mut / vty_eqb / bget.             *)
(* ============================================================ *)

(* Remove the FIRST entry keyed l (non-fixpoint helper of umerge).*)
Fixpoint bremove (l : nat) (bs : sbranch) : sbranch :=
  match bs with
  | SBnil => SBnil
  | SBcons k s rest =>
      if Nat.eqb l k then rest else SBcons k s (bremove l rest)
  end.

(* Decidable structural equality on sty/sbranch — so the SSelect *)
(* (internal +) case can require EQUALITY (the HYC-sound rule:   *)
(* NEVER union a SENDER's own choice).                           *)
Fixpoint sty_eqb (a b : sty) {struct a} : bool :=
  match a, b with
  | SEnd, SEnd => true
  | SSend t1 k1, SSend t2 k2 => vty_eqb t1 t2 && sty_eqb k1 k2
  | SRecv t1 k1, SRecv t2 k2 => vty_eqb t1 t2 && sty_eqb k1 k2
  | SSelect b1, SSelect b2 => sbranch_eqb b1 b2
  | SBranch b1, SBranch b2 => sbranch_eqb b1 b2
  | SVar n1, SVar n2 => Nat.eqb n1 n2
  | SMu a1, SMu a2 => sty_eqb a1 a2
  | _, _ => false
  end
with sbranch_eqb (b1 b2 : sbranch) {struct b1} : bool :=
  match b1, b2 with
  | SBnil, SBnil => true
  | SBcons l1 s1 r1, SBcons l2 s2 r2 =>
      Nat.eqb l1 l2 && sty_eqb s1 s2 && sbranch_eqb r1 r2
  | _, _ => false
  end.

Lemma sty_eqb_refl : forall s, sty_eqb s s = true.
Proof.
  intro s. induction s using sty_mut
    with (P0 := fun bs => sbranch_eqb bs bs = true); simpl; try reflexivity.
  - rewrite vty_eqb_refl, IHs. reflexivity.   (* SSend *)
  - rewrite vty_eqb_refl, IHs. reflexivity.   (* SRecv *)
  - exact IHs.                                (* SSelect *)
  - exact IHs.                                (* SBranch *)
  - apply Nat.eqb_refl.                       (* SVar *)
  - exact IHs.                                (* SMu *)
  - rewrite Nat.eqb_refl, IHs, IHs0. reflexivity.   (* SBcons *)
Qed.
Lemma sbranch_eqb_refl : forall bs, sbranch_eqb bs bs = true.
Proof. intro bs. exact (sty_eqb_refl (SSelect bs)). Qed.

(* umerge : the label-UNION merge.                               *)
(*  message fragment (SEnd/SSend/SRecv/SVar/SMu) : EQUALITY.     *)
(*  SSelect (internal +) : EQUALITY (HYC-sound — a SENDER's      *)
(*    choice is never unioned).                                  *)
(*  SBranch (external &) : LABEL UNION — shared labels' conts     *)
(*    recursively umerged, unshared labels carried over.         *)
(* GUARD: umerge_br {struct b1} recurses on b1; b2 is threaded   *)
(* through bremove (a NON-fixpoint subroutine), so every         *)
(* recursive call's principal arg is a strict subterm of b1.     *)
(* Coq 8.18 ACCEPTS this (verified).                             *)
Fixpoint umerge (s1 s2 : sty) {struct s1} : option sty :=
  match s1, s2 with
  | SEnd, SEnd => Some SEnd
  | SSend t1 k1, SSend t2 k2 =>
      if vty_eqb t1 t2
      then match umerge k1 k2 with Some k => Some (SSend t1 k) | None => None end
      else None
  | SRecv t1 k1, SRecv t2 k2 =>
      if vty_eqb t1 t2
      then match umerge k1 k2 with Some k => Some (SRecv t1 k) | None => None end
      else None
  | SSelect b1, SSelect b2 =>
      if sbranch_eqb b1 b2 then Some (SSelect b1) else None
  | SBranch b1, SBranch b2 =>
      match umerge_br b1 b2 with Some b => Some (SBranch b) | None => None end
  | SVar n1, SVar n2 => if Nat.eqb n1 n2 then Some (SVar n1) else None
  | SMu a1, SMu a2 =>
      match umerge a1 a2 with Some a => Some (SMu a) | None => None end
  | _, _ => None
  end
with umerge_br (b1 b2 : sbranch) {struct b1} : option sbranch :=
  match b1 with
  | SBnil => Some b2                       (* carry over b2-only labels *)
  | SBcons l1 s1 r1 =>
      match bget l1 b2 with
      | Some s2 =>                         (* SHARED label: umerge conts *)
          match umerge s1 s2, umerge_br r1 (bremove l1 b2) with
          | Some s, Some r => Some (SBcons l1 s r)
          | _, _ => None
          end
      | None =>                           (* b1-only label: carry s1 *)
          match umerge_br r1 b2 with
          | Some r => Some (SBcons l1 s1 r)
          | None => None
          end
      end
  end.

(* ===== KEYSTONE: umerge is reflexive (UNCONDITIONAL). =====    *)
(* NOTE (adjudicated empirically): unlike a two-phase-walk        *)
(* encoding, the bremove-threaded design makes idem hold for ALL  *)
(* s — including duplicate-divergent-label branch lists — because *)
(* the head's own key is found (n=?n) and bremove strips it       *)
(* before the tail recursion. No branch_free fence is needed.    *)
Lemma umerge_idem : forall s, umerge s s = Some s.
Proof.
  intro s. induction s using sty_mut
    with (P0 := fun bs => umerge_br bs bs = Some bs); simpl; try reflexivity.
  - rewrite vty_eqb_refl, IHs. reflexivity.   (* SSend *)
  - rewrite vty_eqb_refl, IHs. reflexivity.   (* SRecv *)
  - rewrite sbranch_eqb_refl. reflexivity.    (* SSelect: EQUALITY *)
  - rewrite IHs. reflexivity.                 (* SBranch: union path *)
  - rewrite Nat.eqb_refl. reflexivity.        (* SVar *)
  - rewrite IHs. reflexivity.                 (* SMu *)
  - rewrite Nat.eqb_refl. rewrite IHs, IHs0. reflexivity.  (* SBcons *)
Qed.

(* ===== WITNESSES: the unlocked class + the honest fences ===== *)
(* (i) the WIN: a different-LABEL external choice plain merge     *)
(*     REJECTS, umerge ACCEPTS (the union). *)
Example merge_rejects_disjoint :
  merge (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 1 SEnd SBnil)) = None.
Proof. reflexivity. Qed.
Example umerge_accepts_disjoint :
  umerge (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 1 SEnd SBnil))
    = Some (SBranch (SBcons 0 SEnd (SBcons 1 SEnd SBnil))).
Proof. reflexivity. Qed.
Example umerge_widens_strictly :
  merge   (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 1 SEnd SBnil)) = None
  /\ umerge (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 1 SEnd SBnil)) <> None.
Proof. split; [ reflexivity | discriminate ]. Qed.
Example umerge_overlap :
  umerge (SBranch (SBcons 0 SEnd (SBcons 1 SEnd SBnil)))
         (SBranch (SBcons 1 SEnd (SBcons 2 SEnd SBnil)))
    = Some (SBranch (SBcons 0 SEnd (SBcons 1 SEnd (SBcons 2 SEnd SBnil)))).
Proof. reflexivity. Qed.

(* (ii) HONEST FENCE: SSelect (internal +) is NOT unioned. *)
Example umerge_select_disjoint_rejected :
  umerge (SSelect (SBcons 0 SEnd SBnil)) (SSelect (SBcons 1 SEnd SBnil)) = None.
Proof. reflexivity. Qed.

(* (iii) HONEST FENCE: the EXISTING g_excluded class (same-       *)
(*     direction DIFFERENT-PAYLOAD message) is STILL rejected —   *)
(*     umerge unions LABELS, it does NOT reconcile divergent      *)
(*     message payloads. So S3c.0 does NOT yet unlock g_excluded; *)
(*     it unlocks a DIFFERENT class (different-label & choices).  *)
Example umerge_still_rejects_payload_divergence :
  umerge (SRecv VTNat SEnd) (SRecv VTBool SEnd) = None.
Proof. reflexivity. Qed.

(* (iv) CONSERVATIVE-on-agreement: umerge = merge where branches  *)
(*     AGREE (the overlapping domain).  Witnessed by example;     *)
(*     the full lemma is S3c.0 grind, not claimed here.           *)
Example agree_end  : merge SEnd SEnd = umerge SEnd SEnd.
Proof. reflexivity. Qed.
Example agree_send :
  merge (SSend VTNat SEnd) (SSend VTNat SEnd)
  = umerge (SSend VTNat SEnd) (SSend VTNat SEnd).
Proof. reflexivity. Qed.
Example agree_branch_identical :
  merge (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 0 SEnd SBnil))
  = umerge (SBranch (SBcons 0 SEnd SBnil)) (SBranch (SBcons 0 SEnd SBnil)).
Proof. reflexivity. Qed.

(* ============================================================ *)
(* S3c.0 FENCE (each clause literally true vs the code above):   *)
(*  (1) umerge is a TYPE-ALGEBRA combinator ONLY: sty->sty->     *)
(*      option sty.  It is NOT wired into proj (proj still uses   *)
(*      plain merge — union-projection is the SEPARATE S3c.1).    *)
(*      No theorem here quantifies over a gty, a role, or a       *)
(*      configuration.                                           *)
(*  (2) Keystone is umerge_idem (reflexivity).  It is NOT         *)
(*      commutative syntactically (label ORDER of the union is    *)
(*      argument-order dependent) and NOT proved associative —    *)
(*      only idempotence holds.  Future rungs must reason up to   *)
(*      branch-SET equality, not syntactic sty equality.         *)
(*  (3) NO coherence / safety claim.  The name 'umerge' claims a  *)
(*      defined widening only; it is deliberately NOT             *)
(*      'coherent_merge' / 'safe_merge'.  Safety is EARNED no     *)
(*      earlier than S3c.3 (n-party subject reduction).          *)
(*  (4) The SELECT (+, SSelect) case does NOT widen — it requires *)
(*      EQUALITY (sbranch_eqb).  Unioning a SENDER's internal     *)
(*      choice is HYC-UNSOUND (it would claim the role can select *)
(*      a label with no continuation in one branch).  Only the    *)
(*      external & (SBranch) case unions.  This asymmetry is      *)
(*      load-bearing (umerge_select_disjoint_rejected).          *)
(*  (5) umerge is NOT a syntactic conservative extension of merge:*)
(*      merge s1 s2 = Some s does NOT always give umerge = Some s *)
(*      with the SAME s (branch ORDER can differ).  So proj may   *)
(*      NOT be re-pointed at umerge and reuse projection_total's  *)
(*      proof — S3c.1 introduces a SEPARATE proj_uninv_u and      *)
(*      re-proves totality.  Agreement is witnessed (agree_end /  *)
(*      agree_send / agree_branch_identical).                     *)
(*  (6) The CLASS umerge unlocks is DIFFERENT-LABEL external      *)
(*      choices (&{0:end} vs &{1:end} -> &{0:end,1:end}), which   *)
(*      plain merge rejects (merge_rejects_disjoint).  It does    *)
(*      NOT unlock the EXISTING g_excluded / gchoice_disagree     *)
(*      witnesses — those are same-direction DIFFERENT-PAYLOAD    *)
(*      message clashes (SRecv VTNat vs SRecv VTBool), which      *)
(*      umerge STILL rejects (umerge_still_rejects_payload_       *)
(*      divergence).  Do not over-read the headline.             *)
(*  (7) mu (SMu/SVar) is unioned structurally; divergent mu-      *)
(*      structure across branches still fails to umerge (S2.2     *)
(*      fence b / S3a fence 4 inherited — UNPRUNED mu).           *)
(*  (8) Branch label-sets are NOT required NoDup.  umerge_idem    *)
(*      holds unconditionally regardless (bget/bremove first-     *)
(*      match), but any non-idempotent use on duplicate keys      *)
(*      needs a NoDup side-condition (S3b fence 6 analogue).      *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE — a type-    *)
(*  merge operation emits no obligation / residue / attestation). *)
(* ============================================================ *)

(* ============================================================ *)
(* S3c.1 — UNION-PROJECTION rung.                                *)
(*   Strictly ADDITIVE on top of S3c.0.  Introduces a SEPARATE   *)
(*   union-projection proj_u (proj with the uninvolved fold's    *)
(*   plain merge replaced by umerge), its totality, the          *)
(*   different-LABEL non-vacuity witness, and a ONE-DIRECTIONAL   *)
(*   monotonicity bridge (plain-projectable => union-            *)
(*   projectable, the two projections coinciding there).         *)
(*   proj / merge / projectable_wf / projection_total and ALL    *)
(*   existing witnesses stay BYTE-IDENTICAL — proj is NOT        *)
(*   re-pointed at umerge (S3c.0 FENCE 5 forbids it).            *)
(* ============================================================ *)

(* 1. Single-output induction principles over the global tree.   *)
(*    None pre-exists; needed only by proj_agrees_u.  Harmless.   *)
Scheme gty_mut := Induction for gty Sort Prop
  with gbranch_mut := Induction for gbranch Sort Prop.

(* 2. KEYSTONE: plain merge is the identity-MEET — it succeeds    *)
(*    only on EQUAL arguments.  This is literally the invariant   *)
(*    stated in the merge FENCE comment above; here it is proved. *)
(*    REGRESSION PIN: depends on merge_br being POSITIONAL same-  *)
(*    label and merge being non-reordering.  If merge is ever     *)
(*    generalised to reorder/widen, THIS and the whole            *)
(*    monotonicity chain (proj_agrees_u, proj_uninv_agrees_u,     *)
(*    projectable_wf_implies_u) break.  The existence theorems    *)
(*    (projection_total_u) are independent and survive.           *)
Lemma merge_forces_eq :
  forall s1 s2 s, merge s1 s2 = Some s -> s1 = s2.
Proof.
  intro s1. induction s1 using sty_mut
    with (P0 := fun b1 => forall b2 b, merge_br b1 b2 = Some b -> b1 = b2).
  - intros [|? ?|? ?|?|?|?|?] sR H; simpl in H; try discriminate. reflexivity.
  - intros [|t2 k2|? ?|?|?|?|?] sR H; simpl in H; try discriminate.
    destruct (vty_eqb v t2) eqn:E; try discriminate.
    destruct (merge s1 k2) eqn:M; try discriminate.
    destruct v, t2; simpl in E; try discriminate; erewrite IHs1 by exact M; reflexivity.
  - intros [|? ?|t2 k2|?|?|?|?] sR H; simpl in H; try discriminate.
    destruct (vty_eqb v t2) eqn:E; try discriminate.
    destruct (merge s1 k2) eqn:M; try discriminate.
    destruct v, t2; simpl in E; try discriminate; erewrite IHs1 by exact M; reflexivity.
  - intros [|? ?|? ?|c2|?|?|?] sR H; simpl in H; try discriminate.
    destruct (merge_br s c2) eqn:M; try discriminate. erewrite IHs1 by exact M; reflexivity.
  - intros [|? ?|? ?|?|c2|?|?] sR H; simpl in H; try discriminate.
    destruct (merge_br s c2) eqn:M; try discriminate. erewrite IHs1 by exact M; reflexivity.
  - intros [|? ?|? ?|?|?|n2|?] sR H; simpl in H; try discriminate.
    destruct (Nat.eqb n n2) eqn:E; try discriminate. apply Nat.eqb_eq in E; subst; reflexivity.
  - intros [|? ?|? ?|?|?|?|a2] sR H; simpl in H; try discriminate.
    destruct (merge s1 a2) eqn:M; try discriminate. erewrite IHs1 by exact M; reflexivity.
  - intros [|? ? ?] bR H; simpl in H; try discriminate. reflexivity.
  - intros [|l2 s2 r2] bR H; simpl in H; try discriminate.
    destruct (Nat.eqb n l2) eqn:E; try discriminate. apply Nat.eqb_eq in E; subst.
    destruct (merge s1 s2) eqn:M; try discriminate.
    destruct (merge_br s r2) eqn:Mb; try discriminate.
    f_equal; [ apply (IHs1 _ _ M) | apply (IHs0 _ _ Mb) ].
Qed.

(* 3. The union projection.  VERBATIM clone of proj/proj_br/      *)
(*    proj_uninv with EXACTLY ONE token changed: merge -> umerge  *)
(*    in proj_uninv_u's >=2-branch fold.  Structural guards       *)
(*    mirror proj exactly; umerge is a closed non-recursive       *)
(*    subroutine from the fold's view, so guardedness is fine.    *)
Fixpoint proj_u (G : gty) (r : role) {struct G} : option sty :=
  match G with
  | GEnd          => Some SEnd
  | GMsg p q t G' =>
      if Nat.eqb r p then option_map (SSend t) (proj_u G' r)
      else if Nat.eqb r q then option_map (SRecv t) (proj_u G' r)
      else proj_u G' r
  | GBra p q bs   =>
      if Nat.eqb r p then option_map SSelect (proj_br_u bs r)
      else if Nat.eqb r q then option_map SBranch (proj_br_u bs r)
      else proj_uninv_u bs r
  | GMu G'        => option_map SMu (proj_u G' r)
  | GVar n        => Some (SVar n)
  end
with proj_br_u (bs : gbranch) (r : role) {struct bs} : option sbranch :=
  match bs with
  | GBnil           => Some SBnil
  | GBcons l G rest =>
      match proj_u G r, proj_br_u rest r with
      | Some s, Some sb => Some (SBcons l s sb)
      | _, _ => None end
  end
with proj_uninv_u (bs : gbranch) (r : role) {struct bs} : option sty :=
  match bs with
  | GBnil           => None
  | GBcons l G rest =>
      match rest with
      | GBnil => proj_u G r
      | _     => match proj_u G r, proj_uninv_u rest r with
                 | Some s, Some s' => umerge s s'    (* the ONLY change vs proj_uninv *)
                 | _, _ => None end
      end
  end.

(* 4. Union-projectability.  VERBATIM clone of projectable_wf,    *)
(*    ONE change: PWu_Bra's merge-existence clause uses           *)
(*    proj_uninv_u (not proj_uninv).                              *)
Inductive projectable_u_wf : gty -> Prop :=
| PWu_End : projectable_u_wf GEnd
| PWu_Var : forall n, projectable_u_wf (GVar n)
| PWu_Msg : forall p q t G, p <> q -> projectable_u_wf G -> projectable_u_wf (GMsg p q t G)
| PWu_Bra : forall p q bs,
    p <> q -> bs <> GBnil -> projectable_u_wf_br bs ->
    (forall r, r <> p -> r <> q -> exists s, proj_uninv_u bs r = Some s) ->
    projectable_u_wf (GBra p q bs)
| PWu_Mu : forall G, projectable_u_wf G -> projectable_u_wf (GMu G)
with projectable_u_wf_br : gbranch -> Prop :=
| PWub_nil  : projectable_u_wf_br GBnil
| PWub_cons : forall l G rest, projectable_u_wf G -> projectable_u_wf_br rest -> projectable_u_wf_br (GBcons l G rest).
Scheme projectable_u_wf_mut := Induction for projectable_u_wf Sort Prop
  with projectable_u_wf_br_mut := Induction for projectable_u_wf_br Sort Prop.

(* 5. KEYSTONE existence theorem — the S3c.1 analogue of          *)
(*    projection_total.  Proof = projection_total's proof         *)
(*    VERBATIM with _u renames.  Merge-existence is supplied by   *)
(*    the PWu_Bra hypothesis, so umerge's partiality is never     *)
(*    inspected by the induction.                                 *)
Theorem projection_total_u :
  forall G, projectable_u_wf G -> forall r : role, exists s, proj_u G r = Some s.
Proof.
  intros G Hwf.
  induction Hwf as
    [ | n | p q t G0 Hpq Hsub IH | p q bs Hpq Hne Hsub IHbr Hmerge
    | G0 Hsub IH | | l G0 rest Hsg IHg Hsr IHr ]
    using projectable_u_wf_mut
    with (P0 := fun bs (_ : projectable_u_wf_br bs) =>
                  forall r, exists sb, proj_br_u bs r = Some sb).
  - intro r. exists SEnd. reflexivity.
  - intro r. exists (SVar n). reflexivity.
  - intro r. destruct (IH r) as [s' Hs']. cbn [proj_u]. destruct (r =? p).
    + exists (SSend t s'). rewrite Hs'. reflexivity.
    + destruct (r =? q).
      * exists (SRecv t s'). rewrite Hs'. reflexivity.
      * exists s'. exact Hs'.
  - intro r. cbn [proj_u]. destruct (r =? p) eqn:Ep.
    + destruct (IHbr r) as [sb Hsb]. exists (SSelect sb). rewrite Hsb. reflexivity.
    + destruct (r =? q) eqn:Eq.
      * destruct (IHbr r) as [sb Hsb]. exists (SBranch sb). rewrite Hsb. reflexivity.
      * apply Nat.eqb_neq in Ep. apply Nat.eqb_neq in Eq. exact (Hmerge r Ep Eq).
  - intro r. destruct (IH r) as [s' Hs']. exists (SMu s'). cbn [proj_u]. rewrite Hs'. reflexivity.
  - intro r. exists SBnil. reflexivity.
  - intro r. destruct (IHg r) as [s Hs]. destruct (IHr r) as [sb Hsb].
    exists (SBcons l s sb). cbn [proj_br_u]. rewrite Hs, Hsb. reflexivity.
Qed.

(* 6. AGREEMENT (gty half): on any role where plain proj is       *)
(*    defined, proj_u agrees.  gty_mut is SINGLE-output, so the   *)
(*    gbranch transfers live in the P0 motive (a CONJUNCTION of   *)
(*    the proj_br and proj_uninv transfers); the lemma concludes  *)
(*    the gty half only.  The gbranch-only half is recovered      *)
(*    separately (proj_uninv_agrees_u).  The proj_uninv cons arm  *)
(*    uses merge_forces_eq + merge_idem + umerge_idem.            *)
Lemma proj_agrees_u :
  forall G rr s, proj G rr = Some s -> proj_u G rr = Some s.
Proof.
  intro G.
  induction G as
    [ | p q v Ghead IHG | p q bs [IHbr IHun] | Gbody IHG | n
    | | n ghead IHG gtail [IHbr IHun] ]
    using gty_mut
    with (P0 := fun bs =>
        (forall rr sb, proj_br bs rr = Some sb -> proj_br_u bs rr = Some sb)
        /\ (forall rr s, proj_uninv bs rr = Some s -> proj_uninv_u bs rr = Some s)).
  - (* GEnd *) intros rr s H. cbn in *. exact H.
  - (* GMsg p q v Ghead *) intros rr s H. cbn [proj] in H. cbn [proj_u].
    destruct (rr =? p). { destruct (proj Ghead rr) eqn:Pg; cbn in H; try discriminate.
      rewrite (IHG _ _ Pg). cbn in H. exact H. }
    destruct (rr =? q). { destruct (proj Ghead rr) eqn:Pg; cbn in H; try discriminate.
      rewrite (IHG _ _ Pg). cbn in H. exact H. }
    apply IHG. exact H.
  - (* GBra p q bs *) intros rr s H. cbn [proj] in H. cbn [proj_u].
    destruct (rr =? p). { destruct (proj_br bs rr) eqn:Pb; cbn in H; try discriminate.
      rewrite (IHbr _ _ Pb). cbn in H. exact H. }
    destruct (rr =? q). { destruct (proj_br bs rr) eqn:Pb; cbn in H; try discriminate.
      rewrite (IHbr _ _ Pb). cbn in H. exact H. }
    apply IHun. exact H.
  - (* GMu Gbody *) intros rr s H. cbn [proj] in H. cbn [proj_u].
    destruct (proj Gbody rr) eqn:Pg; cbn in H; try discriminate.
    rewrite (IHG _ _ Pg). cbn in H. exact H.
  - (* GVar n *) intros rr s H. cbn in *. exact H.
  - (* GBnil *) split.
    + intros rr sb H. cbn in *. exact H.
    + intros rr s H. cbn in H. discriminate.
  - (* GBcons n ghead gtail *) split.
    + intros rr sb H. cbn [proj_br] in H. cbn [proj_br_u].
      destruct (proj ghead rr) eqn:Pg; try discriminate.
      destruct (proj_br gtail rr) eqn:Pr; try discriminate.
      rewrite (IHG _ _ Pg). rewrite (IHbr _ _ Pr). exact H.
    + intros rr s H. cbn [proj_uninv] in H. cbn [proj_uninv_u].
      destruct gtail as [|l2 G2 rest2] eqn:Erest.
      * apply IHG. exact H.
      * destruct (proj ghead rr) eqn:Pg; try discriminate.
        destruct (proj_uninv (GBcons l2 G2 rest2) rr) eqn:Pu; try discriminate.
        rewrite (IHG _ _ Pg).
        rewrite (IHun _ _ Pu).
        pose proof (merge_forces_eq _ _ _ H) as Heq. subst s1.
        rewrite merge_idem in H. inversion H. subst s0. apply umerge_idem.
Qed.

(* 7. AGREEMENT (gbranch-only half): the conclusion gty_mut does  *)
(*    NOT directly deliver.  Plain induction on bs; head via      *)
(*    proj_agrees_u, fold via merge_forces_eq.  This is the       *)
(*    lemma projectable_wf_implies_u actually consumes.           *)
Lemma proj_uninv_agrees_u :
  forall bs rr s, proj_uninv bs rr = Some s -> proj_uninv_u bs rr = Some s.
Proof.
  intro bs.
  induction bs as [|l G rest IHrest]; intros rr s H.
  - cbn in H. discriminate.
  - cbn [proj_uninv] in H. cbn [proj_uninv_u].
    destruct rest as [|l2 G2 rest2] eqn:Erest.
    + apply proj_agrees_u. exact H.
    + destruct (proj G rr) eqn:Pg; try discriminate.
      destruct (proj_uninv (GBcons l2 G2 rest2) rr) eqn:Pu; try discriminate.
      rewrite (proj_agrees_u _ _ _ Pg).
      rewrite (IHrest _ _ Pu).
      pose proof (merge_forces_eq _ _ _ H) as Heq. subst s1.
      rewrite merge_idem in H. inversion H. subst s0. apply umerge_idem.
Qed.

(* 8. MONOTONICITY BRIDGE: every plain-projectable global type is *)
(*    union-projectable.  DOMAIN-INCLUSION (one-directional; the  *)
(*    converse is FALSE — g_union3 below).  projectable_wf_mut    *)
(*    induction; the only non-trivial case PWu_Bra discharges its *)
(*    existence clause via proj_uninv_agrees_u on the plain       *)
(*    witness (auto-named 'e' by projectable_wf_mut).             *)
Theorem projectable_wf_implies_u :
  forall G, projectable_wf G -> projectable_u_wf G.
Proof.
  intros G H. induction H using projectable_wf_mut with
    (P0 := fun bs (_ : projectable_wf_br bs) => projectable_u_wf_br bs).
  - apply PWu_End.
  - apply PWu_Var.
  - apply PWu_Msg; assumption.
  - apply PWu_Bra; try assumption.
    intros r Hp Hq. destruct (e r Hp Hq) as [s Hs]. exists s.
    apply proj_uninv_agrees_u. exact Hs.
  - apply PWu_Mu; assumption.
  - apply PWub_nil.
  - apply PWub_cons; assumption.
Qed.

(* 9. NON-VACUITY.  g_union3: uninvolved role 2 sees &{3:end} in  *)
(*    outer-branch 0 and &{4:end} in outer-branch 1 — DIFFERENT   *)
(*    LABELS, same external-choice direction.  Plain merge        *)
(*    rejects (proj = None, ~projectable_wf); umerge UNIONS to    *)
(*    &{3:end,4:end} (proj_u = Some, projectable_u_wf).  This is   *)
(*    genuinely in (union-projectable MINUS plain-projectable).   *)
Definition g_union3 : gty :=
  GBra 0 1 (GBcons 0 (GBra 0 2 (GBcons 3 GEnd GBnil))
           (GBcons 1 (GBra 0 2 (GBcons 4 GEnd GBnil)) GBnil)).
Example proj_union3_role2_plain_none : proj g_union3 2 = None.
Proof. reflexivity. Qed.
Example proj_u_union3_role2_some :
  proj_u g_union3 2 = Some (SBranch (SBcons 3 SEnd (SBcons 4 SEnd SBnil))).
Proof. reflexivity. Qed.
Example g_union3_not_plain_projectable : ~ projectable_wf g_union3.
Proof. intro H. inversion H; subst.
  match goal with Hm : forall r, r<>0 -> r<>1 -> _ |- _ =>
    destruct (Hm 2 ltac:(congruence) ltac:(congruence)) as [s Hs] end.
  cbn in Hs. discriminate. Qed.
Example projectable_u_g_union3 : projectable_u_wf g_union3.
Proof.
  apply PWu_Bra.
  - congruence.
  - discriminate.
  - apply PWub_cons. apply PWu_Bra. congruence. discriminate.
    apply PWub_cons. apply PWu_End. apply PWub_nil.
    intros r Hp Hq. cbn [proj_uninv_u]. exists SEnd. reflexivity.
    apply PWub_cons. apply PWu_Bra. congruence. discriminate.
    apply PWub_cons. apply PWu_End. apply PWub_nil.
    intros r Hp Hq. cbn [proj_uninv_u]. exists SEnd. reflexivity.
    apply PWub_nil.
  - intros r Hp Hq.
    assert (E0 : (r =? 0) = false) by (apply Nat.eqb_neq; congruence).
    cbn [proj_uninv_u proj_u]. rewrite E0.
    destruct (r =? 2) eqn:E2.
    + apply Nat.eqb_eq in E2. subst r.
      exists (SBranch (SBcons 3 SEnd (SBcons 4 SEnd SBnil))). reflexivity.
    + exists SEnd. reflexivity.
Qed.

(* The HONEST-FENCE witness.  Different-PAYLOAD class (role 2     *)
(* sees SRecv VTNat vs SRecv VTBool) STILL gives proj_u = None    *)
(* and ~projectable_u_wf — umerge unions LABELS, never            *)
(* reconciles divergent payloads.  Both the pre-existing          *)
(* g_excluded and the self-contained g_excluded_u are shipped to  *)
(* keep the boundary maximally visible.                           *)
Example proj_u_g_excluded_still_none : proj_u g_excluded 2 = None.
Proof. reflexivity. Qed.
Definition g_excluded_u : gty :=
  GBra 0 1 (GBcons 0 (GMsg 0 2 VTNat  GEnd)
           (GBcons 1 (GMsg 0 2 VTBool GEnd) GBnil)).
Example proj_u_excluded_still_none : proj_u g_excluded_u 2 = None.
Proof. reflexivity. Qed.
Example g_excluded_u_not_u_projectable : ~ projectable_u_wf g_excluded_u.
Proof. intro H. inversion H; subst.
  match goal with Hm : forall r, r<>0 -> r<>1 -> _ |- _ =>
    destruct (Hm 2 ltac:(congruence) ltac:(congruence)) as [s Hs] end.
  cbn in Hs. discriminate. Qed.

(* ============================================================ *)
(* S3c.1 FENCE (each clause literally true vs the code above):   *)
(*  (1) EXISTENCE not SAFETY: projection_total_u asserts ONLY    *)
(*      that every role of a projectable_u_wf global type        *)
(*      projects (option = Some).  NO subject reduction, NO      *)
(*      progress, NO fidelity, NO deadlock-freedom, NO           *)
(*      coherence — identical fence to projection_total.  The    *)
(*      'wf' in projectable_u_wf is the typing/existence         *)
(*      convention, NOT a safety predicate.  No theorem here     *)
(*      quantifies over a cstep, a reduction, or a configuration.*)
(*  (2) TYPE-ALGEBRA / PROJECTION ONLY: proj_u (gty->role->      *)
(*      option sty) + its totality + a one-directional           *)
(*      monotonicity bridge.  No operational metatheory over an  *)
(*      n-party configuration is reached.  proj is NOT re-       *)
(*      pointed at umerge — plain proj/merge and ALL binary      *)
(*      metatheory (projection_duality, projected_config_wf,     *)
(*      config_subject_reduction) remain BYTE-IDENTICAL.         *)
(*      Strictly ADDITIVE rung (S3c.0 FENCE 1/5 carried).        *)
(*  (3) umerge unions LABELS, not PAYLOADS: union-projection     *)
(*      unlocks ONLY the different-label external-choice class   *)
(*      (witness g_union3).  It does NOT unlock the different-    *)
(*      payload message class — proj_u g_excluded 2 = None and    *)
(*      proj_u g_excluded_u 2 = None still hold, and              *)
(*      ~projectable_u_wf g_excluded_u.  Do not over-read the     *)
(*      headline.  (S3c.0 FENCE 6 carried, re-witnessed.)        *)
(*  (4) SELECT (+, SSelect) asymmetry INHERITED: internal choice *)
(*      is NOT unioned (umerge requires sbranch_eqb EQUALITY on   *)
(*      SSelect).  Only the external & (SBranch) case widens.     *)
(*      proj_u inherits this for free — it calls umerge, not a    *)
(*      new combinator.  (S3c.0 FENCE 4 carried.)                *)
(*  (5) ORDER-DEPENDENCE: umerge (hence proj_u) is argument-      *)
(*      order dependent, proved idempotent ONLY (umerge_idem),    *)
(*      not commutative/associative.  proj_u yields a syntactic   *)
(*      sty whose branch ORDER depends on traversal order; the    *)
(*      witness proj_u_union3_role2_some pins ONE concrete order  *)
(*      (3 before 4).  NO canonicity/uniqueness of proj_u is      *)
(*      claimed.  (S3c.0 FENCE 2 carried.)                       *)
(*  (6) MONOTONICITY is DOMAIN-INCLUSION, not behavioural         *)
(*      refinement: projectable_wf_implies_u says the plain-      *)
(*      projectable set is a SUBSET of the union-projectable set  *)
(*      and (proj_agrees_u) the two projections COINCIDE there.   *)
(*      It does NOT claim union-projected types are safe, nor     *)
(*      that the widening preserves any reduction behaviour.  It  *)
(*      is provable-cheap ONLY because this development's merge   *)
(*      is positional-same-label (merge_forces_eq); if merge is   *)
(*      ever generalised to reorder/widen, the bridge must be     *)
(*      re-proved (the existence theorems projection_total /      *)
(*      projection_total_u survive).  ONE-DIRECTIONAL — the       *)
(*      converse is FALSE (g_union3).                            *)
(*  (7) MU UNPRUNED + p<>q a hypothesis: PWu_Mu admits GMu with   *)
(*      no contractivity/guardedness check; divergent mu-         *)
(*      structure across branches still fails to umerge; p<>q is  *)
(*      a constructor hypothesis, not a structural invariant —    *)
(*      all inherited verbatim from projectable_wf /              *)
(*      projection_total.  (S2.2 fence b / S3a fence 4 carried.) *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE).  proj_u   *)
(*  is a pure gty->role->option sty type-algebra projection and  *)
(*  projectable_u_wf a structural Prop predicate; no EchoMode /  *)
(*  EchoResidue / obligation / attestation / residue artefact    *)
(*  enters this file.  echo-types repo audited read-only: its    *)
(*  session-type content is DYADIC (binary) provenance-residue   *)
(*  bridges (direction E->R), NOT multiparty label-union          *)
(*  projection — nothing reusable for this axis.  Consistent     *)
(*  with the axis-separation note above and the S3c.0 audit.     *)
(* ============================================================ *)

(* ============================================================ *)
(* S3c.2 -- n-ary LOCATED operational semantics (nstep / gstep). *)
(*   The FIRST operational metatheory over an n-party config:   *)
(*   a located reduction nstep on role_assignment (the S3b      *)
(*   container) + a global-type reduction gstep, with the       *)
(*   3-party RING run witnessed on BOTH sides.  OPERATIONAL     *)
(*   ADEQUACY ONLY -- there is deliberately NO subject reduction *)
(*   and NO progress here (see the trailing S3c.2 FENCE).       *)
(*   nstep is the LOCATED mirror of the fused binary cstep: one *)
(*   communication updates EXACTLY the two communicating roles  *)
(*   (ra_set p, ra_set q), every other role untouched.          *)
(* ============================================================ *)

(* Functional pointwise update -- first match, duplicate-tolerant *)
(* exactly like ra_get (on a duplicate key only the FIRST entry  *)
(* is updated).  Keys and length are preserved (ra_set_length).   *)
Fixpoint ra_set (ra : role_assignment) (r : role) (P : party) : role_assignment :=
  match ra with
  | [] => []
  | (r', Q) :: rest =>
      if Nat.eqb r r' then (r', P) :: rest else (r', Q) :: ra_set rest r P
  end.

Lemma ra_set_length : forall ra r P, length (ra_set ra r P) = length ra.
Proof.
  induction ra as [| [r' Q] rest IH]; intros r P; cbn.
  - reflexivity.
  - destruct (Nat.eqb r r'); cbn; [ reflexivity | rewrite IH; reflexivity ].
Qed.

(* set-then-get at the SAME role yields the new party, PROVIDED   *)
(* the role was present (else ra_set is a no-op and ra_get None).  *)
Lemma ra_set_get_eq : forall ra r P Q,
  ra_get ra r = Some Q -> ra_get (ra_set ra r P) r = Some P.
Proof.
  induction ra as [| [r' Q'] rest IH]; intros r P Q H.
  - cbn in H. discriminate.
  - cbn in H. cbn. destruct (Nat.eqb r r') eqn:E.
    + cbn. rewrite E. reflexivity.
    + cbn. rewrite E. apply (IH r P Q H).
Qed.

(* set-then-get at a DIFFERENT role is transparent. *)
Lemma ra_set_get_neq : forall ra r r' P,
  r <> r' -> ra_get (ra_set ra r P) r' = ra_get ra r'.
Proof.
  induction ra as [| [r0 Q0] rest IH]; intros r r' P Hne.
  - cbn. reflexivity.
  - cbn. destruct (Nat.eqb r r0) eqn:E.
    + apply Nat.eqb_eq in E. subst r0. cbn.
      destruct (Nat.eqb r' r) eqn:E2.
      * apply Nat.eqb_eq in E2. subst r'. contradiction Hne. reflexivity.
      * reflexivity.
    + cbn. destruct (Nat.eqb r' r0) eqn:E2.
      * reflexivity.
      * apply IH. exact Hne.
Qed.

(* The n-ary located reduction.  One synchronous communication    *)
(* between TWO DISTINCT roles p (acting) and q (reacting); both    *)
(* their endpoints advance via ra_set, every other role is        *)
(* untouched.  The message + select fragments mirror cstep's       *)
(* CStep / CSel (p universally quantified, so the CStepR/CSelR     *)
(* other-direction rules are subsumed -- no fixed left/right).   *)
Inductive nstep : role_assignment -> role_assignment -> Prop :=
| NStep_Comm : forall ra p q v P Q,
    p <> q ->
    ra_get ra p = Some (QSend v P) ->
    ra_get ra q = Some (QRecv Q) ->
    nstep ra (ra_set (ra_set ra p P) q (open_party v Q))
| NStep_Sel  : forall ra p q l P bsP Q,
    p <> q ->
    ra_get ra p = Some (QSel l P) ->
    ra_get ra q = Some (QBra bsP) ->
    pget l bsP = Some Q ->
    nstep ra (ra_set (ra_set ra p P) q Q).

(* Structural adequacy: a communication neither creates nor        *)
(* destroys a role -- the domain (here: the length) is invariant.  *)
Lemma nstep_length : forall ra ra', nstep ra ra' -> length ra' = length ra.
Proof.
  intros ra ra' H. destruct H.
  - rewrite ra_set_length, ra_set_length. reflexivity.
  - rewrite ra_set_length, ra_set_length. reflexivity.
Qed.

(* Reflexive-transitive closure -- a located RUN. *)
Inductive nstar : role_assignment -> role_assignment -> Prop :=
| NS_refl : forall ra, nstar ra ra
| NS_step : forall ra ra' ra'', nstep ra ra' -> nstar ra' ra'' -> nstar ra ra''.

Lemma nstar_length : forall ra ra', nstar ra ra' -> length ra' = length ra.
Proof.
  intros ra ra' H. induction H.
  - reflexivity.
  - rewrite IHnstar. apply nstep_length. exact H.
Qed.

(* Functional (ra_get-based) well-formedness -- the form n-party   *)
(* subject reduction (S3c.3) will be stated against, because       *)
(* ra_set/ra_get reasoning is functional.  wf_assignment (S3b,     *)
(* In-based) IMPLIES it (so the S3b witness transfers); the        *)
(* converse needs NoDup and is deliberately NOT claimed.           *)
Definition wf_assignment_f (G : gty) (ra : role_assignment) : Prop :=
  forall r P, ra_get ra r = Some P ->
    exists s, proj G r = Some s /\ pty [] P s.

Lemma wf_assignment_to_f : forall G ra,
  wf_assignment G ra -> wf_assignment_f G ra.
Proof.
  intros G ra H r P Hget. apply H. apply ra_get_in. exact Hget.
Qed.

(* S3b's static 3-party ring witness transfers to the functional   *)
(* form for free -- keeps S3b green under the new predicate.       *)
Example wf_ra_ring_f : wf_assignment_f g_ring ra_ring.
Proof. apply wf_assignment_to_f. apply wf_ra_ring. Qed.

(* The global-type reduction (message fragment).  Consuming the    *)
(* head message advances the choreography to its continuation.     *)
(* The choice fragment (GBra branch selection) is S3c.3-choice.    *)
(* gbranch label-lookup (first match): the GLOBAL-side analogue of  *)
(* bget (sbranch) / pget (pbranch).  No such gbranch lookup existed  *)
(* in the file -- only proj_br / proj_uninv consume a gbranch, and   *)
(* neither performs a label lookup.  gbget is FUNCTIONAL (a selected *)
(* label has at most one branch body), so no NoDup side-condition is *)
(* needed for choice subject reduction.  Defined HERE (immediately   *)
(* before gstep) because the new GStep_Bra constructor references it.*)
Fixpoint gbget (l : nat) (bs : gbranch) : option gty :=
  match bs with
  | GBnil           => None
  | GBcons k G rest => if Nat.eqb l k then Some G else gbget l rest
  end.

Inductive gstep : gty -> gty -> Prop :=
| GStep_Msg : forall p q t G, gstep (GMsg p q t G) G
| GStep_Bra : forall p q bs l G,                       (* S3c.3-choice *)
    gbget l bs = Some G -> gstep (GBra p q bs) G.

Inductive gstar : gty -> gty -> Prop :=
| GS_refl : forall G, gstar G G
| GS_step : forall G G' G'', gstep G G' -> gstar G' G'' -> gstar G G''.

(* Global-side adequacy: the ring choreography runs to GEnd. *)
Example g_ring_gsteps : gstar g_ring GEnd.
Proof.
  unfold g_ring.
  eapply GS_step. apply GStep_Msg.
  eapply GS_step. apply GStep_Msg.
  eapply GS_step. apply GStep_Msg.
  apply GS_refl.
Qed.

(* ---- the LOCATED 3-party ring run, step by step ---- *)
(* ra_ring = [(0, !1.?.end); (1, ?.!2.end); (2, ?.!3.end)] runs    *)
(* 0->1, 1->2, 2->0 to all-QEnd.  Each step is one NStep_Comm; the  *)
(* result is written as a clean intermediate (definitionally equal  *)
(* to the constructor's ra_set output -- replace ... by reflexivity *)
(* discharges the conversion, then apply fills the premises).       *)
Definition ra_ring1 : role_assignment :=
  [(0, QRecv QEnd); (1, QSend (VNat 2) QEnd); (2, ring_P2)].
Definition ra_ring2 : role_assignment :=
  [(0, QRecv QEnd); (1, QEnd); (2, QSend (VNat 3) QEnd)].
Definition ra_ring3 : role_assignment :=
  [(0, QEnd); (1, QEnd); (2, QEnd)].

Example ring_nstep1 : nstep ra_ring ra_ring1.
Proof.
  replace ra_ring1 with
    (ra_set (ra_set ra_ring 0 (QRecv QEnd)) 1 (open_party (VNat 1) (QSend (VNat 2) QEnd)))
    by reflexivity.
  apply NStep_Comm; [ discriminate | reflexivity | reflexivity ].
Qed.

Example ring_nstep2 : nstep ra_ring1 ra_ring2.
Proof.
  replace ra_ring2 with
    (ra_set (ra_set ra_ring1 1 QEnd) 2 (open_party (VNat 2) (QSend (VNat 3) QEnd)))
    by reflexivity.
  apply NStep_Comm; [ discriminate | reflexivity | reflexivity ].
Qed.

Example ring_nstep3 : nstep ra_ring2 ra_ring3.
Proof.
  replace ra_ring3 with
    (ra_set (ra_set ra_ring2 2 QEnd) 0 (open_party (VNat 3) QEnd))
    by reflexivity.
  apply NStep_Comm; [ discriminate | reflexivity | reflexivity ].
Qed.

Example ring_runs_to_end : nstar ra_ring ra_ring3.
Proof.
  eapply NS_step. apply ring_nstep1.
  eapply NS_step. apply ring_nstep2.
  eapply NS_step. apply ring_nstep3.
  apply NS_refl.
Qed.

(* The whole run preserves the role count -- 3 roles throughout. *)
Example ring_run_preserves_count : length ra_ring3 = length ra_ring.
Proof. apply nstar_length. apply ring_runs_to_end. Qed.

(* ---- the load-bearing HONESTY witness: NO SR against fixed G ---- *)
(* This is WHY S3c.2 stops at adequacy and n-party subject          *)
(* reduction (S3c.3) must be stated against a STEPPING global type. *)
(* ra_ring is wf_assignment_f at g_ring, but after ONE step ra_ring1 *)
(* is NOT (role 0 is now a RECEIVER QRecv, while proj g_ring 0 is a  *)
(* SEND type -- the endpoint is typed at proj of the STEPPED g, not  *)
(* of g_ring).  So `nstep preserves wf_assignment_f G` is FALSE for  *)
(* a fixed G; the true statement couples nstep with gstep (S3c.3).   *)
Example nstep_breaks_wf_at_fixed_G :
  wf_assignment_f g_ring ra_ring /\ ~ wf_assignment_f g_ring ra_ring1.
Proof.
  split.
  - apply wf_ra_ring_f.
  - intro H. destruct (H 0 (QRecv QEnd) eq_refl) as [s [Hp Ht]].
    cbn in Hp. injection Hp as Hs. subst s. inversion Ht.
Qed.

(* ============================================================ *)
(* S3c.2 FENCE (each clause literally true vs the code above):   *)
(*  (1) OPERATIONAL ADEQUACY ONLY -- nstep / gstep are DEFINED    *)
(*      and the 3-party ring RUN is witnessed on both sides       *)
(*      (ring_runs_to_end, g_ring_gsteps).  There is NO subject   *)
(*      reduction: nstep does NOT preserve wf_assignment_f at a    *)
(*      FIXED G (nstep_breaks_wf_at_fixed_G is a PROVED            *)
(*      refutation).  The true n-party SR couples nstep with       *)
(*      gstep against a STEPPING G -- that is S3c.3.               *)
(*  (2) NO PROGRESS / NO DEADLOCK-FREEDOM -- the ring RUN is BY    *)
(*      EXAMPLE; there is no theorem (every wf config steps or is  *)
(*      all-ended).  n-party progress is research-hard (= S3c.4);  *)
(*      `wf_assignment ra -> deadlock-free` is FALSE in general    *)
(*      (per-endpoint typing carries no global compatibility).     *)
(*  (3) gstep and nstep are NOT YET COUPLED -- no gstep<->nstep    *)
(*      simulation / fidelity is proved (= S3c.3).  They are two   *)
(*      independent reductions that happen to run the same ring.   *)
(*  (4) nstep = the LOCATED mirror of cstep, MESSAGE + SELECT      *)
(*      fragments only (NStep_Comm / NStep_Sel); gstep = MESSAGE   *)
(*      fragment only (GStep_Msg) -- choice-gstep is S3c.3-choice. *)
(*  (5) ra_set is FIRST-MATCH (duplicate-tolerant like ra_get):    *)
(*      on a duplicate key only the first entry updates.  Domain   *)
(*      preserved (ra_set_length, nstep_length, nstar_length).     *)
(*  (6) wf_assignment_f is the FUNCTIONAL (ra_get) well-formedness; *)
(*      wf_assignment (S3b, In) => wf_assignment_f (one direction; *)
(*      the converse needs NoDup, NOT claimed).  Still STATIC      *)
(*      typed-at-projection, NOT n-party safety.                   *)
(*  (7) plain merge + unpruned mu + p<>q-as-hypothesis inherited   *)
(*      from S3a/S3b/S3c.1; proj (NOT proj_u) used (the located    *)
(*      semantics is independent of the S3c.1 widening).           *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE -- nstep is   *)
(*  a located reduction relation, emits no obligation / residue).  *)
(* ============================================================ *)

(* ============================================================ *)
(* S3c.3-msg -- HEAD-COUPLED n-party MESSAGE SUBJECT REDUCTION.  *)
(*   The FIRST EARNED n-party safety half.  S3c.2 proved nstep   *)
(*   does NOT preserve wf_assignment_f at a FIXED G              *)
(*   (nstep_breaks_wf_at_fixed_G is a PROVED refutation).        *)
(*   S3c.3-msg proves the TRUE statement: the HEAD communication  *)
(*   sanctioned by a head GMsg p q t G' carries wf from G to its  *)
(*   continuation G'.  3-way role case-split (sender p /          *)
(*   receiver q / uninvolved r).  The COUPLING: the SAME value v  *)
(*   at the SAME payload type t that the sender's pty_send_inv    *)
(*   yields is what the receiver substitutes via pty_subst0.      *)
(*   The head pair (p,q) is forced to be G's head GMsg pair by    *)
(*   the wf hypothesis being stated at (GMsg p q t G') -- no      *)
(*   separate `p,q is the head pair` hypothesis is needed.        *)
(*   Strictly ADDITIVE; touches nothing prior.                   *)
(* ============================================================ *)

(* ===== CORE: head-coupled message SR, 3-way LOCATED role split ===== *)
Theorem nstep_sr_msg_head :
  forall p q t G' ra v P Q,
    p <> q ->
    wf_assignment_f (GMsg p q t G') ra ->
    ra_get ra p = Some (QSend v P) ->
    ra_get ra q = Some (QRecv Q) ->
    wf_assignment_f G' (ra_set (ra_set ra p P) q (open_party v Q)).
Proof.
  intros p q t G' ra v P Q Hpq Hwf HgetP HgetQ.
  assert (Hqne : (q =? p) = false) by (apply Nat.eqb_neq; auto).
  (* sender p: proj(GMsg p q t G')p = SSend t (proj G' p); get v:t *)
  destruct (Hwf p (QSend v P) HgetP) as [sp [Hprojp Htp]].
  cbn [proj] in Hprojp. rewrite Nat.eqb_refl in Hprojp.
  destruct (proj G' p) as [spc|] eqn:EpG; cbn [option_map] in Hprojp;
    [ | discriminate ].
  injection Hprojp as Esp. subst sp.
  apply pty_send_inv in Htp. destruct Htp as (t0 & s0 & Es & Hv & HPc).
  injection Es as Et Es0. subst t0 s0.
  (* receiver q: proj(GMsg p q t G')q = SRecv t (proj G' q) *)
  destruct (Hwf q (QRecv Q) HgetQ) as [sq [Hprojq Htq]].
  cbn [proj] in Hprojq. rewrite Hqne, Nat.eqb_refl in Hprojq.
  destruct (proj G' q) as [sqc|] eqn:EqG; cbn [option_map] in Hprojq;
    [ | discriminate ].
  injection Hprojq as Esq. subst sq.
  apply pty_recv_inv in Htq. destruct Htq as (t1 & s1 & Es' & HQc).
  injection Es' as Et1 Es1. subst t1 s1.
  (* 3-way role case-split on the stepped ra *)
  intros r R Hget.
  destruct (Nat.eqb r p) eqn:Erp.
  - (* r = p (sender) *)
    apply Nat.eqb_eq in Erp. subst r.
    rewrite ra_set_get_neq in Hget by auto.
    erewrite ra_set_get_eq in Hget by exact HgetP.
    injection Hget as ER. subst R.
    exists spc. split; [ exact EpG | exact HPc ].
  - apply Nat.eqb_neq in Erp.
    destruct (Nat.eqb r q) eqn:Erq.
    + (* r = q (receiver): SAME v:t substituted via pty_subst0 *)
      apply Nat.eqb_eq in Erq. subst r.
      assert (Hgetq' : ra_get (ra_set ra p P) q = Some (QRecv Q)).
      { rewrite ra_set_get_neq by auto. exact HgetQ. }
      erewrite ra_set_get_eq in Hget by exact Hgetq'.
      injection Hget as ER. subst R.
      exists sqc. split; [ exact EqG | eapply pty_subst0; eassumption ].
    + (* r uninvolved: ra_get unchanged; proj(GMsg..)r = proj G' r *)
      apply Nat.eqb_neq in Erq.
      rewrite ra_set_get_neq in Hget by auto.
      rewrite ra_set_get_neq in Hget by auto.
      destruct (Hwf r R Hget) as [s [Hprojr Htr]].
      exists s. split; [ | exact Htr ].
      cbn [proj] in Hprojr.
      assert (Hrp : (r =? p) = false) by (apply Nat.eqb_neq; exact Erp).
      assert (Hrq : (r =? q) = false) by (apply Nat.eqb_neq; exact Erq).
      rewrite Hrp, Hrq in Hprojr. exact Hprojr.
Qed.

(* ===== COUPLED COROLLARY: gstep + nstep + wf together =====
   NOTE: use `split; [|split]`, NOT `repeat split` -- gstep is a
   single-constructor inductive, so `repeat split` would greedily
   APPLY GStep_Msg and desync the bullets.  The gstep premise is the
   head-consume tag (GStep_Msg is the sole message gstep); the nstep
   is the head fire (NStep_Comm output config).  Both are PINNED to
   the SAME head (p,q,t,G') and the SAME ra_set output, so this is
   the HEAD wrapper -- it does NOT state general gstep/nstep SR
   (FALSE for a run-ahead nstep -- see the fence). *)
Corollary nstep_gstep_sr_msg_head :
  forall p q t G' ra ra' v P Q,
    p <> q ->
    wf_assignment_f (GMsg p q t G') ra ->
    ra_get ra p = Some (QSend v P) ->
    ra_get ra q = Some (QRecv Q) ->
    ra' = ra_set (ra_set ra p P) q (open_party v Q) ->
    gstep (GMsg p q t G') G' /\ nstep ra ra' /\ wf_assignment_f G' ra'.
Proof.
  intros p q t G' ra ra' v P Q Hpq Hwf HgetP HgetQ Hra'.
  subst ra'. split; [ | split ].
  - apply GStep_Msg.
  - apply NStep_Comm; assumption.
  - eapply nstep_sr_msg_head; eassumption.
Qed.

(* ===== NON-VACUITY: ra_ring1 IS wf at the STEPPED g_ring ===== *)
(* g_ring = GMsg 0 1 VTNat g_ring_stepped definitionally; the head  *)
(* 0->1 message gsteps g_ring to g_ring_stepped.                    *)
Definition g_ring_stepped : gty :=
  GMsg 1 2 VTNat (GMsg 2 0 VTNat GEnd).

Example g_ring_head_gsteps : gstep g_ring g_ring_stepped.
Proof. unfold g_ring, g_ring_stepped. apply GStep_Msg. Qed.

(* projection sanity at the stepped choreography (cheap corroboration):
   role 0 has FLIPPED from a SEND (at g_ring) to a RECV (at the
   stepped g) -- which is exactly why ra_ring1's QRecv at role 0 is
   now well-typed where S3c.2 refuted it. *)
Example proj_stepped_0 : proj g_ring_stepped 0 = Some (SRecv VTNat SEnd).
Proof. reflexivity. Qed.
Example proj_stepped_1 : proj g_ring_stepped 1 = Some (SSend VTNat SEnd).
Proof. reflexivity. Qed.
Example proj_stepped_2 :
  proj g_ring_stepped 2 = Some (SRecv VTNat (SSend VTNat SEnd)).
Proof. reflexivity. Qed.

(* the witness the S3c.2 fence demanded: proved as a DIRECT
   corollary of the core SR (ra_ring1 = the located 0->1 head
   post-state of ra_ring), no re-derivation by hand. *)
Example ra_ring1_wf_at_stepped_g :
  wf_assignment_f g_ring_stepped ra_ring1.
Proof.
  replace ra_ring1 with
    (ra_set (ra_set ra_ring 0 (QRecv QEnd)) 1
            (open_party (VNat 1) (QSend (VNat 2) QEnd)))
    by reflexivity.
  eapply nstep_sr_msg_head with (t := VTNat).
  - discriminate.
  - (* GMsg 0 1 VTNat g_ring_stepped is DEFINITIONALLY g_ring *)
    exact wf_ra_ring_f.
  - reflexivity.
  - reflexivity.
Qed.

(* THE HEADLINE side-by-side: SAME ra_ring1, refuted at g_ring,
   earned at the stepped g_ring, paid for by the matching gstep.
   Left conjunct = S3c.2 (proj2 nstep_breaks_wf_at_fixed_G);
   middle = S3c.3 (earned); right = the step that pays for it. *)
Example sr_earns_safety_across_step :
  ~ wf_assignment_f g_ring ra_ring1
  /\ wf_assignment_f g_ring_stepped ra_ring1
  /\ gstep g_ring g_ring_stepped.
Proof.
  split; [ | split ].
  - exact (proj2 nstep_breaks_wf_at_fixed_G).
  - exact ra_ring1_wf_at_stepped_g.
  - exact g_ring_head_gsteps.
Qed.

(* ============================================================ *)
(* S3c.3-msg FENCE (each clause literally true vs the code above): *)
(*  (1) HEAD-COMMUNICATION SR ONLY. nstep_sr_msg_head covers the   *)
(*      located comm that fires on the pair (p,q) sanctioned by    *)
(*      G's HEAD GMsg p q t G'.  The wf hypothesis is stated at    *)
(*      (GMsg p q t G'), which structurally PINS the firing pair   *)
(*      to the head pair -- there is no over-general `any pair`     *)
(*      claim, and the conclusion is wf at the head-CONTINUATION   *)
(*      G'.  This is the honest, EARNED n-party SR half.           *)
(*  (2) RUN-AHEAD / NON-HEAD nstep is NOT covered = an HONEST       *)
(*      FENCE, not a gap.  A non-head (run-ahead) nstep -- a        *)
(*      comm on a causally-independent pair deeper in G -- has NO   *)
(*      matching head gstep and would NOT be wf at the head        *)
(*      continuation G'.  Covering it needs a gstep-with-           *)
(*      permutation / swap (commutation) relation = a strictly      *)
(*      bigger (asynchronous / causal-order) theory.  Out of scope. *)
(*  (3) MESSAGE FRAGMENT ONLY. gstep here is GStep_Msg (head        *)
(*      message) only; the proof uses pty_send_inv / pty_recv_inv / *)
(*      pty_subst0.  The SELECT/BRANCH fragment (NStep_Sel coupled  *)
(*      to a choice-gstep that consumes a GBra and picks a label)   *)
(*      is S3c.3-choice -- separate, NOT proved here.               *)
(*  (4) COUPLED COROLLARY nstep_gstep_sr_msg_head is a HEAD         *)
(*      WRAPPER, NOT general SR.  Its gstep + nstep are PINNED to   *)
(*      the SAME head (p,q,t,G') and the SAME ra_set output; it     *)
(*      does NOT assert the unrestricted form [gstep G G1 and       *)
(*      nstep ra ra1 and wf G ra ==> wf G1 ra1] for arbitrary       *)
(*      G,G1,ra,ra1 (FALSE for run-ahead).  It is the forward       *)
(*      coupling for ONE head step -- NOT a bisimulation /          *)
(*      operational correspondence / multi-step.                   *)
(*  (5) FUNCTIONAL wf only (wf_assignment_f, ra_get-based).         *)
(*      wf_assignment (S3b, In) => wf_assignment_f                 *)
(*      (wf_assignment_to_f); the converse needs NoDup, NOT used.   *)
(*      ra_set is FIRST-MATCH / duplicate-tolerant; the theorem is  *)
(*      about the first-match endpoint, consistent with ra_get.     *)
(*  (6) plain merge + unpruned mu + p<>q-as-hypothesis inherited    *)
(*      from S3a/S3b/S3c.1/S3c.2; proj (NOT proj_u, the S3c.1       *)
(*      widening) used.  The payload soundness is genuine: the      *)
(*      SAME v:t the sender ships (Hv : vtype [] v t from p's       *)
(*      send-typing) is what the receiver substitutes -- both       *)
(*      projections read the head GMsg's t.                         *)
(*  (7) NO PROGRESS / NO DEADLOCK-FREEDOM / NO FIDELITY here        *)
(*      (those remain S3c.4 and beyond).  This is type             *)
(*      PRESERVATION (subject reduction) only.                     *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE -- a located   *)
(*  reduction relation emits no obligation / residue / attestation; *)
(*  matches the S3c.2 verdict).                                     *)
(* ============================================================ *)

(* ===== SELF-WITNESS of FENCE clause (2): run-ahead is REAL ===== *)
(* The fence above ASSERTS that a non-head (run-ahead) nstep is not  *)
(* covered by head-coupled SR; here it is DEMONSTRATED by example    *)
(* (the estate boundary-by-example discipline, cf. g_excluded /      *)
(* nstep_breaks_wf_at_fixed_G).  g_runahead's head is 0->1, but the  *)
(* INDEPENDENT pair (2,3) can fire ahead of it; the resulting config *)
(* is NOT wf at the head continuation (GMsg 2 3 VTNat GEnd) -- role 2 *)
(* has run to QEnd while that continuation still expects it to SEND.  *)
(* So head-coupled SR genuinely CANNOT cover a non-head step: the     *)
(* fence is a real boundary, not conservatism.  Lifting it needs a    *)
(* gstep-with-permutation/swap relation = S3c.3-perm (a bigger        *)
(* asynchronous/causal-order theory).                                *)
Definition g_runahead : gty := GMsg 0 1 VTNat (GMsg 2 3 VTNat GEnd).
Definition ra_runahead : role_assignment :=
  [(0, QSend (VNat 1) QEnd); (1, QRecv QEnd);
   (2, QSend (VNat 5) QEnd); (3, QRecv QEnd)].

Example wf_ra_runahead : wf_assignment g_runahead ra_runahead.
Proof.
  intros r P Hin. cbn [In ra_runahead] in Hin.
  destruct Hin as [E | [E | [E | [E | F]]]].
  - injection E as Er EP. subst r P. exists (SSend VTNat SEnd).
    split; [ reflexivity | apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
  - injection E as Er EP. subst r P. exists (SRecv VTNat SEnd).
    split; [ reflexivity | apply PT_Recv; apply PT_End ].
  - injection E as Er EP. subst r P. exists (SSend VTNat SEnd).
    split; [ reflexivity | apply PT_Send; [ apply VT_Nat | apply PT_End ] ].
  - injection E as Er EP. subst r P. exists (SRecv VTNat SEnd).
    split; [ reflexivity | apply PT_Recv; apply PT_End ].
  - contradiction.
Qed.

(* The non-head (2->3) step fires from a wf config, yet its post-     *)
(* state is NOT wf at the head continuation -- head-coupled SR cannot *)
(* reach it.  (Contrast nstep_sr_msg_head, which is about the HEAD    *)
(* 0->1 step.)                                                        *)
Example runahead_breaks_head_coupling :
  wf_assignment_f g_runahead ra_runahead
  /\ nstep ra_runahead
       (ra_set (ra_set ra_runahead 2 QEnd) 3 (open_party (VNat 5) QEnd))
  /\ ~ wf_assignment_f (GMsg 2 3 VTNat GEnd)
        (ra_set (ra_set ra_runahead 2 QEnd) 3 (open_party (VNat 5) QEnd)).
Proof.
  split; [ | split ].
  - apply wf_assignment_to_f. apply wf_ra_runahead.
  - apply NStep_Comm; [ discriminate | reflexivity | reflexivity ].
  - intro H. destruct (H 2 QEnd eq_refl) as [s [Hp Ht]].
    cbn in Hp. injection Hp as Hs. subst s. inversion Ht.
Qed.

(* ============================================================ *)
(* S3c.3-choice — head-coupled SELECT/BRANCH n-party SR.        *)
(*   The CHOICE analogue of S3c.3-msg (nstep_sr_msg_head).      *)
(*   Mirrors that theorem's 3-way LOCATED role case-split       *)
(*   (selector p / offerer q / uninvolved r), with the head     *)
(*   GMsg replaced by a head GBra, the message redex            *)
(*   (QSend/QRecv, NStep_Comm) replaced by the choice redex     *)
(*   (QSel l P / QBra bsP, NStep_Sel), and the unique           *)
(*   continuation G' replaced by the SELECTED branch body       *)
(*   Gl = gbget l bs.  The label l drives BOTH the global step  *)
(*   (GStep_Bra) and the located step (NStep_Sel) onto the      *)
(*   SAME Gl; the firing pair (p,q) AND the choice node are     *)
(*   structurally PINNED to the head by stating wf at           *)
(*   (GBra p q bs).  The ONE structural difference from the     *)
(*   message head: the uninvolved arm is no longer DEFINITIONAL *)
(*   (there is no single continuation) but rides proj_uninv,    *)
(*   the PLAIN-merge fold over all branches; the new lemma      *)
(*   proj_uninv_selected discharges it via merge_forces_eq      *)
(*   (plain merge = identity-meet, so EVERY branch projects an  *)
(*   uninvolved r to the SAME type, hence so does the selected  *)
(*   branch Gl).  Strictly ADDITIVE; the sole prior-region edit *)
(*   is the gbget fixpoint + the GStep_Bra constructor on the   *)
(*   gstep Inductive (sanctioned additive edit).                *)
(* ============================================================ *)

(* HELPER 1 (involved roles, sender p / offerer q): proj_br is the   *)
(* POSITIONAL, label-preserving branch projection (NO merge), so the *)
(* l-entry of the projected sbranch IS proj (selected body) r.       *)
(* Straight induction; serves BOTH the SSelect and SBranch arms.     *)
Lemma proj_br_selected :
  forall bs r sbs l Gl,
    proj_br bs r = Some sbs ->
    gbget l bs = Some Gl ->
    exists sl, bget l sbs = Some sl /\ proj Gl r = Some sl.
Proof.
  induction bs as [ | k G rest IH ]; intros r sbs l Gl Hpb Hg.
  - cbn in Hg. discriminate.
  - cbn [proj_br] in Hpb.
    destruct (proj G r) as [sH|] eqn:EH; [ | discriminate ].
    destruct (proj_br rest r) as [sT|] eqn:ET; [ | discriminate ].
    injection Hpb as Esbs. subst sbs.
    cbn [gbget] in Hg. cbn [bget].
    destruct (Nat.eqb l k) eqn:Elk.
    + injection Hg as EGl. subst Gl.
      exists sH. split; [ reflexivity | exact EH ].
    + destruct (IH r sT l Gl ET Hg) as [sl [Hbg HprojGl]].
      exists sl. split; [ exact Hbg | exact HprojGl ].
Qed.

(* HELPER 2 (THE CRUX — uninvolved-role merge-coupling): proj_uninv  *)
(* folds the branch projections with PLAIN merge.  By merge_forces_eq*)
(* a DEFINED proj_uninv forces every branch to project the           *)
(* uninvolved r to the SAME type s, so the gbget-selected branch Gl  *)
(* projects r to that same s.  This is the choice analogue of the    *)
(* message head's DEFINITIONAL uninvolved case.  COMPILE NOTE: the   *)
(* >=2 case uses an explicit ONE-LEVEL unfold by reflexivity (NOT    *)
(* cbn [proj_uninv], which over-unfolds the recursive inner call and *)
(* desyncs the destructs).                                           *)
Lemma proj_uninv_selected :
  forall bs r s l Gl,
    proj_uninv bs r = Some s ->
    gbget l bs = Some Gl ->
    proj Gl r = Some s.
Proof.
  induction bs as [ | k G rest IH ]; intros r s l Gl Hpu Hg.
  - cbn in Hpu. discriminate.
  - cbn [gbget] in Hg.
    destruct rest as [ | k2 G2 rest2 ].
    + (* SINGLETON: proj_uninv (GBcons k G GBnil) r = proj G r *)
      cbn [proj_uninv] in Hpu.
      destruct (Nat.eqb l k) eqn:Elk.
      * injection Hg as EGl. subst Gl. exact Hpu.
      * cbn in Hg. discriminate.
    + (* >= 2 branches: one-level unfold, KEEP the inner call folded *)
      assert (Hunf :
        proj_uninv (GBcons k G (GBcons k2 G2 rest2)) r =
          match proj G r, proj_uninv (GBcons k2 G2 rest2) r with
          | Some a, Some b => merge a b
          | _, _ => None
          end) by reflexivity.
      rewrite Hunf in Hpu.
      destruct (proj G r) as [sH|] eqn:EH; [ | discriminate ].
      destruct (proj_uninv (GBcons k2 G2 rest2) r) as [sT|] eqn:ET;
        [ | discriminate ].
      (* Hpu : merge sH sT = Some s ; plain merge = identity-meet *)
      pose proof (merge_forces_eq _ _ _ Hpu) as Eeq. subst sT.
      rewrite merge_idem in Hpu. injection Hpu as Es. subst s.
      destruct (Nat.eqb l k) eqn:Elk.
      * injection Hg as EGl. subst Gl. exact EH.
      * (* recurse on the tail fold (proj_uninv rest r = Some sH) *)
        apply (IH r sH l Gl ET Hg).
Qed.

(* ===== CORE: head-coupled SELECT/BRANCH SR, 3-way LOCATED split ===== *)
Theorem nstep_sr_choice_head :
  forall p q bs l Gl ra P bsP Q,
    p <> q ->
    gbget l bs = Some Gl ->
    wf_assignment_f (GBra p q bs) ra ->
    ra_get ra p = Some (QSel l P) ->
    ra_get ra q = Some (QBra bsP) ->
    pget l bsP = Some Q ->
    wf_assignment_f Gl (ra_set (ra_set ra p P) q Q).
Proof.
  intros p q bs l Gl ra P bsP Q Hpq Hgl Hwf HgetP HgetQ HgetQbr.
  assert (Hqne : (q =? p) = false) by (apply Nat.eqb_neq; auto).
  (* selector p: proj(GBra p q bs)p = option_map SSelect (proj_br bs p) *)
  destruct (Hwf p (QSel l P) HgetP) as [sp [Hprojp Htp]].
  cbn [proj] in Hprojp. rewrite Nat.eqb_refl in Hprojp.
  destruct (proj_br bs p) as [bp|] eqn:EpB; cbn [option_map] in Hprojp;
    [ | discriminate ].
  injection Hprojp as Esp. subst sp.
  apply pty_sel_inv in Htp. destruct Htp as (slp & bs0 & Es & Hbget & HPc).
  injection Es as Ebs0. subst bs0.
  (* offerer q: proj(GBra p q bs)q = option_map SBranch (proj_br bs q) *)
  destruct (Hwf q (QBra bsP) HgetQ) as [sq [Hprojq Htq]].
  cbn [proj] in Hprojq. rewrite Hqne, Nat.eqb_refl in Hprojq.
  destruct (proj_br bs q) as [bq|] eqn:EqB; cbn [option_map] in Hprojq;
    [ | discriminate ].
  injection Hprojq as Esq. subst sq.
  apply pty_bra_inv in Htq. destruct Htq as (bs1 & Es' & Hcov).
  injection Es' as Ebs1. subst bs1.
  (* 3-way role case-split on the stepped ra *)
  intros r R Hget.
  destruct (Nat.eqb r p) eqn:Erp.
  - (* r = p (selector): the SELECTED-branch continuation slp *)
    apply Nat.eqb_eq in Erp. subst r.
    rewrite ra_set_get_neq in Hget by auto.
    erewrite ra_set_get_eq in Hget by exact HgetP.
    injection Hget as ER. subst R.
    exists slp. split; [ | exact HPc ].
    destruct (proj_br_selected bs p bp l Gl EpB Hgl) as [slp' [Hbget' HprojGl]].
    rewrite Hbget in Hbget'. injection Hbget' as ->. exact HprojGl.
  - apply Nat.eqb_neq in Erp.
    destruct (Nat.eqb r q) eqn:Erq.
    + (* r = q (offerer): the chosen branch type sBl is what Q follows *)
      apply Nat.eqb_eq in Erq. subst r.
      assert (Hgetq' : ra_get (ra_set ra p P) q = Some (QBra bsP)).
      { rewrite ra_set_get_neq by auto. exact HgetQ. }
      erewrite ra_set_get_eq in Hget by exact Hgetq'.
      injection Hget as ER. subst R.
      destruct (proj_br_selected bs q bq l Gl EqB Hgl) as [sBl [HbgetBl HprojGl]].
      destruct (Hcov l sBl HbgetBl) as [q' [Hpgetq' Htq']].
      rewrite HgetQbr in Hpgetq'. injection Hpgetq' as ->.
      exists sBl. split; [ exact HprojGl | exact Htq' ].
    + (* r uninvolved: ra_get unchanged; proj rides proj_uninv *)
      apply Nat.eqb_neq in Erq.
      rewrite ra_set_get_neq in Hget by auto.
      rewrite ra_set_get_neq in Hget by auto.
      destruct (Hwf r R Hget) as [s [Hprojr Htr]].
      exists s. split; [ | exact Htr ].
      cbn [proj] in Hprojr.
      assert (Hrp : (r =? p) = false) by (apply Nat.eqb_neq; exact Erp).
      assert (Hrq : (r =? q) = false) by (apply Nat.eqb_neq; exact Erq).
      rewrite Hrp, Hrq in Hprojr.
      eapply proj_uninv_selected; [ exact Hprojr | exact Hgl ].
Qed.

(* ===== COUPLED COROLLARY: gstep + nstep + wf together =====
   NOTE: use `split; [|split]`, NOT `repeat split` -- gstep now has
   TWO constructors (GStep_Msg, GStep_Bra), so `repeat split` could
   mis-apply.  The gstep premise is the head-consume tag (GStep_Bra
   selecting label l); the nstep is the head fire (NStep_Sel output
   config).  Both are PINNED to the SAME head (p,q,bs), the SAME
   label l, and the SAME ra_set output, so this is the HEAD wrapper
   -- it does NOT state general gstep/nstep SR (FALSE for run-ahead). *)
Corollary nstep_gstep_sr_choice_head :
  forall p q bs l Gl ra ra' P bsP Q,
    p <> q ->
    gbget l bs = Some Gl ->
    wf_assignment_f (GBra p q bs) ra ->
    ra_get ra p = Some (QSel l P) ->
    ra_get ra q = Some (QBra bsP) ->
    pget l bsP = Some Q ->
    ra' = ra_set (ra_set ra p P) q Q ->
    gstep (GBra p q bs) Gl /\ nstep ra ra' /\ wf_assignment_f Gl ra'.
Proof.
  intros p q bs l Gl ra ra' P bsP Q Hpq Hgl Hwf HgetP HgetQ HgetQbr Hra'.
  subst ra'. split; [ | split ].
  - eapply GStep_Bra. exact Hgl.
  - eapply NStep_Sel; eassumption.
  - eapply nstep_sr_choice_head; eassumption.
Qed.

(* ===== NON-VACUITY WITNESS: the choice rung fires end-to-end ===== *)
(* g_choice3 = GBra 0 1 { 0: GMsg 0 2 VTNat GEnd ; 1: GMsg 0 2 VTNat GEnd } *)
(* (the AGREEING 3-party choice already in the file, ~line 1898).    *)
(* Role 0 (selector) selects label 0 and ships a nat to role 2 in    *)
(* the body; role 1 (offerer) is IDLE in the bodies (each branch is  *)
(* uninvolved for role 1 -> projects to SEnd, so role 1's type is    *)
(* SBranch{0:SEnd;1:SEnd} and its continuations are QEnd, NOT QRecv); *)
(* role 2 receives the nat.                                          *)
Definition ra_choice3 : role_assignment :=
  [ (0, QSel 0 (QSend (VNat 7) QEnd))
  ; (1, QBra (PBcons 0 QEnd (PBcons 1 QEnd PBnil)))
  ; (2, QRecv QEnd) ].

Example proj_choice3_0 :
  proj g_choice3 0
  = Some (SSelect (SBcons 0 (SSend VTNat SEnd) (SBcons 1 (SSend VTNat SEnd) SBnil))).
Proof. reflexivity. Qed.
Example proj_choice3_1 :
  proj g_choice3 1 = Some (SBranch (SBcons 0 SEnd (SBcons 1 SEnd SBnil))).
Proof. reflexivity. Qed.
Example proj_choice3_2 : proj g_choice3 2 = Some (SRecv VTNat SEnd).
Proof. reflexivity. Qed.

Example wf_ra_choice3 : wf_assignment_f g_choice3 ra_choice3.
Proof.
  intros r R Hget. cbn [ra_get ra_choice3] in Hget.
  destruct (Nat.eqb r 0) eqn:E0.
  - apply Nat.eqb_eq in E0. subst r. injection Hget as ER. subst R.
    eexists. split; [ reflexivity | ].
    eapply PT_Sel; [ reflexivity | ].
    apply PT_Send; [ apply VT_Nat | apply PT_End ].
  - destruct (Nat.eqb r 1) eqn:E1.
    + apply Nat.eqb_eq in E1. subst r. injection Hget as ER. subst R.
      eexists. split; [ reflexivity | ].
      apply PT_Bra. intros l sB Hbg. cbn [bget] in Hbg.
      destruct (Nat.eqb l 0) eqn:Hl0.
      * injection Hbg as <-. exists QEnd.
        split; [ apply Nat.eqb_eq in Hl0; subst l; reflexivity | apply PT_End ].
      * destruct (Nat.eqb l 1) eqn:Hl1.
        -- injection Hbg as <-. exists QEnd.
           split; [ apply Nat.eqb_eq in Hl1; subst l; reflexivity | apply PT_End ].
        -- discriminate.
    + destruct (Nat.eqb r 2) eqn:E2.
      * apply Nat.eqb_eq in E2. subst r. injection Hget as ER. subst R.
        eexists. split; [ reflexivity | ].
        apply PT_Recv. apply PT_End.
      * discriminate.
Qed.

(* The selected branch body (label 0). *)
Definition g_choice3_sel0 : gty := GMsg 0 2 VTNat GEnd.

Example gbget_choice3_0 :
  gbget 0 (GBcons 0 (GMsg 0 2 VTNat GEnd) (GBcons 1 (GMsg 0 2 VTNat GEnd) GBnil))
  = Some g_choice3_sel0.
Proof. reflexivity. Qed.

(* END TO END: ONE choice step (gstep + nstep + wf) earned by the core. *)
Example choice3_head_fires_end_to_end :
  gstep g_choice3 g_choice3_sel0
  /\ nstep ra_choice3
       (ra_set (ra_set ra_choice3 0 (QSend (VNat 7) QEnd)) 1 QEnd)
  /\ wf_assignment_f g_choice3_sel0
       (ra_set (ra_set ra_choice3 0 (QSend (VNat 7) QEnd)) 1 QEnd).
Proof.
  eapply nstep_gstep_sr_choice_head
    with (p:=0) (q:=1) (l:=0)
         (bs:=(GBcons 0 (GMsg 0 2 VTNat GEnd) (GBcons 1 (GMsg 0 2 VTNat GEnd) GBnil)))
         (P:=QSend (VNat 7) QEnd)
         (bsP:=PBcons 0 QEnd (PBcons 1 QEnd PBnil))
         (Q:=QEnd).
  - discriminate.
  - reflexivity.
  - exact wf_ra_choice3.
  - reflexivity.
  - reflexivity.
  - reflexivity.
  - reflexivity.
Qed.

(* ============================================================ *)
(* S3c.3-choice FENCE (each clause literally true vs the code above):*)
(*  (1) HEAD CHOICE SR ONLY.  wf is stated at (GBra p q bs), which   *)
(*      structurally PINS the firing pair (p,q) AND the choice node  *)
(*      to G's head; the conclusion is wf at the SELECTED branch     *)
(*      continuation Gl = gbget l bs.  The label l drives BOTH the   *)
(*      global step (GStep_Bra) and the located step (NStep_Sel)     *)
(*      onto the same Gl.  No over-general 'any pair / any node'.    *)
(*  (2) RUN-AHEAD / NON-HEAD selection is NOT covered = S3c.3-perm,  *)
(*      out of scope (a non-head/deeper selection has no matching    *)
(*      head GStep_Bra).  Honest boundary, mirroring the message     *)
(*      run-ahead fence (g_runahead, self-witnessed above).          *)
(*  (3) PLAIN proj (NOT proj_u): the uninvolved arm rides proj_uninv *)
(*      (the plain-merge fold); proj_uninv_selected leans on         *)
(*      merge_forces_eq = plain merge is the IDENTITY-MEET, so every *)
(*      branch projects an uninvolved r to the SAME type.  The full  *)
(*      label-UNION merge (union-typed uninvolved continuation) is   *)
(*      explicitly NOT used (= S3, separate).  REGRESSION PIN: if a  *)
(*      future rung widens merge (reorder/union), merge_forces_eq    *)
(*      and proj_uninv_selected BOTH break and this theorem becomes  *)
(*      FALSE-as-stated -- that is exactly the S3 fence, not a bug.  *)
(*  (4) COUPLED COROLLARY nstep_gstep_sr_choice_head is a HEAD       *)
(*      WRAPPER for ONE step (gstep + nstep + wf all pinned to the   *)
(*      same head (p,q,bs), label l, ra_set output), NOT general SR  *)
(*      / bisimulation / operational correspondence / multi-step.    *)
(*  (5) p<>q carried as a HYPOTHESIS (two_party does not entail it;  *)
(*      inherited fence from the message head).  FUNCTIONAL wf only  *)
(*      (wf_assignment_f, ra_get-based), matching S3c.3-msg.         *)
(*  (6) NEW definitions: gbget (gbranch label-lookup, FUNCTIONAL     *)
(*      first-match, the global analogue of bget/pget -- no such     *)
(*      lookup pre-existed in the file) and GStep_Bra (the SOLE      *)
(*      additive edit to an existing Inductive, gstep).              *)
(*  (7) PRESERVATION (subject reduction) ONLY -- no progress /       *)
(*      deadlock-freedom / fidelity.  Naming: nstep_sr_choice_head   *)
(*      (NOT n_party_safety).                                        *)
(*  Echo-types audit: NOT-RELEVANT (axis-2 STRUCTURE -- a located    *)
(*  reduction emits no obligation / residue / attestation; matches   *)
(*  the S3c.2 / S3c.3-msg verdict).                                  *)
(* ============================================================ *)
