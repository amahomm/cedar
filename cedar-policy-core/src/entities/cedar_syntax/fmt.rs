/*
 * Copyright Cedar Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Pretty-printer for converting internal Entities to Cedar entity data syntax

use std::collections::BTreeMap;
use std::fmt::Write;

use smol_str::SmolStr;

use crate::ast::{Entity, EntityUID, Literal, PartialValue, Value, ValueKind};
use crate::entities::Entities;

/// Error type for formatting
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// A formatting error occurred
    #[error("formatting error: {0}")]
    Fmt(#[from] std::fmt::Error),
    /// A residual (unknown) value was encountered
    #[error("cannot format entity with residual (unknown) value")]
    Residual,
}

/// Format entities as Cedar entity data syntax text
pub fn format_entities(entities: &Entities) -> Result<String, FormatError> {
    let mut output = String::new();

    // Group entities by namespace prefix
    let grouped = group_by_namespace(entities);

    let mut first = true;
    for (namespace, ents) in &grouped {
        if !first {
            writeln!(output)?;
        }
        first = false;

        match namespace {
            Some(ns) => {
                writeln!(output, "namespace {ns} {{")?;
                for entity in ents {
                    write_instance(&mut output, entity, "    ", Some(ns))?;
                }
                write!(output, "}}")?;
            }
            None => {
                for (i, entity) in ents.iter().enumerate() {
                    if i > 0 {
                        writeln!(output)?;
                    }
                    write_instance(&mut output, entity, "", None)?;
                }
            }
        }
    }

    Ok(output.trim_end().to_string())
}

/// Group entities by their namespace (type prefix before last segment)
fn group_by_namespace<'a>(entities: &'a Entities) -> BTreeMap<Option<String>, Vec<&'a Entity>> {
    let mut map: BTreeMap<Option<String>, Vec<&Entity>> = BTreeMap::new();
    for entity in entities.iter() {
        let type_name = entity.uid().entity_type().to_string();
        let namespace = extract_namespace(&type_name);
        map.entry(namespace).or_default().push(entity);
    }
    // Sort entities within each group by UID for deterministic output
    for ents in map.values_mut() {
        ents.sort_by(|a, b| a.uid().to_string().cmp(&b.uid().to_string()));
    }
    map
}

/// Extract namespace from "Namespace::Type" → Some("Namespace"), "Type" → None
fn extract_namespace(type_name: &str) -> Option<String> {
    match type_name.rfind("::") {
        Some(pos) => Some(type_name[..pos].to_string()),
        None => None,
    }
}

/// Extract the basename (last segment) from a type name
fn extract_basename(type_name: &str) -> &str {
    match type_name.rfind("::") {
        Some(pos) => &type_name[pos + 2..],
        None => type_name,
    }
}

/// Write a single instance declaration
fn write_instance(
    out: &mut String,
    entity: &Entity,
    indent: &str,
    current_namespace: Option<&str>,
) -> Result<(), FormatError> {
    let uid = entity.uid();
    let type_str = format_type_relative(uid, current_namespace);
    let id_str = escape_eid(uid.eid().as_ref());

    write!(out, "{indent}instance {type_str}::\"{id_str}\"")?;

    // Parents (direct only)
    let parents: Vec<&EntityUID> = entity.parents().collect();
    if !parents.is_empty() {
        if parents.len() == 1 {
            write!(
                out,
                " in {}",
                format_entity_ref(parents[0], current_namespace)
            )?;
        } else {
            let refs: Vec<_> = parents
                .iter()
                .map(|p| format_entity_ref(p, current_namespace))
                .collect();
            write!(out, " in [{}]", refs.join(", "))?;
        }
    }

    // Attributes
    let attr_count = entity.attrs_len();
    if attr_count > 0 {
        writeln!(out, " = {{")?;
        let mut attr_keys: Vec<&SmolStr> = entity.keys().collect();
        attr_keys.sort();
        for (i, key) in attr_keys.iter().enumerate() {
            let val = entity.get(key).unwrap();
            let val_str = format_partial_value(val, current_namespace)?;
            write!(out, "{indent}    {}: {val_str}", format_key(key))?;
            if i < attr_count - 1 {
                writeln!(out, ",")?;
            } else {
                writeln!(out)?;
            }
        }
        write!(out, "{indent}}}")?;
    }

    // Tags
    let tag_count = entity.tags_len();
    if tag_count > 0 {
        writeln!(out, " tags {{")?;
        let mut tag_keys: Vec<&SmolStr> = entity.tag_keys().collect();
        tag_keys.sort();
        for (i, key) in tag_keys.iter().enumerate() {
            let val = entity.get_tag(key).unwrap();
            let val_str = format_partial_value(val, current_namespace)?;
            write!(out, "{indent}    {}: {val_str}", format_key(key))?;
            if i < tag_count - 1 {
                writeln!(out, ",")?;
            } else {
                writeln!(out)?;
            }
        }
        write!(out, "{indent}}}")?;
    }

    writeln!(out, ";")?;
    Ok(())
}

/// Format a PartialValue into Cedar syntax
fn format_partial_value(
    pv: &PartialValue,
    ns: Option<&str>,
) -> Result<String, FormatError> {
    match pv {
        PartialValue::Value(v) => format_value(v, ns),
        PartialValue::Residual(_) => Err(FormatError::Residual),
    }
}

/// Format a Value into Cedar syntax
fn format_value(value: &Value, ns: Option<&str>) -> Result<String, FormatError> {
    match &value.value {
        ValueKind::Lit(lit) => Ok(format_literal(lit, ns)),
        ValueKind::Set(set) => {
            let items: Vec<String> = set
                .iter()
                .map(|v| format_value(v, ns))
                .collect::<Result<_, _>>()?;
            Ok(format!("[{}]", items.join(", ")))
        }
        ValueKind::Record(record) => format_record(record, ns),
        ValueKind::ExtensionValue(ext) => {
            let args: Vec<String> = ext
                .args
                .iter()
                .map(|a| {
                    // RestrictedExpr args: try to extract the value
                    // For simple cases (string literal), format directly
                    format_restricted_expr_arg(a, ns)
                })
                .collect::<Result<_, _>>()?;
            Ok(format!("{}({})", ext.func, args.join(", ")))
        }
    }
}

/// Format a restricted expression argument (used in extension function call reconstruction)
fn format_restricted_expr_arg(
    expr: &crate::ast::RestrictedExpr,
    ns: Option<&str>,
) -> Result<String, FormatError> {
    // Try to evaluate as a simple value by pattern-matching on the expression kind
    use crate::ast::ExprKind;
    match expr.as_borrowed().expr_kind() {
        ExprKind::Lit(lit) => Ok(format_literal(lit, ns)),
        ExprKind::Set(exprs) => {
            let items: Vec<String> = exprs
                .iter()
                .map(|e| {
                    let re = crate::ast::RestrictedExpr::new_unchecked(e.clone());
                    format_restricted_expr_arg(&re, ns)
                })
                .collect::<Result<_, _>>()?;
            Ok(format!("[{}]", items.join(", ")))
        }
        ExprKind::Record(pairs) => {
            if pairs.is_empty() {
                return Ok("{}".to_string());
            }
            let items: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    let re = crate::ast::RestrictedExpr::new_unchecked(v.clone());
                    let val_str = format_restricted_expr_arg(&re, ns)?;
                    Ok(format!("{}: {val_str}", format_key(k)))
                })
                .collect::<Result<_, FormatError>>()?;
            Ok(format!("{{ {} }}", items.join(", ")))
        }
        ExprKind::ExtensionFunctionApp { fn_name, args } => {
            let arg_strs: Vec<String> = args
                .iter()
                .map(|a| {
                    let re = crate::ast::RestrictedExpr::new_unchecked(a.clone());
                    format_restricted_expr_arg(&re, ns)
                })
                .collect::<Result<_, _>>()?;
            Ok(format!("{fn_name}({})", arg_strs.join(", ")))
        }
        _ => Ok(format!("{expr}")), // fallback
    }
}

/// Format a Literal into Cedar syntax
fn format_literal(lit: &Literal, ns: Option<&str>) -> String {
    match lit {
        Literal::Bool(b) => b.to_string(),
        Literal::Long(n) => n.to_string(),
        Literal::String(s) => format!("\"{}\"", escape_string(s)),
        Literal::EntityUID(uid) => format_entity_ref(uid, ns),
    }
}

/// Format a record into Cedar syntax
fn format_record(
    record: &BTreeMap<SmolStr, Value>,
    ns: Option<&str>,
) -> Result<String, FormatError> {
    if record.is_empty() {
        return Ok("{}".to_string());
    }
    let items: Vec<String> = record
        .iter()
        .map(|(k, v)| {
            let val_str = format_value(v, ns)?;
            Ok(format!("{}: {val_str}", format_key(k)))
        })
        .collect::<Result<_, FormatError>>()?;
    Ok(format!("{{ {} }}", items.join(", ")))
}

/// Format an entity reference, stripping namespace prefix if in the current namespace
fn format_entity_ref(uid: &EntityUID, current_namespace: Option<&str>) -> String {
    let type_str = format_type_relative(uid, current_namespace);
    let id_str = escape_eid(uid.eid().as_ref());
    format!("{type_str}::\"{id_str}\"")
}

/// Format a type name relative to the current namespace
fn format_type_relative(uid: &EntityUID, current_namespace: Option<&str>) -> String {
    let full_type = uid.entity_type().to_string();
    match current_namespace {
        Some(ns) => {
            // If the type starts with the current namespace + "::", strip it
            let prefix = format!("{ns}::");
            if full_type.starts_with(&prefix) {
                extract_basename(&full_type).to_string()
            } else {
                full_type
            }
        }
        None => full_type,
    }
}

/// Format a record key — quote it if it contains special characters or is a reserved word
fn format_key(key: &str) -> String {
    if crate::ast::is_normalized_ident(key) {
        key.to_string()
    } else {
        format!("\"{}\"", escape_string(key))
    }
}

/// Escape special characters in a string for Cedar string literals
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\0' => result.push_str("\\0"),
            c if c.is_control() => {
                // Use unicode escape for other control characters
                for unit in c.encode_utf16(&mut [0u16; 2]) {
                    write!(result, "\\u{{{:x}}}", unit).unwrap();
                }
            }
            c => result.push(c),
        }
    }
    result
}

/// Escape entity ID for use in a string literal
fn escape_eid(eid: &str) -> String {
    escape_string(eid)
}
