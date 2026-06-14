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
(*       relation (S1.2), internal/external choice select/branch *)
(*       (S1.2), replication !P, mismatch, mu-recursion,         *)
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

Inductive sty : Type :=
| SEnd  : sty                 (* end *)
| SSend : vty -> sty -> sty   (* !t.s  : send a t, continue as s *)
| SRecv : vty -> sty -> sty.  (* ?t.s  : receive a t, continue as s *)

Fixpoint dual (s : sty) : sty :=
  match s with
  | SEnd      => SEnd
  | SSend t k => SRecv t (dual k)
  | SRecv t k => SSend t (dual k)
  end.

Lemma dual_involutive : forall s, dual (dual s) = s.
Proof. induction s; simpl; congruence. Qed.

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
(* congruence (Par-Comm) is deferred to S1.2.                    *)

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
Inductive party : Type :=
| QEnd  : party                       (* close *)
| QSend : val -> party -> party       (* send v, continue *)
| QRecv : party -> party.             (* receive (bind payload de Bruijn 0), continue *)

(* Party typing: [pty G p s] — under payload context G, party p    *)
(* follows session type s exactly.                                 *)
Inductive pty : list vty -> party -> sty -> Prop :=
| PT_End  : forall G, pty G QEnd SEnd
| PT_Send : forall G v p t s, vtype G v t -> pty G p s -> pty G (QSend v p) (SSend t s)
| PT_Recv : forall G p t s, pty (t :: G) p s -> pty G (QRecv p) (SRecv t s).

Fixpoint psubst_party (c : nat) (u : val) (p : party) : party :=
  match p with
  | QEnd      => QEnd
  | QSend v q => QSend (vsubst c u v) (psubst_party c u q)
  | QRecv q   => QRecv (psubst_party (S c) (vlift 0 u) q)
  end.

Definition open_party (u : val) (p : party) : party := psubst_party 0 u p.

(* Party inversion helpers (clean names). *)
Lemma pty_send_inv : forall G v p s, pty G (QSend v p) s ->
  exists t s', s = SSend t s' /\ vtype G v t /\ pty G p s'.
Proof. intros G v p s H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

Lemma pty_recv_inv : forall G p s, pty G (QRecv p) s ->
  exists t s', s = SRecv t s' /\ pty (t :: G) p s'.
Proof. intros G p s H. inversion H; subst. do 2 eexists; repeat split; eassumption. Qed.

(* Party value-substitution lemma (the party analogue of wt_subst, *)
(* reusing the same value-shift machinery).                        *)
Lemma pty_subst : forall p G1 G2 s t v,
  pty (G1 ++ t :: G2) p s ->
  vtype G2 v t ->
  pty (G1 ++ G2) (psubst_party (length G1) (vshift (length G1) v) p) s.
Proof.
  induction p as [ | w q IHq | q IHq ]; intros G1 G2 s t v Hp Hv; simpl.
  - (* QEnd *) inversion Hp; subst. constructor.
  - (* QSend *) inversion Hp; subst. constructor.
    + eapply vtype_subst; eassumption.
    + apply IHq with (t:=t); assumption.
  - (* QRecv *) inversion Hp; subst. constructor.
    rewrite vlift0_vshift.
    apply (IHq (t0 :: G1) G2 s0 t v); assumption.
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
| CStepR : forall v P Q, cstep (Conf (QRecv Q) (QSend v P)) (Conf (open_party v Q) P).

(* ----- the S1.1b headline: closed-system subject reduction ----- *)
(* Well-formedness (both parties dual) is preserved by reduction —  *)
(* the two endpoints stay dual as their shared protocol advances.   *)
Theorem config_subject_reduction : forall c c',
  wf_config c -> cstep c c' -> wf_config c'.
Proof.
  intros c c' Hwf Hstep. destruct Hstep as [v P Q | v P Q].
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
