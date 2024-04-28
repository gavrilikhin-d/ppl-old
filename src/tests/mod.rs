use crate::e2es;

e2es! {
    array,
    candidate_not_viable,
    clone,
    common_functions,
    constraints,
    constraints_in_constructor,
    consume_greater,
    deps,
    deref_member_ref,
    destructor,
    empty_block,
    empty_constructor,
    escaped_id,
    generics,
    import_all,
    integer,
    integer_not_eq_rational,
    invalid_indentation,
    memory,
    missing_fields,
    monomorphize,
    monomorphize_predeclared,
    multifile,
    multiple_errors,
    multiple_initialization,
    non_class_constructor,
    plus_assign,
    predeclare_function,
    predeclare_vars,
    rational,
    reference_mut,
    reference_to_none,
    references,
    specify_variable_ty,
    star,
    string,
    supertraits,
    traits,
    type_as_value,
    trait_with_ref,
    type_of,
    wrong_initializer_type
}

#[test]
fn ppl() {
    use std::path::Path;

    use insta::assert_snapshot;
    use tempdir::TempDir;

    // Compile-time check that file exists
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/ppl/src/lib.ppl"));

    let temp_dir = TempDir::new("ppl").unwrap();
    let tmp = temp_dir.path();
    let name = "ppl";
    let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/ppl"));

    let res = crate::e2e::internal::compile(&tmp, dir);
    if let Err(err) = res {
        assert_snapshot!("ppl.error", err);
        return;
    }

    let hir = crate::e2e::internal::hir(&tmp, name, &dir);
    assert_snapshot!("ppl.hir", hir);

    let ir = crate::e2e::internal::ir(&tmp, name, &dir);
    assert_snapshot!("ppl.ir", ir);
}
