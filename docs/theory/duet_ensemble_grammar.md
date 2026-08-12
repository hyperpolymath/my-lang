<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Duet & Ensemble Grammar (inferred)

This document reconstructs the ``duet`` dialect layer (AI collaboration) and the ``ensemble`` layer (agent orchestration) using the Solo core grammar plus the My-Newsroom specifications. The Duet dialect is the middle grammar between Solo and the Ensemble apparatus; Suite musicals are inferred from the `docs/dialects/duet.md` and `docs/dialects/ensemble.md` specs alongside the `comptime orchestrate` example.

## Duet Extensions (AI Collaboration)

```
intent_decl     = "intent" , "(" , string_lit , ")" , expr? , ";"
synth_hole      = "@synth" , "{" , { expr } , "}"
ai_directive    = "#[" , ("ai_optimize" | "ai_test" | "ai_cache" | "ai_hint" , "(" , string_lit , ")") , "]"
verify_marker   = "#[ai_check:" , string_lit , "]"
contract_clause = "ai_check:" , string_lit
```

### Highlights
- **Intent declarations** let the programmer declare high-level goals that Duet will fulfill (`intent("find elements > 5")`).
- **Synth holes** (`@synth { ... }`) mark locations where AI can inject code or suggestions.
- **AI directives** attach metadata to functions, enabling the toolchain to trigger AI optimizations, tests, or caching; the parser treats them like attributes.
- **Contracts** now allow AI check strings; the type checker continues to require boolean expressions and uses the AI metadata for verification.

## Ensemble Extensions (Agentic orchestration)

```
agent_decl      = "agent" , ident , "{" , agent_body , "}"
agent_body      = { agent_state | agent_config | agent_behavior }
agent_state     = ident , ":" , type , ","
agent_config    = ident , ":" , type , ","
agent_behavior  = "fn" , ident , "(" , [ param_list ] , ")" , "->" , type? , block
orchestrate_decl= "comptime" , "orchestrate" , ident , "{" , orchestrate_config , "}"
orchestrate_config = "agents:" , "[" , agent_spec_list , "]" , "," , "fusion:" , fusion_rule , "," , "consensus:" , consensus_rule , "," , "topology:" , topology_type
agent_spec_list = agent_spec , { "," , agent_spec }
agent_spec      = ident , "(" , agent_config_args , ")"
action_chain    = expr , "|>" , expr
belief_stmt     = "belief" , ident , ":" , type , "where" , "confidence" , "(" , float_lit , ")"
fusion_rule     = "Dempster" | "Yager" | "DuboisPrade" | ident
consensus_rule  = "Threshold" , "(" , float_lit , ")" | "Unanimous" | "Quorum" , "(" , int , "," , int , ")"
topology_type  = "Ring" | "Star" | "FullyConnected" | "Custom"
```

### Highlights
- Agents encapsulate states, configs, and behaviors; they compile down to Rust structs (see `docs/dialects/ensemble.md Examples`).
- `comptime orchestrate` defines the static agent topology, fusion rule, and consensus parameters. The Solo parser already handles `comptime` blocks; the ensemble runtime interprets them as orchestration specs.
- `belief` annotations reuse Me/epistemic types (`belief x: Float where confidence(0.85)`), connecting to the Dempster-Shafer core in `my-newsroom/src/dempster_shafer.jl`.
- Fusion/consensus/topology keywords are frozen features tracked in `docs/dialects/ensemble.md`.

## Toolchain Notes
1. **Parser**: The Solo parser (`crates/my-lang/src/parser.rs`) can already lex the new keywords (it reuses the unified grammar), so adding Duet/Ensemble support means extending parse rules for `intent`, `@synth`, `agent`, and `comptime orchestrate` blocks.
2. **AST/Type Checker**: AI directives appear as attributes on functions; the type checker tracks `#[ai_*]` metadata while ensuring contracts remain boolean. Agents map to structs with behavior methods and generate belief updates stored in the epistemic ledger.
3. **Backend**: QBE or LLVM would treat the orchestration as configuration data; runtime (Rust or Elixir/Elixir actors) interprets the `OrchestrationConductor` built from the parsed spec.

This reconstructed grammar sits between the Solo core and the Ensemble interpreter, giving you the missing Duet layer along with the ensemble orchestrations.
