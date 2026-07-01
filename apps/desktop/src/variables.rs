use std::collections::{HashMap, HashSet};

/// Parse unique `{{name}}` placeholders from prompt instructions (order preserved).
pub fn parse_variable_placeholders(content: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut names = Vec::new();
    let mut i = 0;
    let bytes = content.as_bytes();
    while i + 4 <= bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = content[i + 2..].find("}}") {
                let name = content[i + 2..i + 2 + end].trim();
                if !name.is_empty() && seen.insert(name.to_string()) {
                    names.push(name.to_string());
                }
                i += end + 4;
                continue;
            }
        }
        i += 1;
    }
    names
}

/// Substitute `{{name}}` placeholders; validate keys match the parsed set.
pub fn validate_and_substitute(
    content: &str,
    variables: &HashMap<String, String>,
    allow_empty_variables: bool,
) -> Result<String, String> {
    let placeholders = parse_variable_placeholders(content);
    let placeholder_set: HashSet<_> = placeholders.iter().cloned().collect();
    let variable_keys: HashSet<_> = variables.keys().cloned().collect();

    if placeholder_set != variable_keys {
        return Err("variables do not match prompt placeholders".into());
    }

    for name in &placeholders {
        let value = variables.get(name).map(String::as_str).unwrap_or("");
        if value.is_empty() && !allow_empty_variables {
            return Err(format!("variable '{name}' is empty"));
        }
    }

    let mut out = content.to_string();
    for name in placeholders {
        let value = variables.get(&name).cloned().unwrap_or_default();
        let needle = format!("{{{{{name}}}}}");
        out = out.replace(&needle, &value);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unique_placeholders_in_order() {
        assert_eq!(
            parse_variable_placeholders("Hi {{name}} about {{topic}} and {{name}}"),
            vec![String::from("name"), String::from("topic")]
        );
    }

    #[test]
    fn substitutes_and_rejects_mismatch() {
        let mut vars = HashMap::new();
        vars.insert("topic".into(), "AI".into());
        vars.insert("extra".into(), "x".into());
        let err = validate_and_substitute("Write about {{topic}}", &vars, false).unwrap_err();
        assert!(err.contains("do not match"));

        let mut vars = HashMap::new();
        vars.insert("topic".into(), "AI".into());
        let out = validate_and_substitute("Write about {{topic}}", &vars, false).unwrap();
        assert_eq!(out, "Write about AI");
    }

    #[test]
    fn rejects_empty_without_allow() {
        let mut vars = HashMap::new();
        vars.insert("name".into(), "".into());
        let err = validate_and_substitute("Greet {{name}}", &vars, false).unwrap_err();
        assert!(err.contains("empty"));
    }

    #[test]
    fn allows_empty_when_flag_set() {
        let mut vars = HashMap::new();
        vars.insert("name".into(), "".into());
        let out = validate_and_substitute("Greet {{name}}", &vars, true).unwrap();
        assert_eq!(out, "Greet ");
    }
}
