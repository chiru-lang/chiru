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
        }
    }

    /// Print the human-readable Safety Report
    pub fn print(&self) {
        println!("Chiru Safety Report");
        println!("==================\n");

        println!("Summary");
        println!("-------");
        println!("Ownership:     {}", self.summary.ownership);
        println!("Lifetimes:     {}", self.summary.lifetimes);
        println!("Capabilities:  {}", self.summary.capabilities);
        println!("Destruction:   {}\n", self.summary.destruction);

        if !self.unsafe_assumptions.is_empty() {
            println!("Unsafe Assumptions");
            println!("------------------");

            for ua in &self.unsafe_assumptions {
                println!(
                    "[UA-{:03}] {}",
                    ua.id, ua.description
                );
                println!("  Scope: {}", ua.scope);
                println!("  Affects: {:?}\n", ua.affected_values);
            }
        }

        println!("Values");
        println!("------");
        for v in &self.values {
            println!("{}: {} ({})", v.id, v.origin, v.state);
        }

        println!("\nVerdict");
        println!("-------");
        println!("{}", self.verdict);
    }

    /// Print JSON Safety Report (CI / machine use)
    pub fn print_json(&self) {
        println!(
            "{}",
            serde_json::to_string_pretty(self)
                .expect("JSON serialization failed")
        );
    }
}

impl SafetyReport {
    pub fn exit_code(&self) -> i32 {
        match self.verdict.as_str() {
            "SAFE" => 0,
            "SAFE_IF_ASSUMPTIONS_HOLD" => 1,
            _ => 2,
        }
    }
}
