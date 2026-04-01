#!/usr/bin/env bash
set -euo pipefail

echo "[1/6] cargo check serial"
cargo check --example financial_serial

echo "[2/6] cargo check multithreaded"
cargo check --example financial_multithreaded --features parallel

echo "[3/6] cargo check mpi"
cargo check --example financial_mpi --features distributed_mpi

echo "[4/6] serial deterministic baseline test"
cargo test --test lib integration::serial_correctness::serial_baseline_matches_financial_fixture

echo "[5/6] docker mpi parity test"
RUN_MPI_DOCKER_TESTS=1 cargo test --test lib integration::mpi_smoke_test::mpi_smoke_via_docker_script -- --nocapture

echo "[6/6] completed"
echo "EOD verification passed."
