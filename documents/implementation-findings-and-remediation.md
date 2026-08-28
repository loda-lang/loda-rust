# LODA-RUST implementation findings and remediation plan

## Scope and baseline

This document summarizes findings from a focused review of the LODA-RUST implementation on the `develop` branch at commit `1303b3aa`.

The review covered:

- parsing and construction of executable programs;
- program execution and state management;
- loop implementations;
- dependency loading and linking;
- result caching and resource limits;
- workspace organization and test coverage.

The default workspace test suite passed during the review:

- `loda-rust-core`: 214 tests passed;
- `loda-rust-cli`: 223 tests passed;
- `loda-rust-web`: no tests;
- total: 437 tests passed, with no failures.

The repository contains many more test functions in the optional ARC subsystem. Those tests are excluded from the default workspace build because the `loda-rust-arc` feature is disabled by default.

## Executive summary

The LODA execution core has a sound overall architecture. Parsing, semantic validation, executable nodes, runtime state, dependency linking, caching, and the command-line and WASM consumers are separated in understandable layers.

The highest-priority work is to fix three correctness and resource-control problems:

1. A constant loop-range limit uses XOR instead of a power-of-two operation.
2. Cached results can bypass execution limits.
3. Failed dependency loads can leave stale cycle-detection state behind.

After those defects, the most valuable work is performance measurement and reducing the cost of cloning program state inside loops. Larger architectural changes, especially splitting the ARC and mining subsystems into separate crates, should be treated as maintainability improvements rather than urgent correctness work.

## Priority 1: correctness and resource control

### 1. Fix the constant loop-range limit

Location:

`rust_project/loda-rust-core/src/parser/create_program.rs`, in `node_loop_range_parameter_constant`.

The current check is:

```rust
if range_length >= (2 ^ LOOP_RANGE_MAX_BITS) {
```

In Rust, `^` is bitwise XOR. It is not exponentiation. With `LOOP_RANGE_MAX_BITS` equal to 8, `2 ^ 8` evaluates to 10. Consequently, constant loop ranges from 10 through 255 are rejected even though other parts of the runtime use a 256-value boundary.

Replace the expression with an explicit power-of-two operation, preferably:

```rust
let range_length_limit = 1_u64 << LOOP_RANGE_MAX_BITS;
if range_length >= range_length_limit {
```

The documentation and implementation should also agree on whether 256 itself is allowed. Existing runtime checks use `>= 1 << LOOP_RANGE_MAX_BITS`, which means 255 is allowed and 256 is rejected.

Required regression tests:

- range 9 is accepted;
- range 10 is accepted;
- range 255 is accepted;
- range 256 is rejected;
- a program using a range above 9 executes with the expected result.

### 2. Prevent cached results from bypassing execution limits

Locations:

- `rust_project/loda-rust-core/src/execute/program_runner.rs`;
- `rust_project/loda-rust-core/src/execute/program_cache.rs`.

`ProgramRunner::run_without_offset` checks the cache before creating a `ProgramState` and before enforcing the current execution policies. A result computed under permissive settings can therefore be returned later under stricter settings without reapplying those settings.

Policies that can be bypassed include:

- the cumulative step-count limit;
- the loop-count limit;
- the register/value limit selected through `NodeRegisterLimit`;
- arithmetic restrictions selected by the execution profile.

The output of a successfully evaluated mathematical program should be independent of these policies, but the decision between success and failure is not. A cache hit must not turn an execution that should be rejected under the current policy into a success.

At minimum, a cache hit must check whether adding the cached step count exceeds the current step-count limit before returning the value.

For the other policies, choose and document one of these designs:

1. Scope each `ProgramCache` to one immutable execution-policy configuration.
2. Include the policy configuration in the cache key.
3. Store enough execution metadata with the cached value to prove that it is valid under the requested policy.
4. Disable cache reuse when policies differ.

The first or fourth design is likely the simplest and safest. If the cache is deliberately scoped by convention today, encode that convention in the type system or API so callers cannot accidentally violate it.

Required regression tests:

- warm the cache with an unlimited run, then repeat with a step limit below the cached step count; the second run must fail;
- warm the cache under unlimited arithmetic semantics, then run under `SmallLimits`; the strict run must not reuse a result that should have failed;
- verify behavior when loop and register limits differ between the cold and warm runs;
- confirm that a cache hit still reports the correct cumulative step count.

### 3. Clean up dependency-loading state after every failure

Location:

`rust_project/loda-rust-core/src/control/dependency_manager.rs`, in `DependencyManager::load_inner`.

The program ID is inserted into `programids_currently_loading` before file reading, parsing, program construction, and recursive dependency loading. It is removed only on the successful path.

If any intermediate operation returns an error, the ID remains in the set. A later retry with the same `DependencyManager` can then report a cyclic dependency even when no cycle exists.

Refactor the function so removal is guaranteed on every exit path after insertion. Possible approaches include:

- wrapping the fallible work in an inner closure and removing the ID before returning its result;
- introducing a small scoped guard responsible for removal;
- separating traversal state from persistent manager state and making traversal state local to one top-level load.

The third approach is the cleanest long-term design because cycle detection belongs to one dependency traversal rather than the lifetime of the manager.

Required regression tests:

- fail because a virtual file is missing, insert the file, then retry successfully with the same manager;
- fail because a program contains invalid syntax, replace the content, then retry successfully;
- fail while loading a transitive dependency, repair it, then retry the root program;
- verify that a real dependency cycle is still detected.

## Priority 2: performance and scalability

The items in this section are performance risks, not confirmed regressions. Measure representative workloads before changing the implementation.

### 4. Reduce full-state cloning inside loops

Every loop implementation clones the complete `ProgramState` before executing the loop body. `ProgramState` contains a `HashMap<u64, BigInt>`, so each iteration can clone the map and every stored large integer.

The clone provides straightforward and correct rollback semantics, but it can dominate execution time and allocation for programs with many registers, large values, or deeply nested loops.

Benchmark these alternatives:

- a mutation journal containing only registers changed during an iteration;
- copy-on-write state;
- a dense vector for commonly used low-numbered registers plus sparse overflow storage;
- specialized fast paths for the common one-register loop;
- retaining full clones only for indirect or otherwise complex loops.

Any replacement must preserve the current behavior where a non-decreasing final iteration is rolled back while its consumed step count remains recorded.

### 5. Keep sparse memory sparse

Unset registers read as zero, but `ProgramState::set_u64` stores explicit zero values in the `HashMap`. Removing a register when its new value is zero would reduce map size, cloning cost, and verbose-state noise.

Before changing this behavior, verify that no diagnostics or external API relies on distinguishing an untouched register from an explicitly zeroed register. The language execution semantics appear not to require that distinction.

### 6. Measure dynamic dispatch and `BigInt` cloning

Executable programs are represented as `Vec<Box<dyn Node>>`. This is extensible and easy to understand, but it introduces one allocation per node and dynamic dispatch per instruction. Operand reads also clone `BigInt` values frequently.

Do not replace this representation without evidence. First establish benchmarks for:

- short programs evaluated for many sequence indexes;
- nested loops;
- direct versus indirect memory access;
- cold versus warm cache behavior;
- small and very large integer values.

If dispatch is significant, consider a closed `ExecutableNode` enum. If `BigInt` cloning dominates, optimize operand access and ownership independently of the node representation.

## Priority 2: architecture and maintainability

### 7. Split unrelated subsystems into separate crates

The workspace currently has a compact execution core and a very large CLI crate. Approximate Rust source sizes at the time of review were:

- `loda-rust-core`: 10,075 lines;
- `loda-rust-cli`: 106,527 lines;
- `loda-rust-web`: 413 lines.

Much of the CLI size belongs to the optional ARC subsystem, with mining, analytics, post-mining, and command coordination also sharing the crate.

Recommended workspace boundaries:

- `loda-rust-core`: parser, executable representation, evaluator, and dependency model;
- `loda-rust-miner`: mutation, candidate evaluation, analytics, and mining support;
- `loda-rust-arc`: ARC image model and solvers;
- `loda-rust-cli`: argument handling and coordination only;
- `loda-rust-web`: WASM interface.

This split would reduce conceptual scope, make feature ownership clearer, and allow each subsystem to have an independent dependency and test matrix. It should be done incrementally without changing behavior.

### 8. Strengthen the CI feature matrix

The default workspace test run does not compile or execute the optional ARC tests. Add explicit CI jobs for:

- `cargo test --workspace`;
- `cargo test --workspace --all-features`;
- the WASM target build;
- formatting and Clippy checks;
- release-mode compilation.

If the full ARC suite is too expensive for every change, run a smaller feature-enabled smoke test for pull requests and the complete suite on the main development branch or on a schedule.

### 9. Make library failures consistently typed

The parser and dependency manager expose useful domain-specific error enums, while parts of execution return `anyhow::Result` and some invariant failures panic.

For the reusable core library:

- reserve panics for genuinely impossible internal invariants;
- return typed errors for invalid input, missing links, resource exhaustion, and dependency failures;
- add context at the CLI and WASM boundaries;
- avoid making consumers parse formatted `anyhow` chains to identify a failure category.

This is an API-quality improvement rather than an urgent rewrite. It can be introduced one subsystem at a time.

### 10. Align release metadata

The checked-out history contains 2025 releases, while `loda-rust-cli/Cargo.toml` reports version `2024.12.12`. Decide whether the Cargo package version is intended to follow release dates. If so, automate updating and validating it during release preparation.

## Suggested implementation order

1. Fix the XOR loop-range defect and add boundary tests.
2. Make dependency traversal cleanup unconditional and add retry tests.
3. Define the cache-policy contract, enforce the step limit on cache hits, and add cross-policy tests.
4. Add an all-features CI job so optional code is continuously validated.
5. Establish performance benchmarks before altering runtime representation.
6. Optimize state snapshots and sparse memory based on benchmark results.
7. Split ARC and mining functionality into separate crates incrementally.
8. Improve typed errors and release automation as ongoing maintenance.

## Completion criteria

The high-priority remediation is complete when:

- constant loop ranges use the intended power-of-two boundary;
- boundary tests cover the accepted and rejected values;
- cache hits cannot bypass the active execution limits;
- dependency loads can be retried safely after any failure;
- default and all-feature test suites pass in CI;
- the behavior of cache policy and loop-range limits is documented.

Performance and architecture work should be considered complete only when benchmarks demonstrate an improvement and compatibility tests show unchanged LODA program results.
