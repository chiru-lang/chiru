use crate::graph::ScopeId;

#[derive(Debug)]
pub struct ScopeNode {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    pub active: bool,
}

#[derive(Debug)]
pub enum ScopeKind {
    Module,
    Function,
    Block,
    Unsafe,
}
