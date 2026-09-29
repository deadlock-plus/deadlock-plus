use super::*;

const SAMPLE: &str = r#"<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:generic:version{7412167c-06e9-4698-aff2-e63eb59037e7} -->
// leading comment
{
	hero_inferno =
	{
		m_flHealth = 550 /* inline */
		m_flSpeed = 6.5
		m_bEnabled = true
		m_pNothing = null
		"quoted key" = "a value"
		m_Tags = [ "a", "b", ]
		m_Icon = resource:"panorama/images/hero.psd"
		m_Nested = { m_Deep = [ 1, -2, 3e2 ] }
		m_Desc = """line one
line "two" """
	}
}
"#;

fn obj<const N: usize>(pairs: [(&str, Value); N]) -> Value {
    Value::Object(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

#[test]
fn parses_sample_document() {
    let doc = parse(SAMPLE).unwrap();
    assert_eq!(doc.header.as_deref(), Some(GENERIC_HEADER));
    let hero = doc.root.get("hero_inferno").unwrap();
    assert_eq!(hero.get("m_flHealth"), Some(&Value::Int(550)));
    assert_eq!(hero.get("m_flSpeed"), Some(&Value::Float(6.5)));
    assert_eq!(hero.get("m_bEnabled"), Some(&Value::Bool(true)));
    assert_eq!(hero.get("m_pNothing"), Some(&Value::Null));
    assert_eq!(hero.get("quoted key"), Some(&Value::String("a value".into())));
    assert_eq!(hero.get("m_Tags"), Some(&Value::Array(vec![Value::String("a".into()), Value::String("b".into())])));
    assert_eq!(
        hero.get("m_Icon"),
        Some(&Value::Flagged("resource".into(), Box::new(Value::String("panorama/images/hero.psd".into()))))
    );
    assert_eq!(
        hero.get("m_Nested").and_then(|n| n.get("m_Deep")),
        Some(&Value::Array(vec![Value::Int(1), Value::Int(-2), Value::Float(300.0)]))
    );
    assert_eq!(hero.get("m_Desc"), Some(&Value::String("line one\nline \"two\" ".into())));
}

#[test]
fn keeps_key_order_and_duplicates() {
    let doc = parse("{ b = 1 a = 2 b = 3 }").unwrap();
    assert_eq!(doc.root, obj([("b", Value::Int(1)), ("a", Value::Int(2)), ("b", Value::Int(3))]));
    assert_eq!(doc.header, None);
}

#[test]
fn write_then_parse_round_trips() {
    let doc = parse(SAMPLE).unwrap();
    let text = write(&doc);
    assert_eq!(parse(&text).unwrap(), doc);
}

#[test]
fn writer_quotes_keys_that_need_it_and_emits_header() {
    let doc = Document {
        header: Some(GENERIC_HEADER.into()),
        root: obj([("plain_key", Value::Int(1)), ("needs quoting", Value::Bool(false)), ("", Value::Null)]),
    };
    let text = write(&doc);
    assert!(text.starts_with(GENERIC_HEADER));
    assert!(text.contains("plain_key = 1"));
    assert!(text.contains("\"needs quoting\" = false"));
    assert!(text.contains("\"\" = null"));
    assert_eq!(parse(&text).unwrap(), doc);
}

#[test]
fn writer_round_trips_awkward_scalars() {
    let doc = Document {
        header: None,
        root: obj([
            ("f", Value::Float(1.0)),
            ("big", Value::Float(1e21)),
            ("neg", Value::Int(-7)),
            ("empty_arr", Value::Array(vec![])),
            ("empty_obj", Value::Object(vec![])),
            ("quote", Value::String("say \"hi\"".into())),
            ("multi", Value::String("a\nb".into())),
            ("backslash", Value::String("C:\\x".into())),
        ]),
    };
    assert_eq!(parse(&write(&doc)).unwrap(), doc);
}

#[test]
fn get_on_non_object_is_none() {
    assert_eq!(Value::Int(1).get("x"), None);
    assert_eq!(Value::Object(vec![]).get("x"), None);
}

#[test]
fn rejects_malformed_input() {
    assert!(parse("{ a = 1").is_err());
    assert!(parse("{ a = }").is_err());
    assert!(parse("{ a = \"open }").is_err());
    assert!(parse("{ a = bareword }").is_err());
    assert!(parse("{ a = 1 } trailing").is_err());
    assert!(parse("").is_err());
}
