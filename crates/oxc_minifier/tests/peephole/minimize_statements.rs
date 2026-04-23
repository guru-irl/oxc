use crate::{CompressOptions, default_options, test, test_options, test_same_options};

#[test]
fn test_for_variable_declaration() {
    test(
        "function _() { var x; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x, i = 0; i < 10; i++) console.log(i) }",
    );
    test(
        "function _() { var x = 1; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x = 1, i = 0; i < 10; i++) console.log(i) }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = function () { return console.log(j), 1 }; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = function () { return console.log(j), 1 }, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = j; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = j, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
}

#[test]
fn test_for_continue_in_for() {
    test("for( a of b ){ if(c) { continue; } d() }", "for ( a of b ) c || d();");
    test("for( a in b ){ if(c) { continue; } d() }", "for ( a in b ) c || d();");
    test("for( ; ; ){ if(c) { continue; } d() }", "for ( ; ; ) c || d();");

    test("for( a of b ){ c(); continue; }", "for ( a of b ) c();");
    test("for( a in b ){ c(); continue; }", "for ( a in b ) c();");
    test("for( ; ; ){ c(); continue; }", "for ( ; ; ) c();");
}

#[test]
fn test_for_in_block_scoped_no_inline() {
    // Should NOT inline when for-in uses `let` or `const` because it can cause variable shadowing
    // https://github.com/oxc-project/oxc/issues/18650
    // The inlined expression might reference a variable with the same name as the for-in variable,
    // causing it to incorrectly reference the shadowed for-in variable instead of the outer variable.
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (let name in foo) { console.log(name); } console.log(name); }",
        "{ var name = 'name1'; let foo = { foo: 1 }; name = 'name2'; for (let name in foo) console.log(name); console.log(name); }",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (const name in foo) { console.log(name); } console.log(name); }",
        "{ var name = 'name1'; let foo = { foo: 1 }; name = 'name2'; for (let name in foo) console.log(name); console.log(name); }",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (var name in foo) { console.log(name); } console.log(name); }",
        "var name = 'name1'; for (var name in name = 'name2', { foo: 1 }) console.log(name); console.log(name);",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (name in foo) { console.log(name); } console.log(name); }",
        "var name = 'name1'; for (name in name = 'name2', { foo: 1 }) console.log(name); console.log(name);",
    );
}

#[test]
fn test_sequences_max_length_caps_long_chains() {
    // Without a cap, six side-effect expression statements collapse into a
    // single sequence expression.
    test(
        "function _() { a(); b(); c(); d(); e(); f(); }",
        "function _() { a(), b(), c(), d(), e(), f(); }",
    );

    // With `sequences_max_length: Some(3)` the chain stops growing at three
    // expressions and the next statement starts a fresh sequence.
    let opts = |n: u32| -> CompressOptions {
        CompressOptions { sequences_max_length: Some(n), ..default_options() }
    };
    test_options(
        "function _() { a(); b(); c(); d(); e(); f(); }",
        "function _() { a(), b(), c(); d(), e(), f(); }",
        &opts(3),
    );

    // Cap of 2: pairs only.
    test_options(
        "function _() { a(); b(); c(); d(); }",
        "function _() { a(), b(); c(), d(); }",
        &opts(2),
    );

    // Cap of 1: no joining at all. The output still has statements, but they
    // stay separate — and because the statements have side effects the
    // minifier cannot drop them, so the body is unchanged.
    test_same_options("function _() { a(); b(); c(); }", &opts(1));

    // Cap does not interfere with other sequence-related optimizations as
    // long as the length stays under the cap. The `return` absorption still
    // fires for a short chain.
    test_options(
        "function _() { a(); b(); return c; }",
        "function _() { return a(), b(), c; }",
        &opts(10),
    );

    // Input: a(); b(); c(); return d;    with cap=2.
    // The return absorbs `c()` because that's a 1+1=2 merge.
    // `a(); b()` then form their own 2-chain expression statement.
    test_options(
        "function _() { a(); b(); c(); return d; }",
        "function _() { a(), b(); return c(), d; }",
        &opts(2),
    );
}
