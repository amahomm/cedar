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

//! Convert entity data AST to internal Entities representation

use std::collections::HashSet;

use smol_str::SmolStr;

use crate::ast::{Eid, Entity, EntityType, EntityUID, Name, RestrictedExpr};
use crate::extensions::Extensions;
use crate::from_normalized_str::FromNormalizedStr;
use crate::parser::Node;

use super::ast::*;
use super::err::{ConversionError, ConversionErrors};

/// Convert parsed entity data AST into a Vec of internal [`Entity`] values.
///
/// The resulting entities can then be passed to [`crate::entities::Entities::from_entities`]
/// for transitive closure computation and schema validation.
pub fn cedar_entities_to_entities(
    ast: EntityDataAst,
    extensions: &Extensions<'_>,
) -> Result<Vec<Entity>, ConversionErrors> {
    let mut entities = Vec::new();
    let mut errors = Vec::new();

    for annotated_ns in ast {
        let namespace_prefix: Option<Vec<SmolStr>> = annotated_ns.data.name.as_ref().map(|p| {
            p.node.iter().map(|s| s.node.clone()).collect()
        });

        for annotated_inst in annotated_ns.data.instances {
            match convert_instance(&annotated_inst.data.node, &namespace_prefix, extensions) {
                Ok(entity) => entities.push(entity),
                Err(e) => errors.push(e),
            }
        }
    }

    if errors.is_empty() {
        Ok(entities)
    } else {
        Err(ConversionErrors::new(errors))
    }
}

/// Convert a single EntityInstance AST node to an Entity
fn convert_instance(
    inst: &EntityInstance,
    namespace_prefix: &Option<Vec<SmolStr>>,
    extensions: &Extensions<'_>,
) -> Result<Entity, ConversionError> {
    // 1. Resolve UID
    let uid = resolve_entity_ref(&inst.entity_ref.node, namespace_prefix)?;

    // 2. Convert parents (direct parents only — TC computed later)
    let parents: HashSet<EntityUID> = inst
        .parents
        .iter()
        .map(|p| resolve_entity_ref(&p.node, namespace_prefix))
        .collect::<Result<_, _>>()?;

    // 3. Convert attributes
    let attrs: Vec<(SmolStr, RestrictedExpr)> = match &inst.attrs {
        Some(record) => record
            .node
            .iter()
            .map(|(k, v)| {
                let val = convert_value(&v.node, namespace_prefix)?;
                Ok((k.node.clone(), val))
            })
            .collect::<Result<_, ConversionError>>()?,
        None => Vec::new(),
    };

    // 4. Convert tags
    let tags: Vec<(SmolStr, RestrictedExpr)> = match &inst.tags {
        Some(record) => record
            .node
            .iter()
            .map(|(k, v)| {
                let val = convert_value(&v.node, namespace_prefix)?;
                Ok((k.node.clone(), val))
            })
            .collect::<Result<_, ConversionError>>()?,
        None => Vec::new(),
    };

    // 5. Build Entity (indirect_ancestors is empty — TC will compute it)
    Entity::new(uid, attrs, HashSet::new(), parents, tags, extensions).map_err(|e| {
        ConversionError::ExtensionError(e.to_string())
    })
}

/// Resolve an EntityReference (Type::"id") to an EntityUID,
/// applying namespace prefix for unqualified types.
fn resolve_entity_ref(
    eref: &EntityReference,
    namespace_prefix: &Option<Vec<SmolStr>>,
) -> Result<EntityUID, ConversionError> {
    let entity_type = resolve_entity_type(&eref.type_path, namespace_prefix)?;
    Ok(EntityUID::from_components(
        entity_type,
        Eid::new(eref.id.clone()),
        None,
    ))
}

/// Resolve an entity type path within a namespace context.
/// - Single-segment path (e.g., "User") → prepend namespace → "PhotoApp::User"
/// - Multi-segment path (e.g., "OtherApp::User") → use as-is (already qualified)
fn resolve_entity_type(
    type_path: &[Node<SmolStr>],
    namespace_prefix: &Option<Vec<SmolStr>>,
) -> Result<EntityType, ConversionError> {
    let full_path: Vec<&str> = if type_path.len() == 1 {
        // Unqualified — prepend namespace
        match namespace_prefix {
            Some(ns) => ns
                .iter()
                .map(|s| s.as_str())
                .chain(std::iter::once(type_path[0].node.as_str()))
                .collect(),
            None => vec![type_path[0].node.as_str()],
        }
    } else {
        // Already qualified — use as-is
        type_path.iter().map(|s| s.node.as_str()).collect()
    };

    // Join segments with "::" to form the type name string
    let type_name = full_path.join("::");

    // Construct EntityType from the name
    let name = Name::from_normalized_str(&type_name).map_err(|_| {
        ConversionError::UnresolvedType {
            name: type_name.clone(),
        }
    })?;
    Ok(EntityType::from(name))
}

/// Convert an EntityValue AST node to a RestrictedExpr
fn convert_value(
    value: &EntityValue,
    namespace_prefix: &Option<Vec<SmolStr>>,
) -> Result<RestrictedExpr, ConversionError> {
    match value {
        EntityValue::Long(n) => Ok(RestrictedExpr::val(*n)),
        EntityValue::String(s) => Ok(RestrictedExpr::val(s.clone())),
        EntityValue::Bool(b) => Ok(RestrictedExpr::val(*b)),
        EntityValue::EntityRef(eref) => {
            let uid = resolve_entity_ref(&eref.node, namespace_prefix)?;
            Ok(RestrictedExpr::val(uid))
        }
        EntityValue::Set(items) => {
            let exprs: Vec<RestrictedExpr> = items
                .iter()
                .map(|v| convert_value(&v.node, namespace_prefix))
                .collect::<Result<_, _>>()?;
            Ok(RestrictedExpr::set(exprs))
        }
        EntityValue::Record(entries) => {
            let pairs: Vec<(SmolStr, RestrictedExpr)> = entries
                .iter()
                .map(|(k, v)| {
                    let val = convert_value(&v.node, namespace_prefix)?;
                    Ok((k.node.clone(), val))
                })
                .collect::<Result<_, ConversionError>>()?;
            RestrictedExpr::record(pairs).map_err(|e| ConversionError::DuplicateRecordKey {
                key: e.to_string(),
            })
        }
        EntityValue::ExtensionCall { fn_name, args } => {
            // Resolve function name — extension functions are unqualified names
            let name_str = fn_name
                .iter()
                .map(|s| s.node.as_str())
                .collect::<Vec<_>>()
                .join("::");
            let name = Name::from_normalized_str(&name_str).map_err(|_| {
                ConversionError::UnknownExtensionFunction {
                    name: name_str.clone(),
                }
            })?;

            // Convert arguments
            let arg_exprs: Vec<RestrictedExpr> = args
                .iter()
                .map(|a| convert_value(&a.node, namespace_prefix))
                .collect::<Result<_, _>>()?;

            Ok(RestrictedExpr::call_extension_fn(name, arg_exprs))
        }
    }
}
