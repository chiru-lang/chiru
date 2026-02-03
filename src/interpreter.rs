use crate::graph::*;
use crate::scope::*;
use std::collections::HashMap;
use crate::scope::{ScopeKind, ScopeNode};
use crate::graph::{RegionNode, RegionKind};

pub type PhaseId = usize;

#[derive(Debug)]
pub struct Phase {
    pub id: PhaseId,
    pub name: String,
    pub order: usize,
}

pub struct Capability {
    pub value_id: ValueId,
    pub kind: CapabilityKind,
    pub phase: PhaseId,
    pub active: bool,
}

pub struct Lifetime {
    pub id: LifetimeId,
    pub phase: PhaseId,
    pub active: bool,
}

pub struct UnsafeAssumption {
    pub id: String,
    pub phase: PhaseId,
    pub scope: ScopeId,
    pub values: Vec<ValueId>,
}



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
    pub current_phase: Option<PhaseId>,
    pub phases: Vec<Phase>,
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
            phases: Vec::new(),
            current_phase: None,
        }
    }

    // =====================
    // PHASE MANAGEMENT
    // =====================

    pub fn declare_phase(&mut self, name: String) -> Result<PhaseId, String> {
        if self.phases.iter().any(|p| p.name == name) {
            return Err(format!("Duplicate phase declaration: {}", name));
        }

        let id = self.phases.len();
        self.phases.push(Phase {
            id,
            name,
            order: id,
        });

        if self.current_phase.is_none() {
            self.current_phase = Some(id);
        }

        Ok(id)
    }

    pub fn current_phase(&self) -> PhaseId {
        self.current_phase.expect("No active phase")
    }

    pub fn current_phase_name(&self) -> &str {
        &self.phases[self.current_phase()].name
    }

    // =====================
    // SCOPE MANAGEMENT
    // =====================

    fn fresh_scope_id(&mut self) -> ScopeId {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        id
    }

    pub fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().expect("No active scope")
    }

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

    pub fn exit_scope(&mut self) -> Result<(), String> {
        let scope_id = self.scope_stack.pop()
            .ok_or("Attempted to exit scope, but no scope is active")?;

        // Expire lifetimes in this scope
        for lifetime in self.graph.lifetimes.values_mut() {
            if lifetime.scope == scope_id {
                lifetime.active = false;
            }
        }

        // Expire capabilities in this scope
        self.graph.capabilities.retain(|_, cap| cap.scope != scope_id);

        // Destroy owned values
        let owned_values: Vec<ValueId> = self.graph.ownership_edges
            .iter()
            .filter(|edge| edge.owner == scope_id)
            .map(|edge| edge.value)
            .collect();

        for value_id in owned_values {
            let value = self.graph.values.get_mut(&value_id).unwrap();
            value.alive = false;
        }

        self.graph.ownership_edges.retain(|e| e.owner != scope_id);
        self.scopes.get_mut(&scope_id).unwrap().active = false;

        Ok(())
    }

    // =====================
    // VALUE / REGION
    // =====================

    pub fn declare_region(&mut self, kind: RegionKind) -> Result<RegionId, String> {
        let scope_id = self.current_scope();
        let id = self.next_region_id;
        self.next_region_id += 1;

        self.graph.regions.insert(id, RegionNode {
            id,
            kind,
            scope: scope_id,
        });

        Ok(id)
    }

    pub fn allocate_value(&mut self, region: RegionId) -> Result<ValueId, String> {
        let id = self.next_value_id;
        self.next_value_id += 1;

        self.graph.values.insert(id, ValueNode {
            id,
            region,
            alive: true,
            origin: if self.is_in_unsafe_scope() {
                ValueOrigin::Unsafe
            } else {
                ValueOrigin::Safe
            },
        });

        self.graph.ownership_edges.push(OwnershipEdge {
            value: id,
            owner: self.current_scope(),
        });

        Ok(id)
    }

    // =====================
    // LIFETIMES (PHASE-BOUND)
    // =====================

    pub fn create_lifetime(
        &mut self,
        scope: ScopeId,
        phase: PhaseId,
    ) -> Result<LifetimeId, String> {
        let id = self.next_lifetime_id;
        self.next_lifetime_id += 1;

        self.graph.lifetimes.insert(id, LifetimeNode {
            id,
            scope,
            phase,
            active: true,
        });

        Ok(id)
    }

    // =====================
    // CAPABILITIES (PHASE-BOUND)
    // =====================

    pub fn create_capability(
        &mut self,
        kind: CapabilityKind,
        value: ValueId,
        lifetime: LifetimeId,
        phase: PhaseId,
    ) -> Result<CapabilityId, String> {

        let value_node = self.graph.values.get(&value)
            .ok_or("Capability refers to non-existent value")?;
        if !value_node.alive {
            return Err("Cannot create capability for destroyed value".into());
        }

        let lifetime_node = self.graph.lifetimes.get(&lifetime)
            .ok_or("Capability refers to non-existent lifetime")?;
        if !lifetime_node.active {
            return Err("Cannot create capability with inactive lifetime".into());
        }

        if lifetime_node.phase != phase {
            return Err(format!(
                "Lifetime phase violation: lifetime created in phase `{}`, used in phase `{}`",
                self.phases[lifetime_node.phase].name,
                self.current_phase_name()
            ));
        }

        for cap in self.graph.capabilities.values() {
            if cap.value == value {
                if matches!(cap.kind, CapabilityKind::UniqueMut)
                    || matches!(kind, CapabilityKind::UniqueMut)
                {
                    return Err("Capability conflict: UniqueMut requires exclusivity".into());
                }
            }
        }

        let id = self.next_capability_id;
        self.next_capability_id += 1;

        self.graph.capabilities.insert(id, CapabilityNode {
            id,
            kind,
            value,
            lifetime,
            scope: self.current_scope(),
            phase,
        });

        Ok(id)
    }

    // =====================
    // UNSAFE ASSUMPTIONS (PHASE-BOUND)
    // =====================

    pub fn add_unsafe_assumption(
        &mut self,
        description: String,
        phase: PhaseId,
        affected_values: Vec<ValueId>,
    ) -> Result<AssumptionId, String> {

        if !self.is_in_unsafe_scope() {
            return Err("Unsafe assumptions must be declared inside unsafe scope".into());
        }

        let id = self.next_assumption_id;
        self.next_assumption_id += 1;

        self.graph.unsafe_assumptions.insert(id, UnsafeAssumptionNode {
            id,
            description,
            scope: self.current_scope(),
            phase,
            affected_values,
        });

        Ok(id)
    }

    // =====================
    // HELPERS
    // =====================

    fn is_in_unsafe_scope(&self) -> bool {
        self.scope_stack.iter().any(|id| {
            matches!(self.scopes[id].kind, ScopeKind::Unsafe)
        })
    }

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
    if !self.graph.ownership_edges.iter().any(|e| {
        e.value == value && e.owner == scope_id
    }) {
        return Err("Cannot drop value not owned by current scope".into());
    }

    // Capability check (regardless of phase)
    let has_active_caps = self.graph.capabilities.values().any(|cap| {
        cap.value == value
    });

    if has_active_caps {
        return Err(format!(
            "Cannot drop value {} because active capabilities exist.\n\
             Rule: A value may only be destroyed after all capabilities expire.",
            value
        ));
    }

    // Destroy value
    let value_node = self.graph.values.get_mut(&value)
        .ok_or("Attempted to drop non-existent value")?;
    value_node.alive = false;

    // Remove ownership edges
    self.graph.ownership_edges.retain(|edge| edge.value != value);

    Ok(())
}

}