(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ============================================================ *)
(* my-lang Solo core: resource-algebra module-type boundary     *)
(*                       (DRAFT — boundary sketch)              *)
(*                                                              *)
(* This file DRAFTS the module-type seam between the concrete    *)
(* three-point quantity semiring (Quantity.v) and the abstract  *)
(* resource algebra the soundness proof actually consumes. It   *)
(* does NOT alter Soundness.v; it validates, by re-instantiating *)
(* the interface from the existing Quantity lemmas, that the    *)
(* boundary has been drawn at the right place.                  *)
(*                                                              *)
(* ---- Provenance of the law set (audit, not textbook) --------*)
(*                                                              *)
(* The SEMIRING module type below bundles EXACTLY the quantity   *)
(* laws that the current soundness development reaches,          *)
(* transitively, from Soundness.v / Typing.v / Usage.v /        *)
(* ContextProps.v. The set was harvested from actual `rewrite`/  *)
(* `apply` citations, not from a canonical semiring             *)
(* axiomatisation, so it is deliberately ASYMMETRIC:            *)
(*                                                              *)
(*   * it has qadd_zero_r but the proof never names qadd_zero_l *)
(*     (uadd_zero_l reduces `qadd Zero q` by simpl instead);    *)
(*   * it carries BOTH distributivity sides but the proof never *)
(*     names qmul_comm;                                         *)
(*   * it includes qmul_assoc only because uscale_compose /      *)
(*     ContextProps cite it (that path is dead w.r.t.          *)
(*     soundness, but the named lemma IS reached at file        *)
(*     scope, so it is kept honest in the closure).            *)
(*                                                              *)
(* A principled SEMIRING would additionally close qmul_one_r    *)
(* for left/right symmetry; the current concrete proof never    *)
(* invokes that name. TWO laws (qadd_zero_l and qmul_zero_l)    *)
(* are included even though the CONCRETE proof reaches them     *)
(* only by simpl-reduction on the finite carrier: the           *)
(* FUNCTORISED proof, over an opaque Q, must cite them by       *)
(* name. We keep them so re-proving uadd_zero_l / uscale_compose*)
(* / uscale_zero AND uadd_uscaleZero_r against an ABSTRACT      *)
(* carrier needs no reach-back into Quantity's computational    *)
(* content. The qmul_zero_l need was caught by the R2           *)
(* functorisation audit (abstract-carrier probe of             *)
(* Soundness.v:1022-1028, uadd_uscaleZero_r, on the live        *)
(* preservation path); without it qmul zero qe is irreducible.  *)
(*                                                              *)
(* DELIBERATELY EXCLUDED (verified unused by the citation       *)
(* audit — grep over Soundness / Typing / Usage / Context-all): *)
(*   * qmul_comm  — never cited anywhere in the proof;          *)
(*   * qle, qle_zero, qle_omega, qle_refl — the entire ordering *)
(*     is never reached by the current soundness proof.         *)
(* The ordering is re-introduced, inertly, in ORDERED_SEMIRING  *)
(* below, where its FUTURE role is documented.                  *)
(*                                                              *)
(* HARD CONSTRAINTS honoured by this draft:                     *)
(*   - no idempotence (nor non-idempotence) of qadd is baked in;*)
(*   - Echo is NOT imported or encoded as a resource instance;  *)
(*   - no final Echo interface is invented;                     *)
(*   - no theorem names from tropical-resource-typing or        *)
(*     echo-types are referenced.                               *)
(* ============================================================ *)

(* ============================================================ *)
(* 1. SEMIRING — the minimal consumed interface.                *)
(* ============================================================ *)

(** The interface the soundness proof consumes once its carrier is
    ABSTRACTED — the R2 functorisation target.

    Carrier [Q], operations [qadd] / [qmul], constants [zero] /
    [one], and the law set the proof needs over an opaque carrier.

    NOTE on the law set: this bundles the TEN laws

      qadd_comm, qadd_assoc, qadd_zero_l, qadd_zero_r,
      qmul_assoc, qmul_one_l, qmul_zero_r, qmul_zero_l,
      qmul_distrib_l, qmul_distrib_r.

    Eight are reached by direct citation in the concrete proof. Two
    ([qadd_zero_l], [qmul_zero_l]) are reached in the CONCRETE proof
    only by [simpl] computing on the finite carrier; they are named
    here because the FUNCTORISED proof, over an opaque [Q], must
    cite them. The [qmul_zero_l] need (left-annihilation under
    [uscale Zero]) was confirmed by the R2 functorisation audit —
    an abstract-carrier probe of Soundness.v:1022-1028
    ([uadd_uscaleZero_r], on the live preservation path) leaves
    [qmul zero qe] irreducible without it.

    We DELIBERATELY OMIT:

      * qmul_comm, qmul_one_r — reached neither by citation nor by
        reduction anywhere in the soundness chain; and
      * qle / qle_refl / qle_zero / qle_omega — the whole ordering
        is never reached by soundness.

    Including [qmul_comm] or [qle] here would over-specify the
    boundary: it would claim the proof depends on facts it does
    not. The ordering's future home is [ORDERED_SEMIRING]. *)
Module Type SEMIRING.

  Parameter Q : Type.

  Parameter qadd : Q -> Q -> Q.
  Parameter qmul : Q -> Q -> Q.

  Parameter zero : Q.
  Parameter one  : Q.

  (* --- additive structure --- *)
  Axiom qadd_comm  : forall a b, qadd a b = qadd b a.
  Axiom qadd_assoc : forall a b c, qadd (qadd a b) c = qadd a (qadd b c).
  Axiom qadd_zero_l : forall q, qadd zero q = q.
  Axiom qadd_zero_r : forall q, qadd q zero = q.

  (* --- multiplicative structure --- *)
  Axiom qmul_assoc  : forall a b c, qmul (qmul a b) c = qmul a (qmul b c).
  Axiom qmul_one_l  : forall q, qmul one q = q.
  Axiom qmul_zero_r : forall q, qmul q zero = zero.
  Axiom qmul_zero_l : forall q, qmul zero q = zero.

  (* --- distributivity (both sides genuinely cited) --- *)
  Axiom qmul_distrib_l : forall a b c, qmul a (qadd b c) = qadd (qmul a b) (qmul a c).
  Axiom qmul_distrib_r : forall a b c, qmul (qadd a b) c = qadd (qmul a c) (qmul b c).

  (* NOTE (excluded, kept as a reminder, NOT as axioms):
       - qmul_comm  : forall a b, qmul a b = qmul b a.       [never reached]
       - qmul_one_r : forall q, qmul q one = q.              [never reached]
     These belong to a canonical semiring but the soundness chain
     reaches them neither by citation nor by carrier reduction, so
     they are not part of this boundary. (Contrast qmul_zero_l, now
     included above: the concrete proof reaches it by simpl-reduction
     and the FUNCTORISED proof needs it named — R2 functorisation
     audit, abstract-carrier probe of Soundness.v:1022-1028.) *)

End SEMIRING.

(* ============================================================ *)
(* 2. ORDERED_SEMIRING — the (currently inert) ordering         *)
(*    extension.                                                *)
(* ============================================================ *)

(** [ORDERED_SEMIRING] extends [SEMIRING] with the subquantity
    ordering [qle] and its compatibility laws (mirroring
    Quantity.v's [qle], [qle_zero], [qle_omega], [qle_refl]).

    WHY THIS EXISTS, AND WHY IT IS INERT:

    The current soundness proof never reaches [qle] (verified by
    the citation audit). This extension is therefore CURRENTLY
    INERT — nothing in Soundness.v depends on it.

    Its purpose is forward-looking. The ordering is the gate at
    which a FUTURE subusage / weakening typing rule attaches: the
    point where "affine" becomes a DISTINCT theorem (a value of
    quantity [a] may stand in where [b] is expected exactly when
    [qle a b]). Until such a rule is added to Typing.v and a
    weakening case is added to the preservation/progress argument,
    [ORDERED_SEMIRING] carries no proof obligation that soundness
    consumes. It is documented here so the seam is named, not so
    it does work today. *)
Module Type ORDERED_SEMIRING.

  Include SEMIRING.

  Parameter qle : Q -> Q -> bool.

  (* Compatibility / sanity laws, mirroring Quantity.v. These are
     the laws a future weakening rule would cite; none is reached
     by the present proof. *)
  Axiom qle_refl  : forall q, qle q q = true.
  Axiom qle_zero  : forall q, qle zero q = true.
  Axiom qle_omega : forall q, qle q one = true \/ qle q one = false.
  (* ^ Deliberately written as a tautology-shaped placeholder for
     the "top element" law: Quantity.v's qle_omega is
     [qle q Omega = true], but [Omega] is concrete and not part of
     the abstract SEMIRING signature (which exposes only zero/one).
     A real ordered-semiring extension would either add a [top]
     parameter or specialise this module to the concrete carrier.
     Kept inert and total here so the module type is inhabitable
     without committing to a top element. *)

End ORDERED_SEMIRING.

(* ============================================================ *)
(* 3. Linear3 — instantiate SEMIRING by reusing Quantity.v.     *)
(* ============================================================ *)

Require Import Quantity.

(** [Linear3] realises [SEMIRING] over the concrete three-point
    quantity semiring {Zero, One, Omega}. Every field maps to the
    existing Quantity.v definition, and every law is discharged by
    the existing Quantity lemma of the same name. This is the
    VALIDATION that the boundary in [SEMIRING] is drawn correctly:
    if the soundness proof's citation closure were wider than the
    nine laws above, this module would still typecheck — but the
    audit (not this module) is what bounds it from above.

    [<: SEMIRING] (transparent ascription) keeps [Q := Quantity.Q]
    etc. definitionally available, which is what a downstream
    consumer wiring this into Soundness.v would want. *)
Module Linear3 <: SEMIRING.

  Definition Q : Type := Quantity.Q.

  Definition qadd : Q -> Q -> Q := Quantity.qadd.
  Definition qmul : Q -> Q -> Q := Quantity.qmul.

  Definition zero : Q := Quantity.Zero.
  Definition one  : Q := Quantity.One.

  Definition qadd_comm  := Quantity.qadd_comm.
  Definition qadd_assoc := Quantity.qadd_assoc.
  Definition qadd_zero_l := Quantity.qadd_zero_l.
  Definition qadd_zero_r := Quantity.qadd_zero_r.

  Definition qmul_assoc  := Quantity.qmul_assoc.
  Definition qmul_one_l  := Quantity.qmul_one_l.
  Definition qmul_zero_r := Quantity.qmul_zero_r.
  Definition qmul_zero_l := Quantity.qmul_zero_l.

  Definition qmul_distrib_l := Quantity.qmul_distrib_l.
  Definition qmul_distrib_r := Quantity.qmul_distrib_r.

End Linear3.

(* A second, sealed instance with OPAQUE ascription, just to
   confirm the laws are sufficient to satisfy the signature
   abstractly (no field leaks). This is purely a boundary check. *)
Module Linear3_Sealed : SEMIRING := Linear3.

(* ============================================================ *)
(* ===== SEAM (PENDING - DO NOT FINALISE) ===================== *)
(* ============================================================ *)
(* This section IDENTIFIES, in prose, where two future          *)
(* interfaces would attach to the resource-algebra boundary.    *)
(* It commits NO types. The only "signatures" below are inert   *)
(* sketches written with [[ ]] brackets (NOT Coq comment        *)
(* delimiters) and are explicitly labelled PENDING. Nothing     *)
(* here compiles as a real interface, and nothing here is       *)
(* consumed by the current soundness proof.                     *)

(* ------------------------------------------------------------ *)
(* (a) ResidueMeasure                                           *)
(* ------------------------------------------------------------ *)
(* A residue measure would map the proof-layer residue object   *)
(* of EchoResidue.v into the carrier of SOME SEMIRING - i.e. a  *)
(* homomorphic "weight" assigning each residue a quantity, so   *)
(* that residue composition lines up with qadd / qmul. The      *)
(* attachment point is the carrier S.Q of a SEMIRING S. The     *)
(* measure's source Residue is the object whose facts           *)
(* EchoResidue.v already mechanises - echo_weaken_keeps_residue *)
(* and the echo_assignable family. NOTHING below is committed:  *)

(*   PENDING sketch only - does NOT compile, NOT a real type:   *)
(*     [[ Module Type RESIDUE_MEASURE (S : SEMIRING).           *)
(*          Parameter Residue : Type.                           *)
(*          Parameter measure : Residue -> S.Q.                 *)
(*          measure respects whatever residue-composition       *)
(*          EchoResidue.v ends up exposing - left OPEN here.    *)
(*        End RESIDUE_MEASURE. ]]                               *)

(* Why PENDING: EchoResidue.v currently exposes residue facts   *)
(* operationally (via the kernel Weaken rule), not as an        *)
(* algebraic object with a composition operator. The measure's  *)
(* target laws cannot be fixed until that algebra is named.     *)

(* ------------------------------------------------------------ *)
(* (b) EchoModality                                             *)
(* ------------------------------------------------------------ *)
(* An echo-modality interface would sit OVER the mode poset of  *)
(* EchoMode.v - the thin two-point order mle, with mle_refl,    *)
(* mle_trans and no_section_weaken already proved there - and   *)
(* describe how passing through the modality acts on a          *)
(* SEMIRING-measured residue, i.e. how Weaken transports a      *)
(* measure. The attachment point is again a SEMIRING carrier,   *)
(* parameterised additionally by EchoMode.Mode. NOTHING below   *)
(* is committed:                                                *)

(*   PENDING sketch only - does NOT compile, NOT a real type:   *)
(*     [[ Module Type ECHO_MODALITY (S : SEMIRING).             *)
(*          over EchoMode.Mode and EchoMode.mle                 *)
(*          Parameter act : Mode -> S.Q -> S.Q.                 *)
(*          act is monotone w.r.t. mle and lax w.r.t.           *)
(*          qadd / qmul - exact laws OPEN until ResidueMeasure  *)
(*          is fixed.                                           *)
(*        End ECHO_MODALITY. ]]                                 *)

(* Why PENDING: the modality's laws depend on (a). We must NOT  *)
(* (per the hard constraints) encode Echo as a resource         *)
(* instance or invent a final Echo interface here, so this      *)
(* remains prose plus a labelled, non-compiling sketch.         *)

(* Citations for the future wiring (read these, do NOT import   *)
(* them as resource instances): EchoResidue.v (the residue      *)
(* object) and EchoMode.v (the mode poset). The                 *)
(* ORDERED_SEMIRING extension above is the natural home for the *)
(* monotonicity side of (b) once it is no longer inert.         *)
(* ============================================================ *)
