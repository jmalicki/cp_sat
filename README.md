# Google CP-SAT solver Rust bindings [![](https://img.shields.io/crates/v/cp_sat.svg)](https://crates.io/crates/cp_sat) [![](https://docs.rs/cp_sat/badge.svg)](https://docs.rs/cp_sat)

Rust bindings to the Google CP-SAT constraint programming solver.

To use this library, you need a C++ compiler and an installation of
google or-tools library files.

If you're using a recent version of or-tools, you'll also need libprotobuf (older versions used to link it statically). Invoke Cargo using `RUSTFLAGS='-Clink-arg=-lprotobuf' cargo <command>`.

The environment variable `ORTOOLS_PREFIX` is used to find include
files and library files. If not setted, `/opt/ortools` will be added
to the search path (classical search path will also be used).

## Features

This crate provides a builder API for constructing CP-SAT models, including:

- **Integer and Boolean variables** - `new_int_var`, `new_bool_var`
- **Linear constraints** - `add_eq`, `add_le`, `add_ge`, `add_lt`, `add_gt`, `add_ne`
- **Boolean constraints** - `add_or`, `add_and`, `add_xor`, `add_at_most_one`, `add_exactly_one`
- **Global constraints** - `add_all_different`, `add_min_eq`, `add_max_eq`
- **Interval variables** - `new_interval_var`, `new_fixed_size_interval_var`, `new_optional_interval_var`, `new_optional_fixed_size_interval_var`
- **Scheduling constraints** - `add_cumulative`, `add_no_overlap`

## Scheduling Example

```rust
use cp_sat::builder::{CpModelBuilder, LinearExpr};
use cp_sat::proto::CpSolverStatus;

fn main() {
    let mut model = CpModelBuilder::default();
    let horizon = 100;

    // Create 3 tasks with duration 5
    let starts: Vec<_> = (0..3)
        .map(|i| model.new_int_var_with_name([(0, horizon)], format!("start_{}", i)))
        .collect();

    let intervals: Vec<_> = starts
        .iter()
        .map(|&s| model.new_fixed_size_interval_var(s, 5))
        .collect();

    // At most 2 tasks can run in parallel (capacity 2)
    model.add_cumulative(intervals, [1, 1, 1], 2);

    // Minimize makespan
    let makespan = model.new_int_var([(0, horizon)]);
    for &s in &starts {
        model.add_le(LinearExpr::from(s) + 5, makespan);
    }
    model.minimize(makespan);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);
    println!("Makespan: {}", makespan.solution_value(&response));
}
```
