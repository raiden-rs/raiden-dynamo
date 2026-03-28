#[test]
fn omit_gsi_requires_optional_or_defaulted_fields() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/omit_gsi/*.rs");
}
