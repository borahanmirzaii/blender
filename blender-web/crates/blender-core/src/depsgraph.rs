use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency graph node, inspired by Blender's depsgraph.
/// Tracks which objects/data depend on others for evaluation ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsgraphNode {
    pub id: u64,
    pub node_type: NodeType,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    ObjectTransform(usize), // object index
    ObjectData(usize),      // object index — mesh/modifiers
    Material(usize),        // material index
    SceneParameters,
}

/// Lightweight dependency graph for reactive evaluation.
/// When a node is marked dirty, all dependents are also dirtied.
#[derive(Debug, Clone, Default)]
pub struct Depsgraph {
    nodes: Vec<DepsgraphNode>,
    /// edges: from → [to] (dependency direction)
    edges: HashMap<usize, Vec<usize>>,
    /// reverse edges for dirty propagation
    reverse_edges: HashMap<usize, Vec<usize>>,
}

impl Depsgraph {
    pub fn add_node(&mut self, node_type: NodeType) -> usize {
        let id = self.nodes.len() as u64;
        self.nodes.push(DepsgraphNode {
            id,
            node_type,
            dirty: true,
        });
        self.nodes.len() - 1
    }

    /// Add dependency: `dependent` depends on `dependency`.
    pub fn add_edge(&mut self, dependency: usize, dependent: usize) {
        self.edges.entry(dependency).or_default().push(dependent);
        self.reverse_edges
            .entry(dependent)
            .or_default()
            .push(dependency);
    }

    /// Mark a node dirty and propagate to all dependents.
    pub fn mark_dirty(&mut self, node_idx: usize) {
        let mut queue = VecDeque::new();
        queue.push_back(node_idx);

        while let Some(idx) = queue.pop_front() {
            if idx < self.nodes.len() && !self.nodes[idx].dirty {
                self.nodes[idx].dirty = true;
                if let Some(deps) = self.edges.get(&idx) {
                    for &dep in deps {
                        queue.push_back(dep);
                    }
                }
            }
        }
        // Also mark the initial node
        if node_idx < self.nodes.len() {
            self.nodes[node_idx].dirty = true;
        }
    }

    /// Topological sort for evaluation order.
    pub fn evaluation_order(&self) -> Vec<usize> {
        let mut in_degree = vec![0usize; self.nodes.len()];
        for deps in self.edges.values() {
            for &dep in deps {
                if dep < in_degree.len() {
                    in_degree[dep] += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|(_, &d)| d == 0)
            .map(|(i, _)| i)
            .collect();

        let mut order = Vec::new();
        let mut visited = HashSet::new();

        while let Some(idx) = queue.pop_front() {
            if !visited.insert(idx) {
                continue;
            }
            if self.nodes[idx].dirty {
                order.push(idx);
            }
            if let Some(deps) = self.edges.get(&idx) {
                for &dep in deps {
                    in_degree[dep] -= 1;
                    if in_degree[dep] == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        order
    }

    /// Clear all dirty flags after evaluation.
    pub fn clear_dirty(&mut self) {
        for node in &mut self.nodes {
            node.dirty = false;
        }
    }
}
