// devela_macros::tests

use super::compile_eval;

use alloc::string::ToString;

#[test]
fn test_compile_eval() {
    /* unary */

    // bare
    assert_eq!(compile_eval("true".into()), true);
    assert_eq!(compile_eval("false".into()), false);
    assert_eq!(compile_eval("NOTTRUE".into()), false);
    assert_eq!(compile_eval("".into()), false);

    // not()
    assert_eq!(compile_eval("not(true)".into()), false);
    assert_eq!(compile_eval("not(NOTTRUE)".into()), true);
    assert_eq!(compile_eval("not()".into()), true);

    /* binary */

    // eq()
    assert_eq!(compile_eval("eq(true, true)".into()), true);
    assert_eq!(compile_eval("eq(false, false)".into()), true);
    assert_eq!(compile_eval("eq(X, X)".into()), true); // both are `false`
    assert_eq!(compile_eval("eq(AA, BB)".into()), true); // both are `false`
    assert_eq!(compile_eval("eq(true, false)".into()), false);

    // ne()
    assert_eq!(compile_eval("ne(true, true)".into()), false);
    assert_eq!(compile_eval("ne(false, false)".into()), false);
    assert_eq!(compile_eval("ne(X, X)".into()), false); // both are `false`
    assert_eq!(compile_eval("ne(AA, BB)".into()), false); // both are `false`
    assert_eq!(compile_eval("ne(true, false)".into()), true);

    // xor()
    assert_eq!(compile_eval("xor(true, false)".into()), true);
    assert_eq!(compile_eval("xor(false, true)".into()), true);
    assert_eq!(compile_eval("xor(true, true)".into()), false);

    /* non-binary */

    // any()
    assert_eq!(compile_eval("any(true, true, true)".to_string()), true);
    assert_eq!(compile_eval("any(false, true, true)".to_string()), true);
    assert_eq!(compile_eval("any(false, false, false)".to_string()), false);

    // all()
    assert_eq!(
        compile_eval("all(true, true, true, true)".to_string()),
        true
    );
    assert_eq!(
        compile_eval("all(true, true, true, false)".to_string()),
        false
    );

    // none()
    assert_eq!(compile_eval("none()".to_string()), true);
    assert_eq!(compile_eval("none(true)".to_string()), false);
    assert_eq!(compile_eval("none(false)".to_string()), false);
    assert_eq!(compile_eval("none(something)".to_string()), false);

    // some()
    assert_eq!(compile_eval("some(true)".to_string()), true);
    assert_eq!(compile_eval("some(false)".to_string()), true);
    assert_eq!(compile_eval("some(something)".to_string()), true);
    assert_eq!(compile_eval("some()".to_string()), false);

    // diff()
    assert_eq!(compile_eval("diff(ABC, DEF, ABC)".to_string()), true);
    assert_eq!(compile_eval("diff(ABC, ABC, ABC)".to_string()), false);

    // same()
    assert_eq!(compile_eval("same(ABC, ABC, ABC)".to_string()), true);
    assert_eq!(compile_eval("same(ABC, DEF, ABC)".to_string()), false);

    // xany()
    assert_eq!(compile_eval("xany(true, true, false)".into()), true);
    assert_eq!(compile_eval("xany(true, true, true)".into()), false);

    // xodd()
    assert_eq!(compile_eval("xodd(true)".into()), true);
    assert_eq!(compile_eval("xodd(true, false, false)".into()), true);
    assert_eq!(compile_eval("xodd(true, true, false)".into()), false);
    assert_eq!(compile_eval("xodd(true, true, true)".into()), true);
    assert_eq!(compile_eval("xodd(true, true, true, true)".into()), false);

    // xodd()
    assert_eq!(compile_eval("xone(true, false, false, false)".into()), true);
    assert_eq!(compile_eval("xone(true, true, false, false)".into()), false);
    assert_eq!(compile_eval("xone(true, true, true, false)".into()), false);
    assert_eq!(compile_eval("xone(true, true, true, true)".into()), false);
}
