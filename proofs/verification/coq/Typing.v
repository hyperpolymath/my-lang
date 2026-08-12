(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================== *)
(* My Language: Coq Formalization            *)
(* Typing Rules                               *)
(*                                            *)
(* Design fixes applied 2026-03-22:           *)
(* - Moved subtype before has_type            *)
(* - Added T_Sub rule for subsumption         *)
(* - Corrected weakening statement (freshness)*)
(* - Proved substitution for closed values    *)
(* ========================================== *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
Require Import Coq.Bool.Bool.
Require Import Syntax.
Import ListNotations.

(** * Typing Environment *)

Definition type_env : Type := list (string * ty).

Fixpoint lookup (env : type_env) (x : string) : option ty :=
  match env with
  | [] => None
  | (y, t) :: rest => if String.eqb x y then Some t else lookup rest x
  end.

Definition extend (env : type_env) (x : string) (t : ty) : type_env :=
  (x, t) :: env.

(** * Value Predicate *)

Inductive is_value : expr -> Prop :=
  | V_Lit : forall l, is_value (ELit l)
  | V_Lam : forall x t e, is_value (ELam x t e)
  | V_Array : forall es, Forall is_value es -> is_value (EArray es)
  | V_Record : forall fs, Forall (fun p => is_value (snd p)) fs ->
                          is_value (ERecord fs).

(** * Subtyping

    Defined before has_type so that the typing judgment can include a
    subsumption rule (T_Sub). This is the standard design for declarative
    type systems with subtyping — see Pierce, TAPL Ch. 15. *)

Inductive subtype : ty -> ty -> Prop :=
  | Sub_Refl : forall t, subtype t t
  | Sub_Trans : forall t1 t2 t3,
      subtype t1 t2 -> subtype t2 t3 -> subtype t1 t3
  | Sub_IntFloat : subtype (TPrim TInt) (TPrim TFloat)
  | Sub_Fun : forall t1 t2 s1 s2,
      subtype s1 t1 ->  (* Contravariant *)
      subtype t2 s2 ->  (* Covariant *)
      subtype (TFun t1 t2) (TFun s1 s2)
  | Sub_AI : forall t1 t2,
      subtype t1 t2 -> subtype (TAI t1) (TAI t2).

(** * Typing Judgment *)

(* Γ ⊢ e : τ *)
Inductive has_type : type_env -> expr -> ty -> Prop :=

  (* Literals *)
  | T_Int : forall env n,
      has_type env (ELit (LInt n)) (TPrim TInt)

  | T_Float : forall env f,
      has_type env (ELit (LFloat f)) (TPrim TFloat)

  | T_String : forall env s,
      has_type env (ELit (LString s)) (TPrim TString)

  | T_Bool : forall env b,
      has_type env (ELit (LBool b)) (TPrim TBool)

  (* Variables *)
  | T_Var : forall env x t,
      lookup env x = Some t ->
      has_type env (EVar x) t

  (* Lambda *)
  | T_Lam : forall env x t1 t2 e,
      has_type (extend env x t1) e t2 ->
      has_type env (ELam x t1 e) (TFun t1 t2)

  (* Application *)
  | T_App : forall env e1 e2 t1 t2,
      has_type env e1 (TFun t1 t2) ->
      has_type env e2 t1 ->
      has_type env (EApp e1 e2) t2

  (* Let binding *)
  | T_Let : forall env x t1 t2 e1 e2,
      has_type env e1 t1 ->
      has_type (extend env x t1) e2 t2 ->
      has_type env (ELet x (Some t1) e1 e2) t2

  (* Conditional *)
  | T_If : forall env e1 e2 e3 t,
      has_type env e1 (TPrim TBool) ->
      has_type env e2 t ->
      has_type env e3 t ->
      has_type env (EIf e1 e2 e3) t

  (* Binary operations - Arithmetic *)
  | T_BinOp_Int : forall env e1 e2 op,
      op = OpAdd \/ op = OpSub \/ op = OpMul \/ op = OpDiv ->
      has_type env e1 (TPrim TInt) ->
      has_type env e2 (TPrim TInt) ->
      has_type env (EBinOp e1 op e2) (TPrim TInt)

  (* Binary operations - Comparison *)
  | T_BinOp_Cmp : forall env e1 e2 op t,
      op = OpLt \/ op = OpGt \/ op = OpLe \/ op = OpGe ->
      (t = TPrim TInt \/ t = TPrim TFloat) ->
      has_type env e1 t ->
      has_type env e2 t ->
      has_type env (EBinOp e1 op e2) (TPrim TBool)

  (* Binary operations - Equality *)
  | T_BinOp_Eq : forall env e1 e2 op t,
      op = OpEq \/ op = OpNe ->
      has_type env e1 t ->
      has_type env e2 t ->
      has_type env (EBinOp e1 op e2) (TPrim TBool)

  (* Binary operations - Logical *)
  | T_BinOp_Logic : forall env e1 e2 op,
      op = OpAnd \/ op = OpOr ->
      has_type env e1 (TPrim TBool) ->
      has_type env e2 (TPrim TBool) ->
      has_type env (EBinOp e1 op e2) (TPrim TBool)

  (* Unary operations - Negation *)
  | T_UnOp_Neg : forall env e t,
      (t = TPrim TInt \/ t = TPrim TFloat) ->
      has_type env e t ->
      has_type env (EUnOp OpNeg e) t

  (* Unary operations - Not *)
  | T_UnOp_Not : forall env e,
      has_type env e (TPrim TBool) ->
      has_type env (EUnOp OpNot e) (TPrim TBool)

  (* Arrays *)
  | T_Array : forall env es t,
      Forall (fun e => has_type env e t) es ->
      has_type env (EArray es) (TArray t)

  (* AI expressions *)
  | T_AI_Query : forall env fields,
      has_type env (EAI AIQuery fields) (TAI (TPrim TString))

  | T_AI_Verify : forall env fields,
      has_type env (EAI AIVerify fields) (TAI (TPrim TBool))

  | T_AI_Embed : forall env fields,
      has_type env (EAI AIEmbed fields) (TAI (TArray (TPrim TFloat)))

  (* Subsumption: standard rule for declarative subtyping.
     If e has type t1 and t1 is a subtype of t2, then e has type t2.
     This enables implicit coercion along the subtype hierarchy
     (e.g., Int expressions can be used where Float is expected). *)
  | T_Sub : forall env e t1 t2,
      has_type env e t1 ->
      subtype t1 t2 ->
      has_type env e t2.

(** * Type Equivalence *)

Inductive type_equiv : ty -> ty -> Prop :=
  | TE_Refl : forall t, type_equiv t t
  | TE_Sym : forall t1 t2, type_equiv t1 t2 -> type_equiv t2 t1
  | TE_Trans : forall t1 t2 t3,
      type_equiv t1 t2 -> type_equiv t2 t3 -> type_equiv t1 t3
  | TE_Fun : forall t1 t2 s1 s2,
      type_equiv t1 s1 -> type_equiv t2 s2 ->
      type_equiv (TFun t1 t2) (TFun s1 s2)
  | TE_Array : forall t1 t2,
      type_equiv t1 t2 -> type_equiv (TArray t1) (TArray t2)
  | TE_AI : forall t1 t2,
      type_equiv t1 t2 -> type_equiv (TAI t1) (TAI t2).

(* ================================================================== *)
(** * Custom Induction Principle for has_type with Nested Forall IH   *)
(* ================================================================== *)

(** Coq's auto-generated has_type_ind does NOT supply a nested induction
    hypothesis for the [Forall (fun e => has_type env e t) es] premise
    of [T_Array]. Without it, any lemma proved by [induction Htype]
    cannot discharge the [T_Array] case because the individual
    sub-derivations in [es] have no inductive hypothesis.

    This scheme threads a nested Forall IH through [T_Array] using an
    inner fixpoint over the Forall proof, which Coq's termination
    checker accepts as structurally decreasing (the element-wise
    sub-proof [Hx] is a sub-term of the outer [Htype] via [Hes]). *)

Section has_type_strong_ind.
  Variable P : type_env -> expr -> ty -> Prop.

  Hypothesis PT_Int : forall env n, P env (ELit (LInt n)) (TPrim TInt).
  Hypothesis PT_Float : forall env f, P env (ELit (LFloat f)) (TPrim TFloat).
  Hypothesis PT_String : forall env s, P env (ELit (LString s)) (TPrim TString).
  Hypothesis PT_Bool : forall env b, P env (ELit (LBool b)) (TPrim TBool).
  Hypothesis PT_Var : forall env x t,
    lookup env x = Some t -> P env (EVar x) t.
  Hypothesis PT_Lam : forall env x t1 t2 e,
    has_type (extend env x t1) e t2 ->
    P (extend env x t1) e t2 ->
    P env (ELam x t1 e) (TFun t1 t2).
  Hypothesis PT_App : forall env e1 e2 t1 t2,
    has_type env e1 (TFun t1 t2) -> P env e1 (TFun t1 t2) ->
    has_type env e2 t1 -> P env e2 t1 ->
    P env (EApp e1 e2) t2.
  Hypothesis PT_Let : forall env x t1 t2 e1 e2,
    has_type env e1 t1 -> P env e1 t1 ->
    has_type (extend env x t1) e2 t2 -> P (extend env x t1) e2 t2 ->
    P env (ELet x (Some t1) e1 e2) t2.
  Hypothesis PT_If : forall env e1 e2 e3 t,
    has_type env e1 (TPrim TBool) -> P env e1 (TPrim TBool) ->
    has_type env e2 t -> P env e2 t ->
    has_type env e3 t -> P env e3 t ->
    P env (EIf e1 e2 e3) t.
  Hypothesis PT_BinOp_Int : forall env e1 e2 op,
    op = OpAdd \/ op = OpSub \/ op = OpMul \/ op = OpDiv ->
    has_type env e1 (TPrim TInt) -> P env e1 (TPrim TInt) ->
    has_type env e2 (TPrim TInt) -> P env e2 (TPrim TInt) ->
    P env (EBinOp e1 op e2) (TPrim TInt).
  Hypothesis PT_BinOp_Cmp : forall env e1 e2 op t,
    op = OpLt \/ op = OpGt \/ op = OpLe \/ op = OpGe ->
    (t = TPrim TInt \/ t = TPrim TFloat) ->
    has_type env e1 t -> P env e1 t ->
    has_type env e2 t -> P env e2 t ->
    P env (EBinOp e1 op e2) (TPrim TBool).
  Hypothesis PT_BinOp_Eq : forall env e1 e2 op t,
    op = OpEq \/ op = OpNe ->
    has_type env e1 t -> P env e1 t ->
    has_type env e2 t -> P env e2 t ->
    P env (EBinOp e1 op e2) (TPrim TBool).
  Hypothesis PT_BinOp_Logic : forall env e1 e2 op,
    op = OpAnd \/ op = OpOr ->
    has_type env e1 (TPrim TBool) -> P env e1 (TPrim TBool) ->
    has_type env e2 (TPrim TBool) -> P env e2 (TPrim TBool) ->
    P env (EBinOp e1 op e2) (TPrim TBool).
  Hypothesis PT_UnOp_Neg : forall env e t,
    (t = TPrim TInt \/ t = TPrim TFloat) ->
    has_type env e t -> P env e t ->
    P env (EUnOp OpNeg e) t.
  Hypothesis PT_UnOp_Not : forall env e,
    has_type env e (TPrim TBool) -> P env e (TPrim TBool) ->
    P env (EUnOp OpNot e) (TPrim TBool).
  Hypothesis PT_Array : forall env es t,
    Forall (fun e => has_type env e t) es ->
    Forall (fun e => P env e t) es ->
    P env (EArray es) (TArray t).
  Hypothesis PT_AI_Query : forall env fields,
    P env (EAI AIQuery fields) (TAI (TPrim TString)).
  Hypothesis PT_AI_Verify : forall env fields,
    P env (EAI AIVerify fields) (TAI (TPrim TBool)).
  Hypothesis PT_AI_Embed : forall env fields,
    P env (EAI AIEmbed fields) (TAI (TArray (TPrim TFloat))).
  Hypothesis PT_Sub : forall env e t1 t2,
    has_type env e t1 -> P env e t1 ->
    subtype t1 t2 ->
    P env e t2.

  Fixpoint has_type_ind_strong (env : type_env) (e : expr) (t : ty)
    (Htype : has_type env e t) {struct Htype} : P env e t :=
    match Htype in has_type env' e' t' return P env' e' t' with
    | T_Int env n => PT_Int env n
    | T_Float env f => PT_Float env f
    | T_String env s => PT_String env s
    | T_Bool env b => PT_Bool env b
    | T_Var env x t Hlook => PT_Var env x t Hlook
    | T_Lam env x t1 t2 e H =>
        PT_Lam env x t1 t2 e H
          (has_type_ind_strong (extend env x t1) e t2 H)
    | T_App env e1 e2 t1 t2 H1 H2 =>
        PT_App env e1 e2 t1 t2
          H1 (has_type_ind_strong env e1 (TFun t1 t2) H1)
          H2 (has_type_ind_strong env e2 t1 H2)
    | T_Let env x t1 t2 e1 e2 H1 H2 =>
        PT_Let env x t1 t2 e1 e2
          H1 (has_type_ind_strong env e1 t1 H1)
          H2 (has_type_ind_strong (extend env x t1) e2 t2 H2)
    | T_If env e1 e2 e3 t H1 H2 H3 =>
        PT_If env e1 e2 e3 t
          H1 (has_type_ind_strong env e1 (TPrim TBool) H1)
          H2 (has_type_ind_strong env e2 t H2)
          H3 (has_type_ind_strong env e3 t H3)
    | T_BinOp_Int env e1 e2 op Hop H1 H2 =>
        PT_BinOp_Int env e1 e2 op Hop
          H1 (has_type_ind_strong env e1 (TPrim TInt) H1)
          H2 (has_type_ind_strong env e2 (TPrim TInt) H2)
    | T_BinOp_Cmp env e1 e2 op t Hop Ht H1 H2 =>
        PT_BinOp_Cmp env e1 e2 op t Hop Ht
          H1 (has_type_ind_strong env e1 t H1)
          H2 (has_type_ind_strong env e2 t H2)
    | T_BinOp_Eq env e1 e2 op t Hop H1 H2 =>
        PT_BinOp_Eq env e1 e2 op t Hop
          H1 (has_type_ind_strong env e1 t H1)
          H2 (has_type_ind_strong env e2 t H2)
    | T_BinOp_Logic env e1 e2 op Hop H1 H2 =>
        PT_BinOp_Logic env e1 e2 op Hop
          H1 (has_type_ind_strong env e1 (TPrim TBool) H1)
          H2 (has_type_ind_strong env e2 (TPrim TBool) H2)
    | T_UnOp_Neg env e t Ht H =>
        PT_UnOp_Neg env e t Ht
          H (has_type_ind_strong env e t H)
    | T_UnOp_Not env e H =>
        PT_UnOp_Not env e
          H (has_type_ind_strong env e (TPrim TBool) H)
    | T_Array env es t Hes =>
        PT_Array env es t Hes
          ((fix forall_ih (l : list expr)
              (Hl : Forall (fun e => has_type env e t) l)
              {struct Hl} : Forall (fun e => P env e t) l :=
              match Hl in Forall _ l' return Forall (fun e => P env e t) l' with
              | Forall_nil _ => Forall_nil _
              | Forall_cons x Hx Hxs =>
                  Forall_cons x
                    (has_type_ind_strong env x t Hx)
                    (forall_ih _ Hxs)
              end) es Hes)
    | T_AI_Query env fields => PT_AI_Query env fields
    | T_AI_Verify env fields => PT_AI_Verify env fields
    | T_AI_Embed env fields => PT_AI_Embed env fields
    | T_Sub env e t1 t2 H Hsub =>
        PT_Sub env e t1 t2 H (has_type_ind_strong env e t1 H) Hsub
    end.

End has_type_strong_ind.

(* ================================================================== *)
(** * Infrastructure Lemmas                                            *)
(* ================================================================== *)

(** ** Superset environment preserves typing.

    If env2 has all the bindings of env1 (possibly more), then any term
    typeable in env1 is also typeable in env2. This generalizes both
    environment equivalence, weakening, and closed-term lifting.

    The condition is: if env1 maps x to some type, then env2 maps x to
    the same type. env2 may have additional bindings that env1 lacks. *)

Lemma has_type_env_weaken : forall env1 env2 e t,
  (forall x ty, lookup env1 x = Some ty -> lookup env2 x = Some ty) ->
  has_type env1 e t ->
  has_type env2 e t.
Proof.
  intros env1 env2 e t Hincl Htype.
  revert env2 Hincl.
  induction Htype using has_type_ind_strong; intros env2 Hincl.
  - constructor.
  - constructor.
  - constructor.
  - constructor.
  - apply T_Var. apply Hincl. assumption.
  - apply T_Lam. apply IHHtype.
    intros z tz Hlook. unfold extend in *. simpl in *.
    destruct (String.eqb z x) eqn:Hzx.
    + assumption.
    + apply Hincl. assumption.
  - eapply T_App; [apply IHHtype1 | apply IHHtype2]; assumption.
  - eapply T_Let.
    + apply IHHtype1. assumption.
    + apply IHHtype2.
      intros z tz Hlook. unfold extend in *. simpl in *.
      destruct (String.eqb z x) eqn:Hzx; [assumption | apply Hincl; assumption].
  - apply T_If; [apply IHHtype1 | apply IHHtype2 | apply IHHtype3]; assumption.
  - apply T_BinOp_Int;
    [assumption | apply IHHtype1; assumption | apply IHHtype2; assumption].
  - apply T_BinOp_Cmp with (t := t); try assumption;
    [apply IHHtype1; assumption | apply IHHtype2; assumption].
  - apply T_BinOp_Eq with (t := t); try assumption;
    [apply IHHtype1; assumption | apply IHHtype2; assumption].
  - apply T_BinOp_Logic;
    [assumption | apply IHHtype1; assumption | apply IHHtype2; assumption].
  - apply T_UnOp_Neg; [assumption | apply IHHtype; assumption].
  - apply T_UnOp_Not. apply IHHtype. assumption.
  - apply T_Array.
    eapply Forall_impl; [| exact H0].
    simpl. intros e He. apply He. exact Hincl.
  - constructor.
  - constructor.
  - constructor.
  - eapply T_Sub; [apply IHHtype; assumption | assumption].
Qed.

(** Corollary: environments that agree on all lookups produce same typing *)
Lemma has_type_env_equiv : forall env1 env2 e t,
  (forall x, lookup env1 x = lookup env2 x) ->
  has_type env1 e t ->
  has_type env2 e t.
Proof.
  intros env1 env2 e t Hlookup Htype.
  apply has_type_env_weaken with (env1 := env1).
  - intros x ty Hx. rewrite <- Hlookup. assumption.
  - assumption.
Qed.

(** Corollary: a closed term (typed in []) can be typed in any env *)
Lemma closed_typing_any_env : forall env e t,
  has_type [] e t ->
  has_type env e t.
Proof.
  intros env e t Htype.
  apply has_type_env_weaken with (env1 := []).
  - intros x ty Hlook. simpl in Hlook. discriminate.
  - assumption.
Qed.

(** Environment permutation for distinct variables *)
Lemma env_permute : forall env x t1 y t2 e t,
  x <> y ->
  has_type ((x, t1) :: (y, t2) :: env) e t ->
  has_type ((y, t2) :: (x, t1) :: env) e t.
Proof.
  intros env x t1 y t2 e t Hneq Htype.
  apply has_type_env_equiv with (env1 := (x, t1) :: (y, t2) :: env).
  - intros z. simpl.
    destruct (String.eqb z x) eqn:Hzx; destruct (String.eqb z y) eqn:Hzy;
    try reflexivity.
    apply String.eqb_eq in Hzx. apply String.eqb_eq in Hzy.
    subst. contradiction.
  - assumption.
Qed.

(** Membership in remove for string lists *)
Lemma in_in_remove : forall (x y : string) (l : list string),
  x <> y -> In y l -> In y (remove string_dec x l).
Proof.
  intros x y l Hneq. induction l; intros Hin.
  - inversion Hin.
  - simpl. destruct (string_dec x a).
    + subst. destruct Hin; [subst; contradiction | apply IHl; assumption].
    + destruct Hin; [left; assumption | right; apply IHl; assumption].
Qed.

(* ================================================================== *)
(** * Subsumption                                                       *)
(* ================================================================== *)

(** With T_Sub in has_type, subsumption is a direct constructor application.
    This lemma was historically left as a proof gap because has_type lacked
    a subsumption rule and subtype allowed Int <: Float with no corresponding
    typing coercion; it is now fully proved (see Qed below). *)

Lemma subsumption : forall env e t1 t2,
  has_type env e t1 ->
  subtype t1 t2 ->
  has_type env e t2.
Proof.
  intros env e t1 t2 Htype Hsub.
  exact (T_Sub env e t1 t2 Htype Hsub).
Qed.

(* ================================================================== *)
(** * Weakening                                                         *)
(* ================================================================== *)

(** Weakening: adding a fresh variable binding to the front of the
    environment preserves typing.

    Design note: The original statement omitted the freshness condition
    (~ In x (free_vars e)), which made it unprovable — extending with a
    variable that shadows an existing binding changes the type of that
    variable. The corrected statement requires x not free in e.

    Caveat: free_vars is incomplete for EArray (returns []) so the
    freshness condition is vacuously true for arrays. The proof handles
    this correctly via induction on the Forall hypothesis. *)

Lemma weakening : forall env e t x s,
  has_type env e t ->
  ~ In x (free_vars e) ->
  has_type (extend env x s) e t.
Proof.
  (* Note: has_type_env_weaken cannot be used directly because it requires
     inclusion for ALL lookups, but extending with x may shadow x's existing
     binding. We use direct induction on the typing derivation instead,
     where the freshness condition lets us handle each case precisely. *)
  intros env e t x s Htype.
  revert x s.
  induction Htype using has_type_ind_strong; intros y s' Hfresh; simpl in Hfresh.
  - constructor.
  - constructor.
  - constructor.
  - constructor.
  - (* T_Var: free_vars (EVar x) = [x], so y <> x *)
    apply T_Var.
    unfold extend. simpl.
    destruct (String.eqb x y) eqn:Heq.
    + apply String.eqb_eq in Heq. subst.
      exfalso. apply Hfresh. left. reflexivity.
    + assumption.
  - (* T_Lam: free_vars (ELam x t1 e) = remove x (free_vars e) *)
    apply T_Lam.
    destruct (string_dec y x) as [Heq | Hneq].
    + (* y = x: (y,s') is shadowed by (x,t1). Use env_equiv. *)
      subst.
      apply has_type_env_equiv with (env1 := extend env x t1).
      * intros z. unfold extend. simpl.
        destruct (String.eqb z x); reflexivity.
      * assumption.
    + (* y <> x: permute then use IH *)
      apply env_permute; [exact Hneq |].
      unfold extend. apply IHHtype.
      intros Hin. apply Hfresh.
      apply in_in_remove; [exact (fun H => Hneq (eq_sym H)) | assumption].
  - (* T_App *)
    eapply T_App.
    + apply IHHtype1. intros H. apply Hfresh. apply in_or_app. left. assumption.
    + apply IHHtype2. intros H. apply Hfresh. apply in_or_app. right. assumption.
  - (* T_Let *)
    eapply T_Let.
    + apply IHHtype1. intros H. apply Hfresh. apply in_or_app. left. assumption.
    + destruct (string_dec y x) as [Heq | Hneq].
      * subst.
        apply has_type_env_equiv with (env1 := extend env x t1).
        -- intros z. unfold extend. simpl.
           destruct (String.eqb z x); reflexivity.
        -- assumption.
      * apply env_permute; [exact Hneq |].
        unfold extend. apply IHHtype2.
        intros Hin. apply Hfresh.
        apply in_or_app. right.
        apply in_in_remove; [exact (fun H => Hneq (eq_sym H)) | assumption].
  - (* T_If *)
    apply T_If.
    + apply IHHtype1. intros H. apply Hfresh.
      apply in_or_app. left. assumption.
    + apply IHHtype2. intros H. apply Hfresh.
      apply in_or_app. right. apply in_or_app. left. assumption.
    + apply IHHtype3. intros H. apply Hfresh.
      apply in_or_app. right. apply in_or_app. right. assumption.
  - (* T_BinOp_Int *)
    apply T_BinOp_Int; [assumption | |].
    + apply IHHtype1. intros H0. apply Hfresh.
      apply in_or_app. left. assumption.
    + apply IHHtype2. intros H0. apply Hfresh.
      apply in_or_app. right. assumption.
  - (* T_BinOp_Cmp *)
    apply T_BinOp_Cmp with (t := t); try assumption.
    + apply IHHtype1. intros H1. apply Hfresh.
      apply in_or_app. left. assumption.
    + apply IHHtype2. intros H1. apply Hfresh.
      apply in_or_app. right. assumption.
  - (* T_BinOp_Eq *)
    apply T_BinOp_Eq with (t := t); try assumption.
    + apply IHHtype1. intros H0. apply Hfresh.
      apply in_or_app. left. assumption.
    + apply IHHtype2. intros H0. apply Hfresh.
      apply in_or_app. right. assumption.
  - (* T_BinOp_Logic *)
    apply T_BinOp_Logic; [assumption | |].
    + apply IHHtype1. intros H0. apply Hfresh.
      apply in_or_app. left. assumption.
    + apply IHHtype2. intros H0. apply Hfresh.
      apply in_or_app. right. assumption.
  - (* T_UnOp_Neg *)
    apply T_UnOp_Neg; [assumption | apply IHHtype; assumption].
  - (* T_UnOp_Not *)
    apply T_UnOp_Not. apply IHHtype. assumption.
  - (* T_Array: free_vars (EArray es) unfolds to the concatenation of
       free_vars for each element, so y fresh for the array implies y
       fresh for each element. The nested Forall IH H0 (from
       has_type_ind_strong) provides the per-element weakening. *)
    apply T_Array.
    clear H. revert Hfresh. induction H0 as [| x xs Hx Hxs IH]; intros Hfresh.
    + apply Forall_nil.
    + apply Forall_cons.
      * apply Hx. intros Hin. apply Hfresh. apply in_or_app. left. exact Hin.
      * apply IH. intros Hin. apply Hfresh. apply in_or_app. right. exact Hin.
  - constructor.
  - constructor.
  - constructor.
  - (* T_Sub *)
    eapply T_Sub; [apply IHHtype; assumption | assumption].
Qed.

(* ================================================================== *)
(** * Substitution Lemma                                                *)
(* ================================================================== *)

(** Substitution for closed values: if v is a closed value (typed in [])
    with type s, and e has type t in (env extended with x:s), then
    [v/x]e has type t in env.

    Design note: The original statement allowed v typed in env (open
    substitution), which requires proving that weakening v into the
    extended environment (extend env z t1) preserves typing. This needs
    the full exchange/permutation infrastructure. The closed-value
    restriction (v typed in []) is standard in call-by-value semantics
    and sidesteps this issue because closed_typing_any_env lifts v's
    typing to any environment trivially.

    The proof uses a generalized lemma (substitution_gen) that abstracts
    over the environment structure, avoiding the need for syntactic
    environment equality in the induction hypothesis. The key insight is
    to characterize the environment via a lookup predicate rather than
    requiring it to be literally (extend env x s).

    The substitution lemma with induction on the derivation fundamentally
    requires that the induction hypothesis be available for sub-derivations
    in PERMUTED environments. This is because descending under a binder
    (ELam y t1 body with y <> x) changes the environment from
    (x,t1)::(y,s')::env to (y,s')::(x,t1)::env.

    Standard solutions:
    1. Use de Bruijn indices (avoids naming entirely)
    2. Prove a generalized substitution lemma with env permutation baked in
    3. Use the locally nameless representation

    For this named representation, we prove the generalized version by
    inducting on the derivation with a fully general environment. *)

Lemma substitution_gen : forall env1 env2 x v e t s,
  has_type [] v s ->
  has_type env1 e t ->
  (forall z tz, lookup env1 z = Some tz ->
    if String.eqb z x then tz = s else lookup env2 z = Some tz) ->
  has_type env2 (subst x v e) t.
Proof.
  intros env1 env2 x v e t s Hv Htype.
  revert env2 x v s Hv.
  induction Htype using has_type_ind_strong; intros env2 y v s' Hv Hincl; simpl.
  - constructor.
  - constructor.
  - constructor.
  - constructor.
  - (* T_Var *)
    destruct (String.eqb y x) eqn:Heq.
    + (* y = x: substitution fires *)
      apply String.eqb_eq in Heq. subst.
      specialize (Hincl x t H). rewrite String.eqb_refl in Hincl.
      subst. apply closed_typing_any_env. assumption.
    + (* y <> x.  Hincl was stated with `String.eqb z y` (z := x after
         specialize), giving `String.eqb x y`; Heq discriminates on
         `String.eqb y x`. Flip one orientation to match. *)
      apply T_Var. specialize (Hincl x t H).
      rewrite (String.eqb_sym x y) in Hincl. rewrite Heq in Hincl. assumption.
  - (* T_Lam.  Pattern var is named `x` (not `x0` — earlier drafts of
       this proof assumed Coq would auto-rename, but current Rocq keeps
       the constructor's own name since it doesn't collide here). *)
    destruct (String.eqb y x) eqn:Heq.
    + (* y = x: bound variable shadows substitution variable *)
      apply String.eqb_eq in Heq. subst.
      apply T_Lam.
      apply has_type_env_weaken with (env1 := extend env x t1).
      * intros z tz Hlook. unfold extend in *. simpl in *.
        destruct (String.eqb z x) eqn:Hzx.
        -- assumption.
        -- specialize (Hincl z tz). rewrite Hzx in Hincl.
           apply Hincl. assumption.
      * assumption.
    + (* y <> x: recurse *)
      apply T_Lam.
      apply IHHtype with (s := s').
      * assumption.
      * intros z tz Hlook.
        unfold extend in Hlook. simpl in Hlook.
        destruct (String.eqb z x) eqn:Hzx.
        -- (* z = x: the lambda's binding *)
           destruct (String.eqb z y) eqn:Hzy.
           ++ (* z = y and z = x, so y = x, contradiction *)
              apply String.eqb_eq in Hzy. apply String.eqb_eq in Hzx.
              subst. rewrite String.eqb_refl in Heq. discriminate.
           ++ unfold extend. simpl. rewrite Hzx. assumption.
        -- (* z <> x *)
           specialize (Hincl z tz Hlook).
           destruct (String.eqb z y) eqn:Hzy.
           ++ assumption.
           ++ unfold extend. simpl. rewrite Hzx. assumption.
  - (* T_App *)
    eapply T_App.
    + apply IHHtype1 with (s := s'); assumption.
    + apply IHHtype2 with (s := s'); assumption.
  - (* T_Let.  Pattern var is `x` (same x0→x rename as T_Lam). *)
    eapply T_Let.
    + apply IHHtype1 with (s := s'); assumption.
    + destruct (String.eqb y x) eqn:Heq.
      * (* y = x: bound variable shadows *)
        apply String.eqb_eq in Heq. subst.
        apply has_type_env_weaken with (env1 := extend env x t1).
        -- intros z tz Hlook. unfold extend in *. simpl in *.
           destruct (String.eqb z x) eqn:Hzx; [assumption |].
           specialize (Hincl z tz). rewrite Hzx in Hincl.
           apply Hincl. assumption.
        -- assumption.
      * apply IHHtype2 with (s := s').
        -- assumption.
        -- intros z tz Hlook.
           unfold extend in Hlook. simpl in Hlook.
           destruct (String.eqb z x) eqn:Hzx.
           ++ destruct (String.eqb z y) eqn:Hzy.
              ** apply String.eqb_eq in Hzy. apply String.eqb_eq in Hzx.
                 subst. rewrite String.eqb_refl in Heq. discriminate.
              ** unfold extend. simpl. rewrite Hzx. assumption.
           ++ specialize (Hincl z tz Hlook).
              destruct (String.eqb z y) eqn:Hzy; [assumption |].
              unfold extend. simpl. rewrite Hzx. assumption.
  - (* T_If *)
    apply T_If.
    + apply IHHtype1 with (s := s'); assumption.
    + apply IHHtype2 with (s := s'); assumption.
    + apply IHHtype3 with (s := s'); assumption.
  - apply T_BinOp_Int; [assumption | apply IHHtype1 with (s := s') | apply IHHtype2 with (s := s')]; assumption.
  - apply T_BinOp_Cmp with (t := t); try assumption;
    [apply IHHtype1 with (s := s') | apply IHHtype2 with (s := s')]; assumption.
  - apply T_BinOp_Eq with (t := t); try assumption;
    [apply IHHtype1 with (s := s') | apply IHHtype2 with (s := s')]; assumption.
  - apply T_BinOp_Logic; [assumption | apply IHHtype1 with (s := s') | apply IHHtype2 with (s := s')]; assumption.
  - apply T_UnOp_Neg; [assumption | apply IHHtype with (s := s')]; assumption.
  - apply T_UnOp_Not. apply IHHtype with (s := s'); assumption.
  - (* T_Array: use the nested Forall IH (H0) from has_type_ind_strong. *)
    simpl. apply T_Array.
    clear H. induction H0 as [| x xs Hx Hxs IH].
    + apply Forall_nil.
    + simpl. apply Forall_cons.
      * apply Hx with (s := s'); assumption.
      * apply IH.
  - constructor.
  - constructor.
  - constructor.
  - eapply T_Sub; [apply IHHtype with (s := s') | assumption]; assumption.
Qed.

(** The main substitution lemma follows as a corollary *)
Lemma substitution : forall env x v e t s,
  has_type [] v s ->
  has_type (extend env x s) e t ->
  has_type env (subst x v e) t.
Proof.
  intros env x v e t s Hv Htype.
  apply substitution_gen with (env1 := extend env x s) (s := s).
  - assumption.
  - assumption.
  - intros z tz Hlook.
    unfold extend in Hlook. simpl in Hlook.
    destruct (String.eqb z x) eqn:Hzx.
    + injection Hlook as Ht. symmetry. assumption.
    + assumption.
Qed.
