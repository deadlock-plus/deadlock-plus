use super::*;

#[test]
fn strip_keeps_length_and_newlines() {
    let src = "a(); // note\nb(); /* multi\nline */ c();\r\n";
    let out = strip(src);
    assert_eq!(out.len(), src.len());
    assert_eq!(out.matches('\n').count(), src.matches('\n').count());
    assert!(out.contains("a();"));
    assert!(out.contains("c();"));
    assert!(!out.contains("note"));
    assert!(!out.contains("multi"));
}

#[test]
fn strip_blanks_string_contents_but_keeps_quotes() {
    let out = strip(r#"x = "a { b"; y = 'c } d'; z = `e ${f} g`;"#);
    assert!(!out.contains('{') && !out.contains('}'));
    assert!(out.contains("x = \""));
    assert!(out.contains("\";"));
}

#[test]
fn strip_handles_escaped_quotes() {
    let out = strip(r#"x = "a\"{b"; f();"#);
    assert!(!out.contains('{'));
    assert!(out.contains("f();"));
}

#[test]
fn strip_comment_markers_inside_strings_are_not_comments() {
    let out = strip(r#"u = "http://x"; g();"#);
    assert!(out.contains("g();"));
}

#[test]
fn strip_blanks_regex_literals() {
    let out = strip("if (/[{]+/.test(s)) { go(); }");
    assert_eq!(out.matches('{').count(), 1);
    assert!(out.contains("go();"));
}

#[test]
fn division_is_not_a_regex() {
    let out = strip("x = a / 2; y = b / 3; { z(); }");
    assert!(out.contains("y = b"));
    assert!(out.contains("z();"));
}

#[test]
fn strip_survives_unterminated_input() {
    strip("x = 'never closed");
    strip("/* never closed");
    strip("y = `never closed");
}

#[test]
fn strip_keeps_multibyte_text_valid() {
    let out = strip("a = 'héllo'; // ünï\nb();");
    assert!(out.contains("b();"));
}

fn names(src: &str) -> Vec<String> {
    functions(&strip(src)).into_iter().map(|f| f.name).collect()
}

#[test]
fn finds_declarations_assignments_and_arrows() {
    let src = "
        function alpha(a, b) { return 1; }
        var beta = function () { };
        const gamma = (x) => { };
        let delta = y => { };
        obj.epsilon = function named() { };
        var lambda = async (q) => { };
    ";
    assert_eq!(names(src), ["alpha", "beta", "gamma", "delta", "epsilon", "lambda"]);
}

#[test]
fn body_spans_matching_braces_and_ignores_braces_in_strings() {
    let src = "function f() { if (a) { x(\"}\"); } // }\n y(); }\nfunction g() { }";
    let stripped = strip(src);
    let fns = functions(&stripped);
    assert_eq!(fns.len(), 2);
    let body = &stripped[fns[0].body.clone()];
    assert!(body.starts_with('{') && body.ends_with('}'));
    assert!(body.contains("y();"));
    assert!(!body.contains("g()"));
}

#[test]
fn nested_functions_are_reported_with_nested_ranges() {
    let stripped = strip("function outer() { function inner() { } }");
    let fns = functions(&stripped);
    assert_eq!(fns.len(), 2);
    let outer = &fns[0];
    let inner = &fns[1];
    assert!(outer.body.start < inner.body.start && inner.body.end < outer.body.end);
}

#[test]
fn arrows_without_a_block_body_are_skipped() {
    assert!(names("const f = x => x + 1;").is_empty());
}

#[test]
fn unbalanced_body_is_dropped() {
    assert!(names("function f() { if (a) {").is_empty());
}

#[test]
fn line_of_counts_from_one() {
    let src = "a\nb\r\nc";
    assert_eq!(line_of(src, 0), 1);
    assert_eq!(line_of(src, 2), 2);
    assert_eq!(line_of(src, 5), 3);
}
