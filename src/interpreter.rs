use crate::graph::*;
use crate::scope::*;
use std::collections::HashMap;
use crate::scope::{ScopeKind, ScopeNode};
use crate::graph::{RegionNode, RegionKind};


pub struct InterpreterState {
    pub graph: ConstraintGraph,
    pub scopes: HashMap<ScopeId, ScopeNode>,
    pub scope_stack: Vec<ScopeId>,

    next_value_id: ValueId,
    next_region_id: RegionId,
    next_lifetime_id: LifetimeId,
    next_capability_id: CapabilityId,
    next_scope_id: ScopeId,
    next_assumption_id: AssumptionId,
}
impl InterpreterState {
    pub fn new() -> Self {
        InterpreterState {
            graph: ConstraintGraph {
                values: HashMap::new(),
                regions: HashMap::new(),
                lifetimes: HashMap::new(),
                capabilities: HashMap::new(),
                unsafe_assumptions: HashMap::new(),
                ownership_edges: Vec::new(),
            },
            scopes: HashMap::new(),
            scope_stack: Vec::new(),
            next_value_id: 1,
            next_region_id: 1,
            next_lifetime_id: 1,
            next_capability_id: 1,
            next_scope_id: 1,
            next_assumption_id: 1,
        }
    }

    fn fresh_scope_id(&mut self) -> ScopeId {
    let id = self.next_scope_id;
    self.next_scope_id += 1;
    id
}

pub fn current_scope(&self) -> ScopeId {
    *self.scope_stack.last().expect("No active scope")
}

fn fresh_region_id(&mut self) -> RegionId {
    let id = self.next_region_id;
    self.next_region_id += 1;
    id
}

fn fresh_value_id(&mut self) -> ValueId {
    let id = self.next_value_id;
    self.next_value_id += 1;
    id
}

fn values_owned_by_scope(&self, scope_id: ScopeId) -> Vec<ValueId> {
    self.graph
        .ownership_edges
        .iter()
        .filter(|edge| edge.owner == scope_id)
        .map(|edge| edge.value)
        .collect()
}

fn fresh_lifetime_id(&mut self) -> LifetimeId {
    let id = self.next_lifetime_id;
    self.next_lifetime_id += 1;
    id
}

fn fresh_capability_id(&mut self) -> CapabilityId {
    let id = self.next_capability_id;
    self.next_capability_id += 1;
    id
}

fn is_owned_by_scope(&self, value: ValueId, scope: ScopeId) -> bool {
    self.graph.ownership_edges.iter().any(|edge| {
        edge.value == value && edge.owner == scope
    })
}

fn is_in_unsafe_scope(&self) -> bool {
    self.scope_stack.iter().any(|scope_id| {
        self.scopes
            .get(scope_id)
            .map(|s| matches!(s.kind, ScopeKind::Unsafe))
            .unwrap_or(false)
    })
}

fn effective_owner_scope(&self) -> ScopeId {
    // Walk the scope stack from top to bottom
    // Return the first NON-unsafe scope
    for scope_id in self.scope_stack.iter().rev() {
        let scope = self.scopes.get(scope_id).unwrap();
        if !matches!(scope.kind, ScopeKind::Unsafe) {
            return *scope_id;
        }
    }
    panic!("No non-unsafe scope found for ownership");
}



}


impl InterpreterState {
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = self.fresh_scope_id();
        let parent = self.scope_stack.last().copied();

        let scope = ScopeNode {
            id,
            parent,
            kind,
            active: true,
        };

        self.scopes.insert(id, scope);
        self.scope_stack.push(id);

        id
    }
}

impl InterpreterState {
    pub fn declare_region(&mut self, kind: RegionKind) -> Result<RegionId, String> {
        let scope_id = *self.scope_stack
            .last()
            .ok_or("Cannot declare region without an active scope")?;

        let id = self.fresh_region_id();

        let region = RegionNode {
            id,
            kind,
            scope: scope_id,
        };

        self.graph.regions.insert(id, region);
        Ok(id)
    }
}

use crate::graph::{ValueNode, ValueOrigin, OwnershipEdge};

impl InterpreterState {
    pub fn allocate_value(&mut self, region: RegionId) -> Result<ValueId, String> {
        let scope_id = self.effective_owner_scope();


        if !self.graph.regions.contains_key(&region) {
            return Err("Region does not exist".into());
        }

        let id = self.fresh_value_id();

        let value = ValueNode {
            id,
            region,
            alive: true,
            origin: if self.is_in_unsafe_scope() {
                ValueOrigin::Unsafe
            } else {
                ValueOrigin::Safe
            },
        };

        self.graph.values.insert(id, value);

        // 🔒 Ownership: value belongs to current scope
        self.graph.ownership_edges.push(OwnershipEdge {
            value: id,
            owner: scope_id,
        });

        Ok(id)
    }
}


impl InterpreterState {
    pub fn exit_scope(&mut self) -> Result<(), String> {
        let scope_id = self.scope_stack.pop()
            .ok_or("Attempted to exit scope, but no scope is active")?;

        // 1. Find all values owned by this scope
        let owned_values = self.values_owned_by_scope(scope_id);
        // Expire capabilities bound to this scope
        self.graph.capabilities.retain(|_, cap| cap.scope != scope_id);


        // 1. Expire lifetimes bound to this scope
            for lifetime in self.graph.lifetimes.values_mut() {
                if lifetime.scope == scope_id {
                    lifetime.active = false;
                }
            }


        // 2. Destroy them
        for value_id in owned_values {
            let value = self.graph.values.get_mut(&value_id)
                .ok_or("Owned value not found in graph")?;

            if !value.alive {
                return Err(format!(
                    "Value {} already destroyed before scope exit",
                    value_id
                ));
            }

            value.alive = false;
        }

        // 3. Remove ownership edges for this scope
        self.graph.ownership_edges.retain(|edge| edge.owner != scope_id);

        // 4. Mark scope inactive
        if let Some(scope) = self.scopes.get_mut(&scope_id) {
            scope.active = false;
        } else {
            return Err("Scope not found during exit".into());
        }

        Ok(())
    }
}


use crate::graph::LifetimeNode;

impl InterpreterState {
    pub fn create_lifetime(&mut self, scope: ScopeId) -> Result<LifetimeId, String> {
        // Scope must exist and be active
        let scope_node = self.scopes.get(&scope)
            .ok_or("Lifetime bound to non-existent scope")?;

        if !scope_node.active {
            return Err("Cannot bind lifetime to inactive scope".into());
        }

        let id = self.fresh_lifetime_id();

        let lifetime = LifetimeNode {
            id,
            scope,
            active: true,
        };

        self.graph.lifetimes.insert(id, lifetime);
        Ok(id)
    }
}


use crate::graph::{CapabilityNode, CapabilityKind};

impl InterpreterState {
    pub fn create_capability(
        &mut self,
        kind: CapabilityKind,
        value: ValueId,
        lifetime: LifetimeId,
    ) -> Result<CapabilityId, String> {

        // Value must exist and be alive
        let value_node = self.graph.values.get(&value)
            .ok_or("Capability refers to non-existent value")?;

        if !value_node.alive {
            return Err("Cannot create capability for destroyed value".into());
        }

        // Lifetime must exist and be active
        let lifetime_node = self.graph.lifetimes.get(&lifetime)
            .ok_or("Capability refers to non-existent lifetime")?;

        if !lifetime_node.active {
            return Err("Cannot create capability with inactive lifetime".into());
        }

        let scope_id = *self.scope_stack
            .last()
            .ok_or("No active scope for capability creation")?;

        // 🔒 CONFLICT CHECKING (core rule)
        for cap in self.graph.capabilities.values() {
            if cap.value == value && cap.lifetime == lifetime {
                match (&cap.kind, &kind) {
                    // Any overlap with UniqueMut is illegal
                    (CapabilityKind::UniqueMut, _) |
                    (_, CapabilityKind::UniqueMut) => {
                        return Err(format!(
                        "Capability conflict on value {}.\n\
                        Existing capability: {:?} (lifetime {})\n\
                        Requested capability: {:?}\n\
                        Rule: UniqueMut requires exclusive access.\n\
                        Suggested fix: End the existing capability's lifetime or use SharedRead instead.",
                        value,
                        cap.kind,
                        cap.lifetime,
                        kind
                    ));

                    }
                    _ => {}
                }
            }
        }

        let id = self.fresh_capability_id();

        let capability = CapabilityNode {
            id,
            kind,
            value,
            lifetime,
            scope: scope_id,
        };

        self.graph.capabilities.insert(id, capability);
        Ok(id)
    }
}

impl InterpreterState {
    pub fn drop_value(&mut self, value: ValueId) -> Result<(), String> {
        let scope_id = *self.scope_stack
            .last()
            .ok_or("No active scope for drop")?;

        {
            let value_node = self.graph.values.get(&value)
                .ok_or("Attempted to drop non-existent value")?;

            if !value_node.alive {
                return Err("Attempted to drop value that is already destroyed".into());
            }
        }

        // Ownership check
        if !self.is_owned_by_scope(value, scope_id) {
            return Err("Cannot drop value not owned by current scope".into());
        }

        // Capability check
        let has_active_caps = self.graph.capabilities.values().any(|cap| {
            cap.value == value && cap.scope == scope_id
        });

        if has_active_caps {
            let caps: Vec<String> = self.graph.capabilities.values()
                .filter(|cap| cap.value == value)
                .map(|cap| format!("{:?} (lifetime {})", cap.kind, cap.lifetime))
                .collect();

            return Err(format!(
                "Cannot drop value {} because active capabilities exist.\n\
                Rule: A value may only be destroyed after all capabilities expire.\n\
                Active capabilities: {:?}\n\
                Suggested fix: End the associated lifetime or allow scope exit to perform destruction.",
                value,
                caps
            ));
        }


        // Destroy value
        let value_node = self.graph.values.get_mut(&value)
            .ok_or("Attempted to drop non-existent value")?;
        value_node.alive = false;

        // Remove ownership edge
        self.graph.ownership_edges.retain(|edge| edge.value != value);

        Ok(())
    }
}

use crate::graph::UnsafeAssumptionNode;

impl InterpreterState {
    pub fn add_unsafe_assumption(
        &mut self,
        description: String,
        affected_values: Vec<ValueId>,
    ) -> Result<AssumptionId, String> {

        if !self.is_in_unsafe_scope() {
            return Err("Unsafe assumptions must be declared inside unsafe scope".into());
        }

        let scope_id = *self.scope_stack.last().unwrap();
        let id = self.next_assumption_id;
        self.next_assumption_id += 1;

        let assumption = UnsafeAssumptionNode {
            id,
            description,
            scope: scope_id,
            affected_values,
        };

        self.graph.unsafe_assumptions.insert(id, assumption);
        Ok(id)
    }
}
