/// Reusable harness for correctness testing and cross-mode parity validation
///
/// Tolerance Policy:
/// - Floating-point equality: Use relative + absolute tolerance
///   - Relative: 1e-6 (covers ~6 decimal places)
///   - Absolute: 1e-9 (covers values near zero)
///   - Comparison: |a - b| <= (absolute_tol + relative_tol * max(|a|, |b|))
/// - Integer counts: Exact equality required
/// - Serialization: JSON for human-readable golden references
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Floating-point comparison tolerances
pub const FLOAT_ABSOLUTE_TOL: f32 = 1e-9;
pub const FLOAT_RELATIVE_TOL: f32 = 1e-6;

/// Tolerance-aware floating-point comparison
pub fn approx_eq(a: f32, b: f32, abs_tol: f32, rel_tol: f32) -> bool {
    let abs_diff = (a - b).abs();
    let max_val = a.abs().max(b.abs());
    abs_diff <= (abs_tol + rel_tol * max_val)
}

/// Snapshot of agent state for deterministic comparison
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

impl AgentSnapshot {
    /// Compare two snapshots with tolerance
    pub fn approx_eq(&self, other: &Self) -> bool {
        self.id == other.id
            && approx_eq(self.x, other.x, FLOAT_ABSOLUTE_TOL, FLOAT_RELATIVE_TOL)
            && approx_eq(self.y, other.y, FLOAT_ABSOLUTE_TOL, FLOAT_RELATIVE_TOL)
            && approx_eq(self.vx, other.vx, FLOAT_ABSOLUTE_TOL, FLOAT_RELATIVE_TOL)
            && approx_eq(self.vy, other.vy, FLOAT_ABSOLUTE_TOL, FLOAT_RELATIVE_TOL)
    }
}

/// Simulation step snapshot (all agents at a given step)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepSnapshot {
    pub step: u64,
    pub agents: Vec<AgentSnapshot>,
}

impl StepSnapshot {
    /// Compare two step snapshots with tolerance
    pub fn approx_eq(&self, other: &Self) -> bool {
        if self.step != other.step || self.agents.len() != other.agents.len() {
            return false;
        }

        // Agents should be in same order (guaranteed by deterministic simulation)
        self.agents
            .iter()
            .zip(other.agents.iter())
            .all(|(a, b)| a.approx_eq(b))
    }

    /// Detailed comparison with error reporting
    pub fn approx_eq_verbose(&self, other: &Self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.step != other.step {
            errors.push(format!(
                "Step mismatch: expected {}, got {}",
                self.step, other.step
            ));
            return errors;
        }

        if self.agents.len() != other.agents.len() {
            errors.push(format!(
                "Agent count mismatch at step {}: expected {}, got {}",
                self.step,
                self.agents.len(),
                other.agents.len()
            ));
            return errors;
        }

        for (i, (expected, actual)) in self.agents.iter().zip(other.agents.iter()).enumerate() {
            if !expected.approx_eq(actual) {
                if expected.id != actual.id {
                    errors.push(format!(
                        "Agent {} id mismatch: expected {}, got {}",
                        i, expected.id, actual.id
                    ));
                } else {
                    errors.push(format!(
                        "Agent {} ({}) state mismatch: expected ({:.6}, {:.6}, {:.6}, {:.6}), got ({:.6}, {:.6}, {:.6}, {:.6})",
                        i, expected.id,
                        expected.x, expected.y, expected.vx, expected.vy,
                        actual.x, actual.y, actual.vx, actual.vy
                    ));
                }
            }
        }

        errors
    }
}

/// Simulation execution trace (snapshots across all steps)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub name: String,
    pub seed: u64,
    pub num_agents: u32,
    pub num_steps: u64,
    pub snapshots: Vec<StepSnapshot>,
}

impl ExecutionTrace {
    pub fn new(name: &str, seed: u64, num_agents: u32, num_steps: u64) -> Self {
        ExecutionTrace {
            name: name.to_string(),
            seed,
            num_agents,
            num_steps,
            snapshots: Vec::new(),
        }
    }

    /// Add a step snapshot
    pub fn record_step(&mut self, step: u64, agents: Vec<AgentSnapshot>) {
        self.snapshots.push(StepSnapshot { step, agents });
    }

    /// Compare two execution traces with tolerance
    pub fn approx_eq(&self, other: &Self) -> bool {
        if self.name != other.name
            || self.seed != other.seed
            || self.num_agents != other.num_agents
            || self.num_steps != other.num_steps
            || self.snapshots.len() != other.snapshots.len()
        {
            return false;
        }

        self.snapshots
            .iter()
            .zip(other.snapshots.iter())
            .all(|(a, b)| a.approx_eq(b))
    }

    /// Save trace to JSON fixture file
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self).expect("Failed to serialize trace");
        fs::write(path, json)
    }

    /// Load trace from JSON fixture file
    pub fn load(path: &str) -> std::io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Detailed comparison with error reporting
    pub fn approx_eq_verbose(&self, other: &Self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name != other.name {
            errors.push(format!(
                "Name mismatch: expected '{}', got '{}'",
                self.name, other.name
            ));
        }
        if self.seed != other.seed {
            errors.push(format!(
                "Seed mismatch: expected {}, got {}",
                self.seed, other.seed
            ));
        }
        if self.num_agents != other.num_agents {
            errors.push(format!(
                "Agent count mismatch: expected {}, got {}",
                self.num_agents, other.num_agents
            ));
        }
        if self.num_steps != other.num_steps {
            errors.push(format!(
                "Step count mismatch: expected {}, got {}",
                self.num_steps, other.num_steps
            ));
        }

        if self.snapshots.len() != other.snapshots.len() {
            errors.push(format!(
                "Snapshot count mismatch: expected {}, got {}",
                self.snapshots.len(),
                other.snapshots.len()
            ));
            return errors;
        }

        for (expected, actual) in self.snapshots.iter().zip(other.snapshots.iter()) {
            let step_errors = expected.approx_eq_verbose(actual);
            errors.extend(step_errors);
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_eq_tolerance() {
        // Test relative tolerance
        assert!(approx_eq(1.0, 1.0 + 1e-7, 1e-9, 1e-6));
        assert!(!approx_eq(1.0, 1.0 + 1e-5, 1e-9, 1e-6));

        // Test absolute tolerance (near zero)
        assert!(approx_eq(0.0, 1e-10, 1e-9, 1e-6));
        assert!(!approx_eq(0.0, 1e-8, 1e-9, 1e-6));
    }

    #[test]
    fn test_snapshot_comparison() {
        let snap1 = AgentSnapshot {
            id: 1,
            x: 1.0,
            y: 2.0,
            vx: 0.5,
            vy: 0.1,
        };

        let snap2 = AgentSnapshot {
            id: 1,
            x: 1.0 + 1e-10,
            y: 2.0 + 1e-10,
            vx: 0.5 + 1e-10,
            vy: 0.1 + 1e-10,
        };

        assert!(snap1.approx_eq(&snap2));
    }

    #[test]
    fn test_execution_trace_serialization() {
        let mut trace = ExecutionTrace::new("test", 12345, 5, 10);
        trace.record_step(
            0,
            vec![
                AgentSnapshot {
                    id: 0,
                    x: 0.0,
                    y: 0.0,
                    vx: 0.1,
                    vy: 0.2,
                },
                AgentSnapshot {
                    id: 1,
                    x: 5.0,
                    y: 5.0,
                    vx: -0.1,
                    vy: -0.2,
                },
            ],
        );

        let json = serde_json::to_string_pretty(&trace).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"seed\":12345"));

        let loaded: ExecutionTrace = serde_json::from_str(&json).unwrap();
        assert!(trace.approx_eq(&loaded));
    }
}
