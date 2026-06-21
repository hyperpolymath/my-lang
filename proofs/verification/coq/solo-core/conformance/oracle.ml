(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)
(*
 * Differential-conformance ORACLE for the QTT checker coupling.
 *
 * Links against the OCaml that `Extract.v` extracts from the VERIFIED Coq
 * `check` (R5). Reads one query S-expression per stdin line
 *     (q (ctx TY...) TM)
 * builds the corresponding `SoloCore.tctx` / `SoloCore.tm`, runs the extracted
 * `SoloCore.check`, and prints the result in the SAME canonical form the Rust
 * `conformance_gen` binary prints — so `diff` of the two output streams is the
 * refinement check "Rust `my_qtt::check` == extracted Coq `check`".
 *
 * The oracle parses; it never generates. Inputs come from the Rust side, so the
 * two implementations are genuinely independent.
 *)

open SoloCore
open Quantity

(* ---------- tiny S-expression reader ---------- *)

type sexp = Atom of string | List of sexp list

let tokenize (s : string) : string list =
  let b = Buffer.create (String.length s * 2) in
  String.iter (fun c ->
    match c with
    | '(' | ')' -> Buffer.add_char b ' '; Buffer.add_char b c; Buffer.add_char b ' '
    | _ -> Buffer.add_char b c) s;
  Buffer.contents b
  |> String.split_on_char ' '
  |> List.filter (fun t -> t <> "")

let parse_sexp (toks : string list) : sexp =
  let rec go toks =
    match toks with
    | [] -> failwith "unexpected eof"
    | "(" :: rest ->
      let rec loop acc toks =
        match toks with
        | ")" :: rest -> (List (List.rev acc), rest)
        | [] -> failwith "unterminated list"
        | _ -> let (e, rest') = go toks in loop (e :: acc) rest'
      in loop [] rest
    | ")" :: _ -> failwith "unexpected )"
    | a :: rest -> (Atom a, rest)
  in
  let (e, rest) = go toks in
  (match rest with [] -> () | _ -> failwith "trailing tokens");
  e

(* ---------- sexp -> Coq AST ---------- *)

let q_of = function
  | Atom "0" -> Zero | Atom "1" -> One | Atom "w" -> Omega
  | _ -> failwith "bad q"

let m_of = function
  | Atom "lin" -> EchoMode.Linear | Atom "aff" -> EchoMode.Affine
  | _ -> failwith "bad mode"

let rec ty_of = function
  | Atom "unit" -> TUnit
  | List [Atom "with"; a; b] -> TWith (ty_of a, ty_of b)
  | List [Atom "tensor"; a; b] -> TTensor (ty_of a, ty_of b)
  | List [Atom "sum"; a; b] -> TSum (ty_of a, ty_of b)
  | List [Atom "arr"; q; a; b] -> TArr (q_of q, ty_of a, ty_of b)
  | List [Atom "echo"; m; a; b] -> TEcho (m_of m, ty_of a, ty_of b)
  | _ -> failwith "bad ty"

let rec tm_of = function
  | List [Atom "var"; Atom n] -> Var (int_of_string n)
  | Atom "star" -> UnitT
  | List [Atom "lam"; q; a; t] -> Lam (q_of q, ty_of a, tm_of t)
  | List [Atom "app"; f; x] -> App (tm_of f, tm_of x)
  | List [Atom "with"; a; b] -> With (tm_of a, tm_of b)
  | List [Atom "fst"; t] -> Fst (tm_of t)
  | List [Atom "snd"; t] -> Snd (tm_of t)
  | List [Atom "tensor"; a; b] -> Tensor (tm_of a, tm_of b)
  | List [Atom "letpair"; a; b] -> LetPair (tm_of a, tm_of b)
  | List [Atom "inl"; ty; t] -> Inl (ty_of ty, tm_of t)
  | List [Atom "inr"; ty; t] -> Inr (ty_of ty, tm_of t)
  | List [Atom "case"; s; l; r] -> Case (tm_of s, tm_of l, tm_of r)
  | List [Atom "let"; q; a; b] -> Let (q_of q, tm_of a, tm_of b)
  | List [Atom "mkecho"; m; a; b; t] -> MkEcho (m_of m, ty_of a, ty_of b, tm_of t)
  | List [Atom "weaken"; t] -> Weaken (tm_of t)
  | _ -> failwith "bad tm"

(* context list is OUTERMOST-first; fold into TSnoc so the LAST element is the
   outer snoc = de Bruijn 0 (matches Rust `Vec` where `.last()` is db0). *)
let ctx_of = function
  | List (Atom "ctx" :: tys) ->
    List.fold_left (fun acc s -> TSnoc (acc, ty_of s)) TEmpty tys
  | _ -> failwith "bad ctx"

(* ---------- canonical printer (MUST match the Rust side) ---------- *)

let show_q = function Zero -> "0" | One -> "1" | Omega -> "w"
let show_m = function EchoMode.Linear -> "lin" | EchoMode.Affine -> "aff"

let rec show_ty = function
  | TUnit -> "unit"
  | TWith (a, b) -> "(with " ^ show_ty a ^ " " ^ show_ty b ^ ")"
  | TTensor (a, b) -> "(tensor " ^ show_ty a ^ " " ^ show_ty b ^ ")"
  | TSum (a, b) -> "(sum " ^ show_ty a ^ " " ^ show_ty b ^ ")"
  | TArr (q, a, b) -> "(arr " ^ show_q q ^ " " ^ show_ty a ^ " " ^ show_ty b ^ ")"
  | TEcho (m, a, b) -> "(echo " ^ show_m m ^ " " ^ show_ty a ^ " " ^ show_ty b ^ ")"

(* uvec OUTERMOST-first: recurse into `rest` before emitting the outer `q`, so
   db0 (the outer USnoc) prints LAST — identical to the Rust `Vec` index order. *)
let show_uvec u =
  let rec go = function UEmpty -> [] | USnoc (rest, q) -> go rest @ [show_q q] in
  "[" ^ String.concat " " (go u) ^ "]"

let show_result = function
  | None -> "none"
  | Some (a, d) -> "(ok " ^ show_ty a ^ " " ^ show_uvec d ^ ")"

(* ---------- driver ---------- *)

let () =
  try
    while true do
      let line = input_line stdin in
      if String.trim line <> "" then begin
        let q = parse_sexp (tokenize line) in
        match q with
        | List [Atom "q"; ctx; tm] ->
          let g = ctx_of ctx in
          let t = tm_of tm in
          print_endline (show_result (check g t))
        | _ -> failwith "bad query"
      end
    done
  with End_of_file -> ()
