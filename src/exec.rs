use crate::ast::AstNode;
use crate::interpreter::InterpreterState;
use crate::scope::ScopeKind;
use crate::graph::{RegionKind, CapabilityKind};

use std::collections::HashMap;

pub struct ExecContext {
    pub lifetimes: HashMap<String, u64>,
    pub regions: HashMap<String, u64>,
    pub values: HashMap<String, u64>,
}

impl ExecContext {
    pub fn new() -> Self {
        ExecContext {
            lifetimes: HashMap::new(),
            regions: HashMap::new(),
            values: HashMap::new(),
        }
    }
}

pub fn execute(
    nodes: &[AstNode],
    state: &mut InterpreterState,
    ctx: &mut ExecContext,
) -> Result<(), String> {
    for node in nodes {
        match node {
            // ─────────────────────────────
            // Scopes
            // ─────────────────────────────
            AstNode::Function { name: _, body } => {
                state.enter_scope(ScopeKind::Function);
                execute(body, state, ctx)?;
                state.exit_scope()?;
            }

            AstNode::Unsafe { body } => {
                state.enter_scope(ScopeKind::Unsafe);
                execute(body, state, ctx)?;
                state.exit_scope()?;
            }

            // ─────────────────────────────
            // Region
            // ─────────────────────────────
            AstNode::Region { kind, name } => {
                let k = match kind.as_str() {
                    "heap" => RegionKind::Heap,
                    "stack" => RegionKind::Stack,
                    "external" => RegionKind::External,
                    "static" => RegionKind::Static,
                    _ => return Err(format!("Unknown region kind: {}", kind)),
                };

                let id = state.declare_region(k)?;
                ctx.regions.insert(name.clone(), id);
            }

            // ─────────────────────────────
            // Lifetime
            // ─────────────────────────────
            AstNode::Lifetime { name, scope: _ } => {
                let scope_id = state.current_scope();
                let id = state.create_lifetime(scope_id)?;
                ctx.lifetimes.insert(name.clone(), id);
            }

            // ─────────────────────────────
            // Let (value allocation)
            // ─────────────────────────────
            AstNode::Let { name, region } => {
                let region_id = ctx
                    .regions
                    .get(region)
                    .ok_or(format!("Unknown region: {}", region))?;

                let value_id = state.allocate_value(*region_id)?;
                ctx.values.insert(name.clone(), value_id);
            }

            // ─────────────────────────────
            // Capability
            // ─────────────────────────────
            AstNode::Capability {
                kind,
                value,
                lifetime,
            } => {
                let cap_kind = match kind.as_str() {
                    "Own" => CapabilityKind::Own,
                    "SharedRead" => CapabilityKind::SharedRead,
                    "UniqueMut" => CapabilityKind::UniqueMut,
                    _ => return Err(format!("Unknown capability kind: {}", kind)),
                };

                let value_id = ctx
                    .values
                    .get(value)
                    .ok_or(format!("Unknown value: {}", value))?;

                let lifetime_id = ctx
                    .lifetimes
                    .get(lifetime)
                    .ok_or(format!("Unknown lifetime: {}", lifetime))?;

                state.create_capability(cap_kind, *value_id, *lifetime_id)?;
            }

            // ─────────────────────────────
            // Drop
            // ─────────────────────────────
            AstNode::Drop { value } => {
                let value_id = ctx
                    .values
                    .get(value)
                    .ok_or(format!("Unknown value: {}", value))?;

                state.drop_value(*value_id)?;
            }

            // ─────────────────────────────
            // Unsafe assumption
            // ─────────────────────────────
            AstNode::Assume { text } => {
                let affected: Vec<u64> = ctx.values.values().copied().collect();
                state.add_unsafe_assumption(text.clone(), affected)?;
            }
        }
    }

    Ok(())
}
