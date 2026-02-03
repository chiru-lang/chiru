use std::collections::HashMap;

pub type ValueId = u64;
pub type RegionId = u64;
pub type LifetimeId = u64;
pub type CapabilityId = u64;
pub type ScopeId = u64;
pub type AssumptionId = u64;

#[derive(Debug)]
#[derive(Clone)]

pub struct ValueNode {
    pub id: ValueId,
    pub region: RegionId,
    pub alive: bool,
    pub origin: ValueOrigin,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum ValueOrigin {
    Safe,
    Unsafe,
}

#[derive(Debug)]
pub struct RegionNode {
    pub id: RegionId,
    pub kind: RegionKind,
    pub scope: ScopeId,
}

#[derive(Debug)]
pub enum RegionKind {
    Stack,
    Heap,
    External,
    Static,
}

#[derive(Debug)]
pub struct LifetimeNode {
    pub id: LifetimeId,
    pub scope: ScopeId,
    pub active: bool,
    pub(crate) phase: usize,
}

#[derive(Debug)]
pub struct CapabilityNode {
    pub id: CapabilityId,
    pub kind: CapabilityKind,
    pub value: ValueId,
    pub lifetime: LifetimeId,
    pub scope: ScopeId,
    pub(crate) phase: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityKind {
    Own,
    SharedRead,
    UniqueMut,
    ThreadSend,
    ThreadShare,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
pub struct UnsafeAssumptionNode {
    pub id: AssumptionId,
    pub description: String,
    pub scope: ScopeId,
    pub affected_values: Vec<ValueId>,
    pub(crate) phase: usize,
}
#[derive(Debug)]
pub struct OwnershipEdge {
    pub value: ValueId,
    pub owner: ScopeId,
}

#[derive(Debug)]
pub struct ConstraintGraph {
    pub values: HashMap<ValueId, ValueNode>,
    pub regions: HashMap<RegionId, RegionNode>,
    pub lifetimes: HashMap<LifetimeId, LifetimeNode>,
    pub capabilities: HashMap<CapabilityId, CapabilityNode>,
    pub unsafe_assumptions: HashMap<AssumptionId, UnsafeAssumptionNode>,

    pub ownership_edges: Vec<OwnershipEdge>,
}
