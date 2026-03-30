#[test]
fn gsi_key_conditions_are_type_safe() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gsi_key_conditions/*.rs");
}
