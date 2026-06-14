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
