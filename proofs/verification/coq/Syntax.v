(* ========================================== *)
(* My Language: Coq Formalization            *)
(* Syntax Definitions                         *)
(* ========================================== *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
Import ListNotations.

(** * Type Syntax *)

Inductive prim_type : Type :=
  | TInt : prim_type
  | TFloat : prim_type
  | TString : prim_type
  | TBool : prim_type.

Inductive ty : Type :=
  | TPrim : prim_type -> ty
  | TUnit : ty
  | TFun : ty -> ty -> ty          (* τ₁ → τ₂ *)
  | TArray : ty -> ty              (* [τ] *)
  | TTuple : list ty -> ty         (* (τ₁, ..., τₙ) *)
  | TRecord : list (string * ty) -> ty
  | TRef : bool -> ty -> ty        (* &τ or &mut τ *)
  | TAI : ty -> ty                 (* AI⟨τ⟩ *)
  | TEffect : ty -> ty             (* Effect⟨τ⟩ *)
  | TVar : nat -> ty               (* Type variable *)
  | TError : ty
  | TNamed : string -> ty.

(** * Expression Syntax *)

Inductive literal : Type :=
  | LInt : Z -> literal
  | LFloat : Q -> literal          (* Using rationals for simplicity *)
  | LString : string -> literal
  | LBool : bool -> literal.

Inductive binop : Type :=
  | OpAdd | OpSub | OpMul | OpDiv
  | OpEq | OpNe | OpLt | OpGt | OpLe | OpGe
  | OpAnd | OpOr
  | OpAssign.

Inductive unop : Type :=
  | OpNeg | OpNot | OpRef | OpRefMut.

Inductive ai_keyword : Type :=
  | AIQuery | AIVerify | AIGenerate | AIEmbed | AIClassify
  | AIOptimize | AITest | AIInfer | AIConstrain | AIValidate.

Inductive pattern : Type :=
  | PLit : literal -> pattern
  | PVar : string -> pattern
  | PWild : pattern
  | PCtor : string -> list pattern -> pattern.

Inductive expr : Type :=
  | ELit : literal -> expr
  | EVar : string -> expr
  | ELam : string -> ty -> expr -> expr     (* λx:τ. e *)
  | EApp : expr -> expr -> expr             (* e₁ e₂ *)
  | ELet : string -> option ty -> expr -> expr -> expr
  | EIf : expr -> expr -> expr -> expr
  | EMatch : expr -> list (pattern * expr) -> expr
  | EField : expr -> string -> expr
  | EBinOp : expr -> binop -> expr -> expr
  | EUnOp : unop -> expr -> expr
  | EArray : list expr -> expr
  | ERecord : list (string * expr) -> expr
  | EBlock : list stmt -> expr
  | EAI : ai_keyword -> list (string * expr) -> expr

with stmt : Type :=
  | SExpr : expr -> stmt
  | SLet : bool -> string -> option ty -> expr -> stmt
  | SIf : expr -> list stmt -> option (list stmt) -> stmt
  | SReturn : option expr -> stmt.

(** * Declaration Syntax *)

Inductive modifier : Type :=
  | ModAsync | ModSafe | ModAIOptimize | ModAITest | ModComptime.

Inductive contract_clause : Type :=
  | CPre : expr -> contract_clause
  | CPost : expr -> contract_clause
  | CInvariant : expr -> contract_clause
  | CAICheck : string -> contract_clause
  | CAIEnsure : string -> contract_clause.

Record contract : Type := mkContract {
  contract_clauses : list contract_clause
}.

Record fn_decl : Type := mkFnDecl {
  fn_modifiers : list modifier;
  fn_name : string;
  fn_params : list (string * ty);
  fn_return : option ty;
  fn_contract : option contract;
  fn_body : list stmt
}.

Record struct_field : Type := mkStructField {
  field_name : string;
  field_type : ty
}.

Record struct_decl : Type := mkStructDecl {
  struct_name : string;
  struct_type_params : list string;
  struct_fields : list struct_field
}.

Inductive top_level : Type :=
  | TLFunction : fn_decl -> top_level
  | TLStruct : struct_decl -> top_level
  | TLEffect : string -> list (string * ty) -> top_level
  | TLAIModel : string -> list (string * string) -> top_level
  | TLPrompt : string -> string -> top_level.

Definition program : Type := list top_level.

(** * Substitution *)

(* TODO: Implement capture-avoiding substitution *)
Fixpoint subst (x : string) (v : expr) (e : expr) : expr :=
  match e with
  | EVar y => if String.eqb x y then v else e
  | ELam y t body =>
      if String.eqb x y then e
      else ELam y t (subst x v body)
  | EApp e1 e2 => EApp (subst x v e1) (subst x v e2)
  | ELet y t e1 e2 =>
      ELet y t (subst x v e1)
        (if String.eqb x y then e2 else subst x v e2)
  | EIf e1 e2 e3 => EIf (subst x v e1) (subst x v e2) (subst x v e3)
  | EBinOp e1 op e2 => EBinOp (subst x v e1) op (subst x v e2)
  | EUnOp op e1 => EUnOp op (subst x v e1)
  | _ => e  (* TODO: Complete all cases *)
  end.

(** * Free Variables *)

(* TODO: Implement free variable computation *)
Fixpoint free_vars (e : expr) : list string :=
  match e with
  | EVar x => [x]
  | ELam x _ body => remove string_dec x (free_vars body)
  | EApp e1 e2 => free_vars e1 ++ free_vars e2
  | ELet x _ e1 e2 => free_vars e1 ++ remove string_dec x (free_vars e2)
  | EIf e1 e2 e3 => free_vars e1 ++ free_vars e2 ++ free_vars e3
  | EBinOp e1 _ e2 => free_vars e1 ++ free_vars e2
  | EUnOp _ e1 => free_vars e1
  | ELit _ => []
  | _ => []  (* TODO: Complete all cases *)
  end.
