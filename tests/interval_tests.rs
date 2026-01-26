use cp_sat::builder::{CpModelBuilder, LinearExpr};
use cp_sat::proto::CpSolverStatus;

#[test]
fn test_interval_var() {
    let mut model = CpModelBuilder::default();
    let start = model.new_int_var([(0, 10)]);
    let size = model.new_int_var([(2, 5)]);
    let end = model.new_int_var([(0, 15)]);
    let _interval = model.new_interval_var(start, size, end);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // Verify start + size == end
    let s = start.solution_value(&response);
    let sz = size.solution_value(&response);
    let e = end.solution_value(&response);
    assert_eq!(s + sz, e);
}

#[test]
fn test_fixed_size_interval_var() {
    let mut model = CpModelBuilder::default();
    let start = model.new_int_var([(0, 10)]);
    let _interval = model.new_fixed_size_interval_var(start, 2);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);
}

#[test]
fn test_optional_interval_var() {
    let mut model = CpModelBuilder::default();
    let start = model.new_int_var([(0, 10)]);
    let size = model.new_int_var([(2, 5)]);
    let end = model.new_int_var([(0, 15)]);
    let is_present = model.new_bool_var();
    let _interval = model.new_optional_interval_var(start, size, end, is_present);

    // Force is_present to be true
    model.add_and([is_present]);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // When present, start + size == end
    let s = start.solution_value(&response);
    let sz = size.solution_value(&response);
    let e = end.solution_value(&response);
    assert_eq!(s + sz, e);
}

#[test]
fn test_optional_fixed_size_interval_var() {
    let mut model = CpModelBuilder::default();
    let start = model.new_int_var([(0, 10)]);
    let is_present = model.new_bool_var();
    let _interval = model.new_optional_fixed_size_interval_var(start, 3, is_present);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);
}

#[test]
fn test_cumulative() {
    let mut model = CpModelBuilder::default();
    let s1 = model.new_int_var([(0, 10)]);
    let s2 = model.new_int_var([(0, 10)]);
    let i1 = model.new_fixed_size_interval_var(s1, 3);
    let i2 = model.new_fixed_size_interval_var(s2, 3);

    // Capacity 1 means intervals cannot overlap (like no_overlap)
    model.add_cumulative([i1, i2], [1, 1], 1);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // With capacity 1, intervals must not overlap
    let v1 = s1.solution_value(&response);
    let v2 = s2.solution_value(&response);
    assert!(v1 + 3 <= v2 || v2 + 3 <= v1);
}

#[test]
fn test_cumulative_with_higher_capacity() {
    let mut model = CpModelBuilder::default();
    let s1 = model.new_int_var([(0, 10)]);
    let s2 = model.new_int_var([(0, 10)]);
    let s3 = model.new_int_var([(0, 10)]);
    let i1 = model.new_fixed_size_interval_var(s1, 5);
    let i2 = model.new_fixed_size_interval_var(s2, 5);
    let i3 = model.new_fixed_size_interval_var(s3, 5);

    // Capacity 2 means at most 2 intervals can overlap
    model.add_cumulative([i1, i2, i3], [1, 1, 1], 2);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);
}

#[test]
fn test_no_overlap() {
    let mut model = CpModelBuilder::default();
    let s1 = model.new_int_var([(0, 10)]);
    let s2 = model.new_int_var([(0, 10)]);
    let s3 = model.new_int_var([(0, 10)]);
    let i1 = model.new_fixed_size_interval_var(s1, 3);
    let i2 = model.new_fixed_size_interval_var(s2, 3);
    let i3 = model.new_fixed_size_interval_var(s3, 3);

    model.add_no_overlap([i1, i2, i3]);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // All intervals must be disjoint
    let v1 = s1.solution_value(&response);
    let v2 = s2.solution_value(&response);
    let v3 = s3.solution_value(&response);

    // Check all pairs are disjoint
    assert!(v1 + 3 <= v2 || v2 + 3 <= v1);
    assert!(v1 + 3 <= v3 || v3 + 3 <= v1);
    assert!(v2 + 3 <= v3 || v3 + 3 <= v2);
}

#[test]
fn test_scheduling_problem() {
    // 3 tasks, 2 machines (capacity 2), minimize makespan
    let mut model = CpModelBuilder::default();
    let horizon = 100;

    let starts: Vec<_> = (0..3)
        .map(|i| model.new_int_var_with_name([(0, horizon)], format!("start_{}", i)))
        .collect();

    let intervals: Vec<_> = starts
        .iter()
        .map(|&s| model.new_fixed_size_interval_var(s, 5))
        .collect();

    model.add_cumulative(intervals.clone(), [1, 1, 1], 2);

    let makespan = model.new_int_var([(0, horizon)]);
    for &s in &starts {
        model.add_le(LinearExpr::from(s) + 5, makespan);
    }
    model.minimize(makespan);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // With capacity 2, two tasks can overlap, one alone: makespan = 10
    assert_eq!(makespan.solution_value(&response), 10);
}

#[test]
fn test_scheduling_with_no_overlap() {
    // 3 tasks on single machine (no overlap), minimize makespan
    let mut model = CpModelBuilder::default();
    let horizon = 100;

    let starts: Vec<_> = (0..3)
        .map(|i| model.new_int_var_with_name([(0, horizon)], format!("start_{}", i)))
        .collect();

    let intervals: Vec<_> = starts
        .iter()
        .map(|&s| model.new_fixed_size_interval_var(s, 5))
        .collect();

    model.add_no_overlap(intervals.clone());

    let makespan = model.new_int_var([(0, horizon)]);
    for &s in &starts {
        model.add_le(LinearExpr::from(s) + 5, makespan);
    }
    model.minimize(makespan);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // With no overlap, all 3 tasks must be sequential: makespan = 15
    assert_eq!(makespan.solution_value(&response), 15);
}

#[test]
fn test_optional_intervals_with_cumulative() {
    let mut model = CpModelBuilder::default();

    let s1 = model.new_int_var([(0, 10)]);
    let s2 = model.new_int_var([(0, 10)]);
    let present1 = model.new_bool_var();
    let present2 = model.new_bool_var();

    let i1 = model.new_optional_fixed_size_interval_var(s1, 5, present1);
    let i2 = model.new_optional_fixed_size_interval_var(s2, 5, present2);

    // Capacity 1 - only one interval can be present at any time
    model.add_cumulative([i1, i2], [1, 1], 1);

    // Force both intervals to be present
    model.add_and([present1, present2]);

    let response = model.solve();
    assert_eq!(response.status(), CpSolverStatus::Optimal);

    // Both present, so they must not overlap
    let v1 = s1.solution_value(&response);
    let v2 = s2.solution_value(&response);
    assert!(v1 + 5 <= v2 || v2 + 5 <= v1);
}
