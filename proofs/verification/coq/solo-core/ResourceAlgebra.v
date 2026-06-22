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

  (* TRUSTED-BASE NOTE (Trusted-base reduction policy). Every [Axiom] in
     this and the two module types below is a MODULE-TYPE INTERFACE FIELD
     — an abstract parameter of the parametric soundness functor
     [SoloCoreF], NOT a global assumption admitted into the trusted base.
     Each is DISCHARGED by every concrete instance: [Module Linear3 <:
     ORDERED_SEMIRING] proves them by [destruct; reflexivity] (real [Qed]),
     [Module Tropical <: ORDERED_SEMIRING] likewise at the infinite carrier.
     That is exactly why [Print Assumptions progress] / [preservation] is
     "Closed under the global context" for the CONCRETE development
     ([Include SoloCoreF Linear3]) — CI asserts this in proofs.yml. The
     per-line [AXIOM:] annotations below satisfy the policy scanner. *)

  (* --- additive structure --- *)
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qadd_comm  : forall a b, qadd a b = qadd b a.
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qadd_assoc : forall a b c, qadd (qadd a b) c = qadd a (qadd b c).
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qadd_zero_l : forall q, qadd zero q = q.
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qadd_zero_r : forall q, qadd q zero = q.

  (* --- multiplicative structure --- *)
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qmul_assoc  : forall a b c, qmul (qmul a b) c = qmul a (qmul b c).
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qmul_one_l  : forall q, qmul one q = q.
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qmul_zero_r : forall q, qmul q zero = zero.
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qmul_zero_l : forall q, qmul zero q = zero.

  (* --- distributivity (both sides genuinely cited) --- *)
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
  Axiom qmul_distrib_l : forall a b c, qmul a (qadd b c) = qadd (qmul a b) (qmul a c).
  (* AXIOM: SEMIRING interface law; discharged by each <: SEMIRING instance. *)
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
(* 2. ORDERED_SEMIRING — ordering extension (LIVE as of R3:      *)
(*    the affine layer consumes qle).                           *)
(* ============================================================ *)

(** [ORDERED_SEMIRING] extends [SEMIRING] with the subquantity
    ordering [qle] and the PREORDER laws the affine layer consumes.

    LIVE AS OF R3 (no longer inert). The consolidated functor
    [SoloCoreF] is now parameterised by [ORDERED_SEMIRING], and the
    affine budget order [ule] (pointwise [qle]) drives the DISTINCT
    [affine_pres] theorem — "a term that fits a usage budget still
    fits it after a step". The LINEAR soundness proofs
    ([progress]/[preservation]) use only the [SEMIRING] laws; [qle]
    is consumed exclusively by the affine layer, so widening the
    parameter does not disturb them.

    Laws: [qle] is a PREORDER ([qle_refl], [qle_trans]) with [zero]
    a bottom element ([qle_zero]). No top-element law — [Omega] is
    concrete and outside the abstract signature, and the affine
    layer does not need a top. *)
Module Type ORDERED_SEMIRING.

  Include SEMIRING.

  Parameter qle : Q -> Q -> bool.

  (* AXIOM: ORDERED_SEMIRING interface law; discharged by each <: ORDERED_SEMIRING instance. *)
  Axiom qle_refl  : forall q, qle q q = true.
  (* AXIOM: ORDERED_SEMIRING interface law; discharged by each <: ORDERED_SEMIRING instance. *)
  Axiom qle_trans : forall a b c, qle a b = true -> qle b c = true -> qle a c = true.
  (* AXIOM: ORDERED_SEMIRING interface law; discharged by each <: ORDERED_SEMIRING instance. *)
  Axiom qle_zero  : forall q, qle zero q = true.

  (* Decidable carrier equality (R5). The executable usage-walk checker
     [check] (SoloCore.v) compares quantities — a lambda/let body must
     use its binder with EXACTLY the declared quantity, additive pairs
     must share one usage, case branches must agree — so the algorithmic
     presentation needs to DECIDE [Q] equality. The declarative
     soundness proofs (R2/R3/R4) never use it; it is consumed only by
     the checker. Both concrete carriers discharge it trivially
     (three-point by [decide equality]; tropical via [Nat.eq_dec]). *)
  Parameter Q_eq_dec : forall x y : Q, {x = y} + {x <> y}.

End ORDERED_SEMIRING.

(* ============================================================ *)
(* 3. Linear3 — instantiate SEMIRING by reusing Quantity.v.     *)
(* ============================================================ *)

Require Import Quantity.

(** [Linear3] realises [ORDERED_SEMIRING] over the concrete
    three-point quantity semiring {Zero, One, Omega}. Every field
    maps to the existing Quantity.v definition, and every law is
    discharged by the existing Quantity lemma of the same name.
    This is the VALIDATION that the boundary is drawn correctly: if
    the soundness proof's citation closure were wider than the law
    set above, this module would still typecheck — but the audit
    (not this module) is what bounds it from above.

    [<: ORDERED_SEMIRING] (transparent ascription) keeps
    [Q := Quantity.Q] etc. definitionally available, which is what
    a downstream consumer wiring this into SoloCore.v would want. *)
Module Linear3 <: ORDERED_SEMIRING.

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

  (* --- order (ORDERED_SEMIRING, R3) --- *)
  Definition qle       := Quantity.qle.
  Definition qle_refl  := Quantity.qle_refl.
  Definition qle_trans := Quantity.qle_trans.
  Definition qle_zero  := Quantity.qle_zero.

  (* --- decidable equality (R5) --- *)
  Definition Q_eq_dec : forall x y : Q, {x = y} + {x <> y}.
  Proof. unfold Q; decide equality. Defined.

End Linear3.

(* A second, sealed instance with OPAQUE ascription, just to
   confirm the laws are sufficient to satisfy the signature
   abstractly (no field leaks). This is purely a boundary check. *)
Module Linear3_Sealed : SEMIRING := Linear3.

(* ============================================================ *)
(* ===== SEAM (E4) — the residue-measure capstone ============= *)
(* ============================================================ *)
(* The my-lang Coq HALF of the joint E4 seam capstone — the      *)
(* boundary where "interaction, not identification" becomes a    *)
(* theorem. The seam is already MECHANISED UPSTREAM on BOTH      *)
(* foundations; per the echo-types-audit directive this file     *)
(* MIRRORS the upstream INTERFACE (reuse, do not re-invent). The *)
(* concrete witness + measure-independence live downstream in    *)
(* EchoMeasure.v (which may Require Tropical — this file may     *)
(* NOT, to avoid a Require cycle).                               *)
(*                                                              *)
(* Upstream sources MIRRORED here (read, do NOT import as        *)
(* resource instances):                                         *)
(*   * tropical-resource-typing Resource/EchoBridge.lean —       *)
(*       structure ResidueMeasure (combine / empty +            *)
(*       measure_empty / measure_combine monoid-hom laws): the   *)
(*       "residue algebra WITH a composition operator" the       *)
(*       former PENDING note named as the gating step. It is     *)
(*       now named upstream, so the seam is unblocked.           *)
(*   * echo-types Echo/Measure/Interface.agda (ResidueMeasure    *)
(*       record: measure + monotone) and                        *)
(*       Echo/Separation/NotResourceInstance.agda                *)
(*       (equal-measure-does-not-imply-equal-echo — the          *)
(*       measure-INDEPENDENCE theorems, --safe --without-K).     *)
(*                                                              *)
(* HARD IS-NOT INVARIANT (estate-load-bearing): Echo is NOT a    *)
(* resource / SEMIRING instance. [measure] is a ONE-DIRECTIONAL  *)
(* lossy decoration  Residue -> S.Q  (E -> R), never R -> E.     *)
(* EchoMode.v / EchoResidue.v stay Quantity-independent; nothing *)
(* Includes a SEMIRING into the echo modules. The measure        *)
(* OBSERVES residues; it never reconstructs them (witnessed by   *)
(* echo_measure_not_injective in EchoMeasure.v).                 *)

(* (a) The SEAM interface — a residue algebra (a monoid           *)
(* (Residue, combine, empty)) equipped with a measure            *)
(* HOMOMORPHISM into a SEMIRING carrier. Faithful mirror of       *)
(* EchoBridge.lean's [structure ResidueMeasure]: the empty        *)
(* residue measures as [one], and a composite measures as the     *)
(* [qmul] of the parts — i.e. [measure] is a monoid homomorphism  *)
(* (Residue, combine, empty) -> (S.Q, qmul, one). It needs only   *)
(* SEMIRING (no Tropical, no Echo import), so it lives here at     *)
(* the resource-algebra boundary where the PENDING sketch was.    *)
Module Type RESIDUE_MEASURE (S : SEMIRING).
  Parameter Residue : Type.
  Parameter empty   : Residue.
  Parameter combine : Residue -> Residue -> Residue.
  Parameter measure : Residue -> S.Q.
  (* AXIOM: RESIDUE_MEASURE interface law (monoid-homomorphism unit);
     discharged by each <: RESIDUE_MEASURE instance (e.g. EchoTraceTropical). *)
  Axiom measure_empty   : measure empty = S.one.
  (* AXIOM: RESIDUE_MEASURE interface law (monoid-homomorphism multiplicativity);
     discharged by each <: RESIDUE_MEASURE instance (e.g. EchoTraceTropical). *)
  Axiom measure_combine : forall a b,
    measure (combine a b) = S.qmul (measure a) (measure b).
End RESIDUE_MEASURE.

(* (b) EchoModality — the [act : Mode -> S.Q -> S.Q] transport of *)
(* a measure through a weakening — is NOT separately mechanised   *)
(* in the Coq mirror: it is realised UPSTREAM by echo-types       *)
(* Echo/Modality/Core (degrade-compose, the residue path-         *)
(* independence law) and is CITED, not re-derived. The Coq mirror *)
(* carries the measure HOMOMORPHISM (a) + the residue-level non-  *)
(* injectivity witness (EchoMeasure.v) only. ORDERED_SEMIRING's   *)
(* [qle] above stays the natural home for the monotonicity side   *)
(* should (b) ever be brought in-tree.                            *)
(* ============================================================ *)
