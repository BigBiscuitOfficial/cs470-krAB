pub mod financial_headless_parity;
pub mod mode_work_contract;
pub mod mpi_smoke_test;
/// Integration tests for correctness baselines and cross-mode parity
///
/// Test Strategy:
/// - Deterministic: Fixed seeds, reduced workload (fast CI/local execution)
/// - No flaky timing assertions
/// - Floating-point tolerance: 1e-6 relative, 1e-9 absolute
/// - Reusable harness for serial vs MPI vs multithreaded vs hybrid parity
/// - Golden fixture files for regression detection
pub mod parity_harness;
pub mod serial_correctness;
