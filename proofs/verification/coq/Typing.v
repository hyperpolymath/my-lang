(* ========================================== *)
(* My Language: Coq Formalization            *)
(* Typing Rules                               *)
(* ========================================== *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
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
      (* Simplified: AI query returns AI<String> *)
      has_type env (EAI AIQuery fields) (TAI (TPrim TString))

  | T_AI_Verify : forall env fields,
      has_type env (EAI AIVerify fields) (TAI (TPrim TBool))

  | T_AI_Embed : forall env fields,
      has_type env (EAI AIEmbed fields) (TAI (TArray (TPrim TFloat))).

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

(** * Subtyping *)

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

(** * Subsumption *)

Lemma subsumption : forall env e t1 t2,
  has_type env e t1 ->
  subtype t1 t2 ->
  has_type env e t2.
Proof.
  (* TODO: Prove subsumption *)
Admitted.

(** * Environment Weakening *)

Lemma weakening : forall env e t x s,
  has_type env e t ->
  has_type (extend env x s) e t.
Proof.
  (* TODO: Prove weakening *)
Admitted.

(** * Substitution Lemma *)

Lemma substitution : forall env x v e t s,
  has_type env v s ->
  has_type (extend env x s) e t ->
  has_type env (subst x v e) t.
Proof.
  (* TODO: Prove substitution lemma *)
Admitted.
