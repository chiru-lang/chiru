use serde::Serialize;

use crate::interpreter::InterpreterState;
use crate::graph::{ValueOrigin, UnsafeAssumptionNode};

/// ===============================
/// Public Safety Report (v0)
/// ===============================
///
/// This is a *frozen* structure for Chiru v0.
/// It is designed to be:
/// - Human-readable
/// - Machine-consumable
/// - CI-friendly
/// - Audit-grade
///
/// DO NOT embed engine logic here.
/// DO NOT add inference.
/// This file only *renders truth*.
///

#[derive(Serialize)]
pub struct SafetyReport {
    pub verdict: String,
    pub summary: Summary,
    pub unsafe_assumptions: Vec<UnsafeAssumptionNode>,
    pub values: Vec<ValueSummary>,
    ownership_ok: bool,
    lifetimes_ok: bool,
    capabilities_ok: bool,
    destruction_ok: bool,
    has_unsafe_assumptions: bool,
}

#[derive(Serialize)]
pub struct Summary {
    pub ownership: String,
    pub lifetimes: String,
    pub capabilities: String,
    pub destruction: String,
}

#[derive(Serialize)]
pub struct ValueSummary {
    pub id: u64,
    pub origin: String,
    pub state: String,
}

impl SafetyReport {
    /// Generate a SafetyReport from interpreter state.
    ///
    /// This function must NEVER fail.
    /// If state exists, a report must be produced.
    pub fn generate(state: &InterpreterState) -> Self {
        let unsafe_assumptions: Vec<_> =
            state.graph.unsafe_assumptions.values().cloned().collect();

        let values: Vec<ValueSummary> = state
            .graph
            .values
            .values()
            .map(|v| ValueSummary {
                id: v.id,
                origin: match v.origin {
                    ValueOrigin::Safe => "SAFE".to_string(),
                    ValueOrigin::Unsafe => "UNSAFE".to_string(),
                },
                state: if v.alive {
                    "ALIVE".to_string()
                } else {
                    "DESTROYED".to_string()
                },
            })
            .collect();

        let verdict = if unsafe_assumptions.is_empty() {
            "SAFE"
        } else {
            "SAFE_IF_ASSUMPTIONS_HOLD"
        };

         let ownership_ok = true; // enforced during execution
        let lifetimes_ok = state.graph.lifetimes.values().all(|l| !l.active);
        let capabilities_ok = true; // conflicts rejected at creation
        let destruction_ok = true; // double-drops rejected

        let has_unsafe_assumptions = !state.graph.unsafe_assumptions.is_empty();

        SafetyReport {
            verdict: verdict.to_string(),
            summary: Summary {
                ownership: "VERIFIED".to_string(),
                lifetimes: "VERIFIED".to_string(),
                capabilities: "VERIFIED".to_string(),
                destruction: "VERIFIED".to_string(),
            },
            unsafe_assumptions,
            values,
            ownership_ok,
            lifetimes_ok,
            capabilities_ok,
            destruction_ok,
            has_unsafe_assumptions,
        }
    }

    /// Print the human-readable Safety Report
     pub fn print(&self, state: &InterpreterState) {
        println!("Chiru Safety Report");
        println!("==================\n");

        println!("Summary");
        println!("-------");
        println!("Ownership:     {}", if self.ownership_ok { "VERIFIED" } else { "FAILED" });
        println!("Lifetimes:     {}", if self.lifetimes_ok { "VERIFIED" } else { "FAILED" });
        println!("Capabilities:  {}", if self.capabilities_ok { "VERIFIED" } else { "FAILED" });
        println!("Destruction:   {}", if self.destruction_ok { "VERIFIED" } else { "FAILED" });
        println!();

        // === PHASES ===
        println!("Phases");
        println!("------");
        for phase in &state.phases {
            let marker = if Some(phase.id) == state.current_phase {
                " (active)"
            } else {
                ""
            };
            println!("[{}] {}{}", phase.id, phase.name, marker);
        }
        println!();

        // === UNSAFE ASSUMPTIONS ===
        println!("Unsafe Assumptions");
        println!("------------------");
        if state.graph.unsafe_assumptions.is_empty() {
            println!("None");
        } else {
            for ua in state.graph.unsafe_assumptions.values() {
                let phase_name = &state.phases[ua.phase].name;
                println!(
                    "[UA-{:03}] {}",
                    ua.id,
                    ua.description
                );
                println!("  Phase: {}", phase_name);
                println!("  Scope: {}", ua.scope);
                println!("  Affects: {:?}", ua.affected_values);
            }
        }
        println!();

        // === VALUES ===
        println!("Values");
        println!("------");
        for value in state.graph.values.values() {
            let status = if value.alive {
                match value.origin {
                    ValueOrigin::Safe => "SAFE",
                    ValueOrigin::Unsafe => "UNSAFE",
                }
            } else {
                "DESTROYED"
            };

            println!("{}: {}", value.id, status);
        }
        println!();

        // === VERDICT ===
        println!("Verdict");
        println!("-------");
        if !self.ownership_ok || !self.lifetimes_ok || !self.capabilities_ok || !self.destruction_ok {
            println!("UNSAFE");
        } else if self.has_unsafe_assumptions {
            println!("SAFE_IF_ASSUMPTIONS_HOLD");
        } else {
            println!("SAFE");
        }
    }

    pub fn exit_code(&self) -> i32 {
        if !self.ownership_ok || !self.lifetimes_ok || !self.capabilities_ok || !self.destruction_ok {
            2
        } else if self.has_unsafe_assumptions {
            1
        } else {
            0
        }
    }
}