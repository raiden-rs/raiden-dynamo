#[test]
fn table_key_conditions_are_type_safe() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/table_key_conditions/*.rs");
}
