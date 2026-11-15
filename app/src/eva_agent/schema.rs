//! A small JSON-schema validator for tool arguments (own code, no crate).
//!
//! It covers exactly the keywords the tool registry uses: `type` (object,
//! string, integer, number, boolean, array), `properties`, `required`,
//! `additionalProperties: false`, `enum`, `minimum`, `maximum`, `maxLength`,
//! `maxItems`, `items`, `default` and `pattern` (anchored patterns made of
//! literals and `[a-z]` classes, which is all the registry needs).
//!
//! Models sometimes send numbers as strings (`"ml": "250"`); a string that is
//! a whole number is accepted for an `integer` field and converted. Nothing
//! else is coerced. Unknown keys are rejected, so a model cannot smuggle
//! extra parameters into a tool.

use serde_json::{Map, Value};

/// Validate `args` against `schema`. Returns the (lightly coerced) arguments
/// or a plain sentence saying what is wrong.
pub fn validate(args: &Value, schema: &Value) -> Result<Value, String> {
    check(args, schema, "arguments")
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn check(v: &Value, schema: &Value, at: &str) -> Result<Value, String> {
    let ty = schema.get("type").and_then(|t| t.as_str()).unwrap_or("any");
    let mut v = v.clone();
    // coercion: "250" → 250 for integer fields
    if ty == "integer" {
        if let Some(s) = v.as_str() {
            if let Ok(n) = s.trim().parse::<i64>() {
                v = Value::from(n);
            }
        }
        if let Some(f) = v.as_f64().filter(|_| !v.is_i64() && !v.is_u64()) {
            if f.fract() == 0.0 && f.abs() < 9e15 {
                v = Value::from(f as i64);
            }
        }
    }
    let got = type_name(&v);
    let type_ok = match ty {
        "any" => true,
        "number" => got == "integer" || got == "number",
        t => t == got,
    };
    if !type_ok {
        return Err(format!("{at} must be {ty}, got {got}"));
    }
    if let Some(choices) = schema.get("enum").and_then(|e| e.as_array()) {
        if !choices.contains(&v) {
            let list: Vec<String> = choices.iter().map(|c| c.as_str().map(String::from).unwrap_or_else(|| c.to_string())).collect();
            return Err(format!("{at} must be one of {}", list.join(", ")));
        }
    }
    match &mut v {
        Value::Number(n) => {
            let x = n.as_f64().unwrap_or(0.0);
            if let Some(min) = schema.get("minimum").and_then(|m| m.as_f64()) {
                if x < min {
                    return Err(format!("{at} must be at least {min}"));
                }
            }
            if let Some(max) = schema.get("maximum").and_then(|m| m.as_f64()) {
                if x > max {
                    return Err(format!("{at} must be at most {max}"));
                }
            }
        }
        Value::String(s) => {
            if let Some(max) = schema.get("maxLength").and_then(|m| m.as_u64()) {
                if s.chars().count() as u64 > max {
                    return Err(format!("{at} is longer than {max} characters"));
                }
            }
            if let Some(p) = schema.get("pattern").and_then(|p| p.as_str()) {
                if !pattern_matches(p, s) {
                    return Err(format!("{at} has the wrong format"));
                }
            }
        }
        Value::Array(items) => {
            if let Some(max) = schema.get("maxItems").and_then(|m| m.as_u64()) {
                if items.len() as u64 > max {
                    return Err(format!("{at} has more than {max} items"));
                }
            }
            if let Some(item_schema) = schema.get("items") {
                for (i, it) in items.iter_mut().enumerate() {
                    *it = check(it, item_schema, &format!("{at}[{i}]"))?;
                }
            }
        }
        Value::Object(obj) => {
            let props = schema.get("properties").and_then(|p| p.as_object()).cloned().unwrap_or_default();
            if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
                if let Some(extra) = obj.keys().find(|k| !props.contains_key(*k)) {
                    return Err(format!("unknown parameter '{}'", extra.chars().take(40).collect::<String>()));
                }
            }
            if let Some(req) = schema.get("required").and_then(|r| r.as_array()) {
                for r in req.iter().filter_map(|r| r.as_str()) {
                    if obj.get(r).is_none_or(|x| x.is_null()) {
                        return Err(format!("'{r}' is required"));
                    }
                }
            }
            let mut out = Map::new();
            for (k, val) in obj.iter() {
                match props.get(k) {
                    Some(ps) => {
                        out.insert(k.clone(), check(val, ps, k)?);
                    }
                    None => {
                        out.insert(k.clone(), val.clone());
                    }
                }
            }
            *obj = out;
        }
        _ => {}
    }
    Ok(v)
}

/// Anchored patterns built from literal characters and `[x-y]` / `[abc]`
/// classes, e.g. `^[0-2][0-9]:[0-5][0-9]$`. Anything fancier is rejected
/// (the registry never uses it), so a pattern can never be silently ignored.
pub fn pattern_matches(pattern: &str, s: &str) -> bool {
    let Some(body) = pattern.strip_prefix('^').and_then(|p| p.strip_suffix('$')) else { return false };
    let mut atoms: Vec<Vec<(char, char)>> = Vec::new();
    let mut it = body.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '[' => {
                let mut ranges = Vec::new();
                loop {
                    let Some(a) = it.next() else { return false };
                    if a == ']' {
                        break;
                    }
                    if it.peek() == Some(&'-') {
                        it.next();
                        let Some(b) = it.next() else { return false };
                        ranges.push((a, b));
                    } else {
                        ranges.push((a, a));
                    }
                }
                atoms.push(ranges);
            }
            '\\' => {
                let Some(a) = it.next() else { return false };
                atoms.push(vec![(a, a)]);
            }
            '*' | '+' | '?' | '{' | '}' | '(' | ')' | '|' | '.' => return false,
            other => atoms.push(vec![(other, other)]),
        }
    }
    let chars: Vec<char> = s.chars().collect();
    chars.len() == atoms.len() && chars.iter().zip(&atoms).all(|(c, ranges)| ranges.iter().any(|(a, b)| c >= a && c <= b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn water() -> Value {
        json!({"type":"object","properties":{"ml":{"type":"integer","minimum":50,"maximum":2000}},"required":["ml"],"additionalProperties":false})
    }

    #[test]
    fn accepts_valid_and_coerces_numeric_strings() {
        assert_eq!(validate(&json!({"ml": 250}), &water()).unwrap(), json!({"ml": 250}));
        assert_eq!(validate(&json!({"ml": "300"}), &water()).unwrap(), json!({"ml": 300}));
        assert_eq!(validate(&json!({"ml": 250.0}), &water()).unwrap(), json!({"ml": 250}));
    }

    #[test]
    fn rejects_bad_types_ranges_missing_and_extra_keys() {
        assert!(validate(&json!({"ml": 10}), &water()).unwrap_err().contains("at least"));
        assert!(validate(&json!({"ml": 99999}), &water()).unwrap_err().contains("at most"));
        assert!(validate(&json!({}), &water()).unwrap_err().contains("required"));
        assert!(validate(&json!({"ml": true}), &water()).unwrap_err().contains("integer"));
        assert!(validate(&json!({"ml": 250, "dose_mg": 5}), &water()).unwrap_err().contains("unknown parameter"));
        assert!(validate(&json!([1]), &water()).is_err());
    }

    #[test]
    fn enums_lengths_items_and_patterns() {
        let s = json!({"type":"object","properties":{
            "format":{"type":"string","enum":["csv","json"]},
            "text":{"type":"string","maxLength":5},
            "labels":{"type":"array","items":{"type":"string"},"maxItems":2},
            "at":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"}},"additionalProperties":false});
        assert!(validate(&json!({"format":"pdf"}), &s).is_err());
        assert!(validate(&json!({"text":"too long"}), &s).is_err());
        assert!(validate(&json!({"labels":["a","b","c"]}), &s).is_err());
        assert!(validate(&json!({"labels":[1]}), &s).is_err());
        assert!(validate(&json!({"at":"14:30"}), &s).is_ok());
        assert!(validate(&json!({"at":"14:61"}), &s).is_err());
        assert!(validate(&json!({"at":"4:30"}), &s).is_err());
    }

    #[test]
    fn pattern_engine_refuses_what_it_cannot_check() {
        assert!(!pattern_matches("^a.*$", "abc"));
        assert!(!pattern_matches("no-anchors", "no-anchors"));
        assert!(pattern_matches("^v\\.1$", "v.1"));
    }
}
