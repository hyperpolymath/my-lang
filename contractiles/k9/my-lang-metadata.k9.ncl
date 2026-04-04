K9!
# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
# K9 Kennel-level guard: my-lang project metadata and dialect invariants
# Security Level: Kennel (data-only, no execution)
# No signature required

{
  pedigree = {
    schema_version = "1.0.0",
    component_type = "language-compiler-metadata",
    security = {
      leash = 'Kennel,
      trust_level = "data-only",
      allow_network = false,
      allow_filesystem_write = false,
      allow_subprocess = false,
    },
    metadata = {
      name = "my-lang-metadata",
      version = "0.1.0",
      description = "My-Lang: multi-dialect language with Me/Solo/Duet/Ensemble — project invariants",
      author = "Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>",
    },
  },

  # Project configuration
  config = {
    # Required dialects that must be present
    required_dialects = [
      "me",
      "solo",
      "duet",
      "ensemble",
    ],

    # Ensemble dialect specifics (Variant A + Variant B)
    ensemble = {
      # Variant A: imperative agent/workflow (implemented)
      variant_a_types = ["agent_decl", "workflow_decl"],
      # Variant B: declarative task-distribution (implemented 2026-04-04)
      variant_b_types = ["task_decl", "ensemble_decl"],
      # Coordination kinds
      coordination_kinds = ["Consensus", "Leader", "Gossip", "Blackboard", "Market"],
      # Topology kinds
      topology_kinds = ["Ring", "Star", "Mesh", "Pipeline", "Bus"],
    },

    # 7-Tentacles: NOT a language — educational framework only
    seven_tentacles_is_not_a_language = true,

    # Safety invariants
    safety = {
      forbidden_patterns = [
        "unwrap()",
        "expect()",
        "todo!()",
        "unimplemented!()",
      ],
    },

    # Test requirements
    tests = {
      min_unit_tests = 30,
      dialect_tests_required = true,
    },

    # Build configuration
    build = {
      edition = "2021",
      msrv = "1.75",
      workspace_resolver = "2",
    },

    # Language policy
    banned_files = ["Makefile", "Dockerfile"],
    banned_languages = ["python", "typescript", "javascript", "go"],
  },

  # RSR compliance
  rsr = {
    explainme = false,  # Not required for language sub-repos
    ai_manifest = false,
    machine_readable = true,
    spdx_headers_required = true,
    security_md_required = false,
    idris2_abi = false,
    zig_ffi = false,
  },
}
