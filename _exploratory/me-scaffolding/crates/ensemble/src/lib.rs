// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//! Ensemble dialect — multi-agent coordination for My Language.
//!
//! # Two Variants
//!
//! ## Variant A — Imperative Agent Model (implemented in dialects/me/)
//! Agents are first-class objects with typed state, capability functions,
//! goal conditions, and message handlers. Coordinated via explicit
//! `spawn`/`send`/`receive`/`broadcast` statements.
//!
//! ## Variant B — Declarative Task Model (this crate)
//! Tasks declare `input`/`output` types and a decomposition pipeline.
//! Ensembles bind tasks to topologies and coordination protocols.
//! The runtime assigns agents automatically based on capability matching.
//!
//! # AI Semantics
//! Both variants support `AI<T>` effect types on agent messages, signalling
//! non-determinism, latency, and fallibility. An optional `ai_arbiter`
//! string in Variant B ensembles allows LLM-mediated conflict resolution.

#![forbid(unsafe_code)]

/// Coordination protocol between ensemble members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinationKind {
    /// Majority vote on shared decisions.
    Consensus,
    /// Single elected coordinator delegates work.
    Leader,
    /// Epidemic state dissemination (no central authority).
    Gossip,
    /// Shared write-space; agents post/consume entries.
    Blackboard,
    /// Auction-based task assignment (agents bid on tasks).
    Market,
}

/// Communication topology of the ensemble graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyKind {
    /// Agents arranged in a ring; messages travel one direction.
    Ring,
    /// One hub agent connects to all others.
    Star,
    /// Fully connected; every agent can reach every other.
    Mesh,
    /// Linear data-flow chain; output of each stage feeds the next.
    Pipeline,
    /// All agents share a common bus; broadcast is the default.
    Bus,
}

/// Decomposition strategy for a `task` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStrategy {
    /// Agents process in a linear pipeline.
    Pipeline,
    /// All agents process in parallel; results merged.
    Parallel,
    /// Input scattered across agents; results gathered and merged.
    ScatterGather,
    /// Sequential chain with each agent consuming the prior output.
    Chain,
}

/// A single step in a task decomposition pipeline.
#[derive(Debug, Clone)]
pub enum TaskStep {
    /// Call a named function or invoke a named agent.
    Named(String),
    /// Apply a mapping function element-wise.
    Map(String),
    /// Reduce a collection with a binary function.
    Reduce(String),
    /// Filter elements by a predicate.
    Filter(String),
    /// Split a collection according to a discriminator.
    Partition(String),
}

/// Variant B: declarative task declaration.
///
/// ```text
/// task ComputeSumOfSquares {
///   input:  [Int],
///   output: Int,
///   decompose: map(square) |> reduce(sum),
///   strategy: pipeline,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TaskDecl {
    pub name: String,
    /// Source type fed into the task.
    pub input_type: String,
    /// Result type produced by the task.
    pub output_type: String,
    /// Optional pipeline of decomposition steps.
    pub decompose: Vec<TaskStep>,
    /// Optional list of agent names that may execute this task.
    pub agents: Vec<String>,
    /// How the task partitions work across agents.
    pub strategy: Option<TaskStrategy>,
}

/// Variant B: declarative ensemble declaration.
///
/// ```text
/// ensemble Summariser {
///   agents:      [Chunker, Extractor, Merger],
///   tasks:       [ChunkText, ExtractFacts, MergeFacts],
///   topology:    pipeline,
///   coordination: leader,
///   ai_arbiter:  "resolve ordering conflicts by semantic similarity",
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EnsembleDecl {
    pub name: String,
    /// Agent names participating in this ensemble.
    pub agents: Vec<String>,
    /// Task names this ensemble executes.
    pub tasks: Vec<String>,
    /// Communication topology.
    pub topology: Option<TopologyKind>,
    /// Coordination protocol.
    pub coordination: Option<CoordinationKind>,
    /// Optional LLM prompt used to arbitrate agent conflicts.
    pub ai_arbiter: Option<String>,
}

/// Runtime handle for a running Variant B ensemble instance.
/// Returned by `spawn ensemble <name>` in Variant A code that
/// delegates sub-tasks to a Variant B specification.
#[derive(Debug)]
pub struct EnsembleHandle {
    pub name: String,
    pub topology: TopologyKind,
    pub coordination: CoordinationKind,
    /// Number of active agent slots in this instance.
    pub agent_count: usize,
}

impl EnsembleHandle {
    /// Create a handle for a freshly spawned ensemble.
    pub fn new(decl: &EnsembleDecl) -> Self {
        Self {
            name: decl.name.clone(),
            topology: decl.topology.clone().unwrap_or(TopologyKind::Mesh),
            coordination: decl.coordination.clone().unwrap_or(CoordinationKind::Consensus),
            agent_count: decl.agents.len(),
        }
    }
}
