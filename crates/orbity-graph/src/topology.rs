//! Topology validation algorithms: Kahn's topological sort, cycle detection, and reachability.

use crate::types::{EdgeKind, GraphDefinition, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TopologyError {
    #[error("Start node '{0}' does not exist in graph definition")]
    StartNodeNotFound(NodeId),
    #[error("Edge references non-existent node: from '{0}' to '{1}'")]
    EdgeNodeNotFound(NodeId, NodeId),
    #[error("Graph contains an illegal cyclic dependency without feedback loop: {0:?}")]
    IllegalCycle(Vec<NodeId>),
    #[error("Graph contains unreachable nodes: {0:?}")]
    UnreachableNodes(Vec<NodeId>),
    #[error("Start node '{0}' cannot reach any terminal node")]
    NoPathToTerminal(NodeId),
    #[error("Duplicate node ID '{0}'")]
    DuplicateNodeId(NodeId),
}

/// Topology analyzer and validator for GraphDefinition.
pub struct TopologyValidator;

impl TopologyValidator {
    /// Validates graph structure and returns an ordered execution plan using Kahn's algorithm
    /// (ignoring feedback loop edges during DAG acyclicity validation).
    pub fn validate_and_plan(graph: &GraphDefinition) -> Result<Vec<NodeId>, TopologyError> {
        // 1. Check start node exists
        if !graph.nodes.contains_key(&graph.start_node) {
            return Err(TopologyError::StartNodeNotFound(graph.start_node.clone()));
        }

        // 2. Check all edge nodes exist
        for edge in &graph.edges {
            if !graph.nodes.contains_key(&edge.from) {
                return Err(TopologyError::EdgeNodeNotFound(edge.from.clone(), edge.to.clone()));
            }
            if !graph.nodes.contains_key(&edge.to) {
                return Err(TopologyError::EdgeNodeNotFound(edge.from.clone(), edge.to.clone()));
            }
        }

        // 3. Build adjacency lists (excluding FeedbackLoop edges for DAG validation)
        let mut adj: HashMap<&NodeId, Vec<&NodeId>> = HashMap::new();
        let mut in_degree: HashMap<&NodeId, usize> = HashMap::new();

        for node_id in graph.nodes.keys() {
            adj.entry(node_id).or_default();
            in_degree.insert(node_id, 0);
        }

        for edge in &graph.edges {
            // FeedbackLoop edges are explicitly permitted cycles, so we exclude them from the DAG skeleton
            if !matches!(edge.kind, EdgeKind::FeedbackLoop { .. }) {
                adj.entry(&edge.from).or_default().push(&edge.to);
                *in_degree.entry(&edge.to).or_insert(0) += 1;
            }
        }

        // 4. Reachability from start node
        let mut visited: HashSet<&NodeId> = HashSet::new();
        let mut queue: VecDeque<&NodeId> = VecDeque::new();
        queue.push_back(&graph.start_node);
        visited.insert(&graph.start_node);

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adj.get(current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Check if there are unreachable nodes
        let mut unreachable: Vec<NodeId> = Vec::new();
        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                unreachable.push(node_id.clone());
            }
        }
        if !unreachable.is_empty() {
            unreachable.sort_by(|a, b| a.0.cmp(&b.0));
            return Err(TopologyError::UnreachableNodes(unreachable));
        }

        // 5. Check if at least one terminal node is reachable if terminal nodes are specified
        if !graph.terminal_nodes.is_empty() {
            let reached_any_terminal = graph.terminal_nodes.iter().any(|t| visited.contains(t));
            if !reached_any_terminal {
                return Err(TopologyError::NoPathToTerminal(graph.start_node.clone()));
            }
        }

        // 6. Kahn's Algorithm for Topological Sort & Cycle Detection
        let mut kahn_in_degree = in_degree.clone();
        let mut kahn_queue: VecDeque<&NodeId> = VecDeque::new();

        // Start with nodes having in_degree == 0
        for (node_id, degree) in &kahn_in_degree {
            if *degree == 0 {
                kahn_queue.push_back(node_id);
            }
        }

        let mut execution_plan: Vec<NodeId> = Vec::new();

        while let Some(node) = kahn_queue.pop_front() {
            execution_plan.push((*node).clone());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if let Some(deg) = kahn_in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            kahn_queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        // If not all nodes were visited by Kahn's algorithm, an uncontrolled cycle exists
        if execution_plan.len() != graph.nodes.len() {
            let cycle_candidates: Vec<NodeId> = graph
                .nodes
                .keys()
                .filter(|id| !execution_plan.contains(id))
                .cloned()
                .collect();
            return Err(TopologyError::IllegalCycle(cycle_candidates));
        }

        Ok(execution_plan)
    }

    /// Finds all predecessor node IDs for a given node.
    pub fn get_predecessors(graph: &GraphDefinition, target: &NodeId) -> Vec<NodeId> {
        graph
            .edges
            .iter()
            .filter(|e| &e.to == target && !matches!(e.kind, EdgeKind::FeedbackLoop { .. }))
            .map(|e| e.from.clone())
            .collect()
    }

    /// Finds all successor edges departing from a given node.
    pub fn get_outgoing_edges<'a>(graph: &'a GraphDefinition, source: &NodeId) -> Vec<&'a crate::types::GraphEdge> {
        graph
            .edges
            .iter()
            .filter(|e| &e.from == source)
            .collect()
    }
}
