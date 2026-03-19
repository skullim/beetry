#[test]
fn assert_unique_types_compile_behavior() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assert_unique_types/pass_primitive.rs");
    t.pass("tests/assert_unique_types/pass_equivalent_primitive_path.rs");
    t.pass("tests/assert_unique_types/pass_non_primitive.rs");
    t.pass("tests/assert_unique_types/pass_non_trivial.rs");
    t.compile_fail("tests/assert_unique_types/fail_duplicate.rs");
}
