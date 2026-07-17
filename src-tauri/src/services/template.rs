use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub fn extract_variables(template: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*(?::[^}]*)?\}\}").unwrap();
    let mut set = BTreeSet::new();
    for cap in re.captures_iter(template) {
        set.insert(cap[1].to_string());
    }
    set.into_iter().collect()
}

pub fn render_template(template: &str, variables: &Map<String, Value>) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*(?::([^}]*))?\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let name = &caps[1];
        if let Some(val) = variables.get(name) {
            match val {
                Value::String(s) => s.clone(),
                other => other.to_string().trim_matches('"').to_string(),
            }
        } else if let Some(default) = caps.get(2) {
            default.as_str().trim().to_string()
        } else {
            format!("{{{{{name}}}}}")
        }
    })
    .to_string()
}

pub fn cartesian_product(variable_values: &Map<String, Value>) -> Vec<Map<String, Value>> {
    let mut keys: Vec<String> = variable_values.keys().cloned().collect();
    keys.sort();
    if keys.is_empty() {
        return vec![Map::new()];
    }

    let mut lists: Vec<Vec<Value>> = Vec::new();
    for key in &keys {
        let vals = match variable_values.get(key) {
            Some(Value::Array(arr)) => arr.clone(),
            Some(v) => vec![v.clone()],
            None => vec![Value::Null],
        };
        lists.push(if vals.is_empty() {
            vec![Value::Null]
        } else {
            vals
        });
    }

    let mut result = vec![Map::new()];
    for (i, key) in keys.iter().enumerate() {
        let mut next = Vec::new();
        for existing in &result {
            for val in &lists[i] {
                let mut map = existing.clone();
                map.insert(key.clone(), val.clone());
                next.push(map);
            }
        }
        result = next;
    }
    result
}

pub fn parse_csv_cases(csv: &str) -> Result<(Vec<String>, Vec<Map<String, Value>>), String> {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| "CSV is empty".to_string())?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    if header.is_empty() {
        return Err("CSV header is empty".into());
    }
    let mut cases = Vec::new();
    for line in lines {
        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let mut map = Map::new();
        for (i, key) in header.iter().enumerate() {
            map.insert(
                key.clone(),
                Value::String(cols.get(i).unwrap_or(&"").to_string()),
            );
        }
        cases.push(map);
    }
    Ok((header, cases))
}
