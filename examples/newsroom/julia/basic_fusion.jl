#!/usr/bin/env julia

"""
Basic Dempster-Shafer belief fusion example in Julia.
"""

using MyNewsroom

println("="^70)
println("Dempster-Shafer Belief Fusion Demo (Julia)")
println("="^70)
println()

# Define frame
θ = Set(["true", "false"])

# Source A: Reuters (high credibility)
println("Source A (Reuters):")
source_a = BeliefMass(Dict(Set(["true"]) => 0.85, θ => 0.15))
println("  Belief in 'true': ", get(source_a.masses, Set(["true"]), 0.0))
println("  Uncertainty: ", get(source_a.masses, θ, 0.0))
println()

# Source B: Twitter (lower credibility)
println("Source B (Twitter):")
source_b = BeliefMass(Dict(Set(["true"]) => 0.60, θ => 0.40))
println("  Belief in 'true': ", get(source_b.masses, Set(["true"]), 0.0))
println("  Uncertainty: ", get(source_b.masses, θ, 0.0))
println()

# Calculate conflict
conflict = calculate_conflict(source_a, source_b)
println("Conflict between sources: ", round(conflict, digits=4))
println()

# Fuse with Dempster's rule
println("Fusing with Dempster's rule:")
result_dempster = fuse_beliefs(source_a, source_b, Dempster)
println("  Combined belief in 'true': ", round(get(result_dempster.masses, Set(["true"]), 0.0), digits=4))
println("  Combined uncertainty: ", round(get(result_dempster.masses, θ, 0.0), digits=4))
println()

# Compare with Yager's rule
println("Fusing with Yager's rule (more conservative):")
result_yager = fuse_beliefs(source_a, source_b, Yager)
println("  Combined belief in 'true': ", round(get(result_yager.masses, Set(["true"]), 0.0), digits=4))
println("  Combined uncertainty: ", round(get(result_yager.masses, θ, 0.0), digits=4))
println()

# Uncertainty intervals
bel = belief(result_dempster, Set(["true"]))
pl = plausibility(result_dempster, Set(["true"]))
println("Uncertainty Intervals:")
println("  [Belief, Plausibility] = [", round(bel, digits=4), ", ", round(pl, digits=4), "]")
println("  Width of interval: ", round(pl - bel, digits=4))
println()

println("="^70)
println("✅ Fusion complete! Both sources agree, increasing confidence.")
println("="^70)
