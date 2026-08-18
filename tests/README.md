# V.I.S.O.R. Testing & Verification Suite

This directory houses the comprehensive test suites, benchmarks, protocol validation scripts, and verification reports for the **V.I.S.O.R.** system.

---

## Structure

- **[`TEST_REPORT.md`](./TEST_REPORT.md)**: Full verification report detailing all unit test results, latency benchmarks, edge-case handling, and hardware-in-the-loop (HIL) verification protocols.
- **[`arduino_protocol_suite.py`](./arduino_protocol_suite.py)**: Automated Python test harness validating serial command framing, supply permutations, buffer overflow protection, and error emission.
- **[`../src/pi_logic/tests/`](../src/pi_logic/tests/)**:
  - **`pipeline_integration_test.rs`**: End-to-end integration test simulating the entire audio $\to$ camera $\to$ AI $\to$ serial dispensing $\to$ video guidance pipeline.
  - **`latency_benchmarks.rs`**: High-resolution latency profiling across audio downmixing, JSON deserialization, and packet serialization.

---

## Running the Tests

### 1. Pi Logic Rust Tests & Benchmarks
```powershell
cd src/pi_logic
cargo test -- --nocapture
```

### 2. Arduino Protocol Verification Suite
```powershell
python tests/arduino_protocol_suite.py
```
