// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Type names resolution.

use crate::errors::{Error, Result};
use convert_case::{Case, Casing};
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
use gprimitives::*;
use scale_info::{
    PortableType, Type, TypeDef, TypeDefArray, TypeDefPrimitive, TypeDefSequence, TypeDefTuple,
    TypeInfo, form::PortableForm,
};
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    result::Result as StdResult,
    sync::OnceLock,
};

pub(super) fn resolve<'a>(
    types: impl Iterator<Item = &'a PortableType>,
) -> Result<(BTreeMap<u32, String>, BTreeMap<u32, String>)> {
    let types = types
        .map(|t| (t.id, t))
        .collect::<BTreeMap<u32, &PortableType>>();
    let type_names = types.iter().try_fold(
        (
            BTreeMap::<u32, RcTypeName>::new(),
            HashMap::<(String, Vec<u32>), u32>::new(),
        ),
        |mut type_names, ty| {
            resolve_type_name(&types, *ty.0, &mut type_names.0, &mut type_names.1)
                .map(|_| type_names)
        },
    );
    type_names.map(|type_names| {
        let concrete_names = type_names
            .0
            .iter()
            .map(|(id, name)| (*id, name.as_string(false, &type_names.1)))
            .collect();
        
        // Build generic names by analyzing type parameter patterns across multiple instantiations
        let generic_param_mappings = build_generic_param_mappings(&types);
        
        let mut generic_names = BTreeMap::new();
        for (id, name) in &type_names.0 {
            let type_info = types.get(id).map(|t| &t.ty);
            if let Some(type_info) = type_info {
                // Get the generic parameter mapping for this specific type
                let own_param_map = generic_param_mappings.get(id).cloned().unwrap_or_default();
                // For top-level types, there's no parent context
                let parent_param_map = HashMap::new();
                
                let generic_name = name.as_generic_string(&type_names.1, &own_param_map, &parent_param_map, Some(*id));
                generic_names.insert(*id, generic_name);
            }
        }
        
        (concrete_names, generic_names)
    })
}

/// Analyzes all types in the registry to build mappings of which type IDs correspond to generic parameters.
/// Returns a map from type_id -> (concrete_type_id -> generic_name like "T1", "T2", etc.)
fn build_generic_param_mappings(
    types: &BTreeMap<u32, &PortableType>,
) -> HashMap<u32, HashMap<u32, String>> {
    let mut mappings = HashMap::new();
    
    // Group types by their base name and type param count
    let mut type_families: HashMap<(String, usize), Vec<u32>> = HashMap::new();
    
    for (type_id, portable_type) in types {
        let type_info = &portable_type.ty;
        if !type_info.type_params.is_empty() {
            // Get the base name (without type parameters)
            let base_name = type_info.path.segments.join("::");
            let param_count = type_info.type_params.len();
            
            type_families
                .entry((base_name, param_count))
                .or_default()
                .push(*type_id);
        }
    }
    
    // For each family of related generic types, analyze their fields to determine which are generic
    for ((base_name, param_count), family_members) in &type_families {
        if family_members.len() < 2 {
            // With only one instantiation, we can't detect generic patterns by comparison
            // Fall back to simple mapping: type_params[i] -> T{i+1}
            if let Some(&type_id) = family_members.first() {
                if let Some(portable_type) = types.get(&type_id) {
                    let mut param_map = HashMap::new();
                    for (idx, param) in portable_type.ty.type_params.iter().enumerate() {
                        if let Some(param_ty) = param.ty {
                            param_map.insert(param_ty.id, format!("T{}", idx + 1));
                        }
                    }
                    mappings.insert(type_id, param_map);
                }
            }
            continue;
        }
        
        // Analyze field patterns across multiple instantiations
        let field_analysis = analyze_field_patterns(types, family_members);
        
        // For each member of the family, build its generic parameter mapping
        for &type_id in family_members {
            if let Some(portable_type) = types.get(&type_id) {
                let param_map = build_param_map_from_analysis(
                    &portable_type.ty,
                    &field_analysis,
                    types,
                );
                eprintln!("Built param_map with {} entries for type {} (type_id {})", param_map.len(), base_name, type_id);
                for (tid, name) in &param_map {
                    eprintln!("  {} -> {}", tid, name);
                }
                mappings.insert(type_id, param_map);
            }
        }
        
        // Additional pass: handle tuple type parameters
        // NOTE: Disabled for now - tuples are handled by recursive rendering with parent context
        // add_tuple_element_mappings(types, family_members, &mut mappings);
    }
    
    // Second pass: Propagate param_maps to nested types
    // For example, if MultiLevelReuse has param_map {51→T1, 73→T2},
    // and it has a field of type ReusableGenericStruct<Vec<u64>>,
    // then ReusableGenericStruct should also inherit {51→T1, 73→T2}
    // so that when it renders Vec<u64>, it can replace u64 with T1
    propagate_param_maps_to_nested_types(types, &mut mappings);
    
    mappings
}

/// Analyzes tuple type parameters across family members to determine which tuple elements are generic
fn add_tuple_element_mappings(
    types: &BTreeMap<u32, &PortableType>,
    family_members: &[u32],
    mappings: &mut HashMap<u32, HashMap<u32, String>>,
) {
    // For each family member, check if any of its type parameters are tuples
    for &member_id in family_members {
        if let Some(member_type) = types.get(&member_id) {
            // Check each type parameter
            for (param_idx, param) in member_type.ty.type_params.iter().enumerate() {
                if let Some(param_ty) = param.ty {
                    // Check if this parameter is a tuple
                    if let Some(tuple_type) = types.get(&param_ty.id) {
                        if let TypeDef::Tuple(tuple_def) = &tuple_type.ty.type_def {
                            eprintln!("  Found tuple parameter at index {} for type {}: tuple type_id {}", 
                                param_idx, member_id, param_ty.id);
                            
                            // Collect all tuple type_ids for this parameter position across all family members
                            let mut tuple_type_ids = Vec::new();
                            for &other_member_id in family_members {
                                if let Some(other_type) = types.get(&other_member_id) {
                                    if let Some(other_param) = other_type.ty.type_params.get(param_idx) {
                                        if let Some(other_param_ty) = other_param.ty {
                                            tuple_type_ids.push(other_param_ty.id);
                                            eprintln!("    Member {} has tuple type_id {} at param {}", 
                                                other_member_id, other_param_ty.id, param_idx);
                                        }
                                    }
                                }
                            }
                            
                            eprintln!("    Analyzing {} tuple instances for param position {}", 
                                tuple_type_ids.len(), param_idx);
                            
                            // Analyze which tuple element positions vary
                            let position_generics = analyze_tuple_elements_across_instances(
                                &tuple_type_ids,
                                types,
                                param_idx,
                            );
                            
                            // Add mappings for tuple elements to this member's param_map
                            if let Some(member_param_map) = mappings.get_mut(&member_id) {
                                for (tid, generic_name) in position_generics {
                                    eprintln!("    Adding tuple element mapping: {} -> {}", tid, generic_name);
                                    member_param_map.insert(tid, generic_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Analyzes tuple elements across multiple tuple instances to determine which positions are generic
fn analyze_tuple_elements_across_instances(
    tuple_type_ids: &[u32],
    types: &BTreeMap<u32, &PortableType>,
    param_idx: usize,
) -> HashMap<u32, String> {
    let mut element_mappings = HashMap::new();
    
    // Collect all tuple instances
    let tuple_instances: Vec<_> = tuple_type_ids.iter()
        .filter_map(|&tid| {
            types.get(&tid).and_then(|pt| {
                if let TypeDef::Tuple(tuple_def) = &pt.ty.type_def {
                    Some(tuple_def)
                } else {
                    None
                }
            })
        })
        .collect();
    
    if tuple_instances.is_empty() {
        return element_mappings;
    }
    
    // All tuples should have the same number of elements
    let element_count = tuple_instances[0].fields.len();
    
    // For each position in the tuple, check if the type varies across instances
    for pos in 0..element_count {
        let type_ids_at_pos: Vec<u32> = tuple_instances.iter()
            .filter_map(|tuple_def| tuple_def.fields.get(pos).map(|f| f.id))
            .collect();
        
        let unique_types: std::collections::HashSet<_> = type_ids_at_pos.iter().copied().collect();
        
        if unique_types.len() > 1 {
            // This position varies across instances - it's generic
            // Map each unique type_id at this position to the same generic name
            let generic_name = format!("T{}", param_idx + 1);
            for &type_id in &unique_types {
                element_mappings.insert(type_id, generic_name.clone());
            }
            eprintln!("    Tuple position {} varies ({} unique types) -> {}", pos, unique_types.len(), generic_name);
        } else {
            eprintln!("    Tuple position {} is concrete (1 unique type)", pos);
        }
    }
    
    element_mappings
}

/// Propagates param_maps from parent types to nested types used in their fields
fn propagate_param_maps_to_nested_types(
    types: &BTreeMap<u32, &PortableType>,
    mappings: &mut HashMap<u32, HashMap<u32, String>>,
) {
    eprintln!("Propagating param_maps to nested types...");
    
    // First, collect all (parent, nested) relationships
    let mut all_relationships: Vec<(u32, u32)> = Vec::new();
    
    for (&parent_id, parent_param_map) in mappings.iter() {
        if let Some(portable_type) = types.get(&parent_id) {
            collect_all_nested_type_ids(&portable_type.ty.type_def, types, &mut |field_type_id| {
                if !parent_param_map.contains_key(&field_type_id) && field_type_id != parent_id {
                    all_relationships.push((parent_id, field_type_id));
                }
            });
        }
    }
    
    // Propagate param_maps from parents to nested types
    // For single-parent: always propagate
    // For multi-parent: only propagate mappings that don't conflict
    eprintln!("Propagating param_maps to nested types...");
    
    let mut nested_to_parents: HashMap<u32, Vec<u32>> = HashMap::new();
    for (parent_id, nested_id) in &all_relationships {
        nested_to_parents.entry(*nested_id).or_default().push(*parent_id);
    }
    
    for (nested_id, parent_ids) in nested_to_parents {
        if parent_ids.len() == 1 {
            // Single parent - safe to propagate everything
            let parent_id = parent_ids[0];
            if let Some(parent_param_map) = mappings.get(&parent_id).cloned() {
                let nested_param_map = mappings.entry(nested_id).or_default();
                let original_len = nested_param_map.len();
                
                for (type_id, generic_name) in parent_param_map {
                    // Only insert if not already present - don't override nested type's own mappings
                    nested_param_map.entry(type_id).or_insert(generic_name);
                }
                
                if nested_param_map.len() > original_len {
                    eprintln!("  Propagating from {} to {}: added {} new mappings",
                        parent_id, nested_id, nested_param_map.len() - original_len);
                }
            }
        } else {
            // Multiple parents - only propagate non-conflicting mappings
            // Collect all mappings from all parents
            let parent_param_maps: Vec<HashMap<u32, String>> = parent_ids.iter()
                .filter_map(|&pid| mappings.get(&pid).cloned())
                .collect();
            
            if !parent_param_maps.is_empty() {
                // Find all type_ids that appear in any parent's param_map
                let mut all_type_ids = std::collections::HashSet::new();
                for pm in &parent_param_maps {
                    all_type_ids.extend(pm.keys());
                }
                
                let nested_param_map = mappings.entry(nested_id).or_default();
                let mut added = 0;
                
                for &type_id in &all_type_ids {
                    // Check if this type_id appears in nested type's own params
                    let is_nested_own_param = if let Some(nested_type) = types.get(&nested_id) {
                        nested_type.ty.type_params.iter()
                            .any(|p| p.ty.map(|t| t.id) == Some(type_id))
                    } else {
                        false
                    };
                    
                    // For nested type's own params, use majority vote:
                    // - Count how many parents map this param to a generic name
                    // - If majority (> 50%) map it, use the most common mapping
                    // - If majority leave it concrete (no mapping), clear the mapping
                    if is_nested_own_param {
                        let parents_with_mapping = parent_param_maps.iter()
                            .filter(|pm| pm.contains_key(&type_id))
                            .count();
                        
                        let total_parents = parent_param_maps.len();
                        let percentage = (parents_with_mapping as f64 / total_parents as f64) * 100.0;
                        
                        // Debug: show what each parent maps this to
                        if nested_id == 62 {
                            eprintln!("    DEBUG Type 62: Checking own param {}", type_id);
                            for (idx, pm) in parent_param_maps.iter().enumerate() {
                                eprintln!("      Parent {} (type_id={}): ", idx, parent_ids[idx]);
                                if let Some(mapping) = pm.get(&type_id) {
                                    eprintln!("        {} -> '{}'", type_id, mapping);
                                } else {
                                    eprintln!("        {} -> NOT MAPPED", type_id);
                                }
                                // Show all mappings this parent has
                                eprintln!("        All mappings: {:?}", pm);
                            }
                        }
                        
                        if parents_with_mapping == 0 {
                            // NO parent maps this - remove it from nested type's param_map
                            if nested_param_map.remove(&type_id).is_some() {
                                eprintln!("    Type {}: Removing own param {} - no parents map it (0/{})", 
                                    nested_id, type_id, total_parents);
                            }
                        } else if parents_with_mapping * 2 <= total_parents {
                            // Less than or equal to 50% map it - treat as concrete
                            if let Some(removed_value) = nested_param_map.remove(&type_id) {
                                eprintln!("    Type {}: Removing own param {} (was '{}') - minority of parents map it ({}/{} = {:.0}%)", 
                                    nested_id, type_id, removed_value, parents_with_mapping, total_parents, percentage);
                            }
                        } else {
                            // More than 50% map it - use the parent mapping if all parents agree
                            let current_value = nested_param_map.get(&type_id).map(|s| s.clone()).unwrap_or_else(|| "NONE".to_string());
                            
                            // Collect what parents map this to
                            let parent_names: Vec<&String> = parent_param_maps.iter()
                                .filter_map(|pm| pm.get(&type_id))
                                .collect();
                            
                            if !parent_names.is_empty() {
                                let all_agree = parent_names.iter().all(|n| **n == *parent_names[0]);
                                if all_agree {
                                    // All parents agree on the mapping - update to use it
                                    let new_value = parent_names[0].clone();
                                    if new_value != current_value {
                                        nested_param_map.insert(type_id, new_value.clone());
                                        eprintln!("    Type {}: Updating own param {} from '{}' to '{}' - all parents agree ({}/{} = {:.0}%)", 
                                            nested_id, type_id, current_value, new_value, parents_with_mapping, total_parents, percentage);
                                    } else {
                                        eprintln!("    Type {}: Keeping own param {} (currently '{}') - majority of parents map it ({}/{} = {:.0}%)", 
                                            nested_id, type_id, current_value, parents_with_mapping, total_parents, percentage);
                                    }
                                } else {
                                    eprintln!("    Type {}: Keeping own param {} (currently '{}') - parents disagree on mapping ({}/{} = {:.0}%)", 
                                        nested_id, type_id, current_value, parents_with_mapping, total_parents, percentage);
                                }
                            }
                        }
                        continue;
                    }
                    
                    // Collect all names parents use for this type_id
                    let parent_names: Vec<&String> = parent_param_maps.iter()
                        .filter_map(|pm| pm.get(&type_id))
                        .collect();
                    
                    // Only propagate if all parents that have this mapping agree on the name
                    if !parent_names.is_empty() {
                        let all_agree = parent_names.iter().all(|n| **n == *parent_names[0]);
                        
                        if all_agree {
                            // All parents agree - safe to propagate
                            if nested_param_map.insert(type_id, parent_names[0].clone()).is_none() {
                                added += 1;
                            }
                        }
                    }
                }
                
                if added > 0 {
                    eprintln!("  Propagating from {} parents to {}: added {} non-conflicting mappings",
                        parent_ids.len(), nested_id, added);
                }
            }
        }
    }
}

/// Collects all type IDs used in fields of a type, recursively
fn collect_all_nested_type_ids(
    type_def: &TypeDef<PortableForm>, 
    types: &BTreeMap<u32, &PortableType>,
    callback: &mut impl FnMut(u32)
) {
    match type_def {
        TypeDef::Composite(composite) => {
            for field in &composite.fields {
                callback(field.ty.id);
                // Also recurse into the field's type
                if let Some(field_type) = types.get(&field.ty.id) {
                    collect_nested_types_from_type(&field_type.ty.type_def, types, callback);
                }
            }
        }
        TypeDef::Variant(variant) => {
            for var in &variant.variants {
                for field in &var.fields {
                    callback(field.ty.id);
                    // Also recurse into the field's type
                    if let Some(field_type) = types.get(&field.ty.id) {
                        collect_nested_types_from_type(&field_type.ty.type_def, types, callback);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Helper to recursively collect types from nested structures
fn collect_nested_types_from_type(
    type_def: &TypeDef<PortableForm>,
    types: &BTreeMap<u32, &PortableType>,
    callback: &mut impl FnMut(u32)
) {
    match type_def {
        TypeDef::Sequence(seq) => {
            callback(seq.type_param.id);
            if let Some(inner_type) = types.get(&seq.type_param.id) {
                collect_nested_types_from_type(&inner_type.ty.type_def, types, callback);
            }
        }
        TypeDef::Array(arr) => {
            callback(arr.type_param.id);
            if let Some(inner_type) = types.get(&arr.type_param.id) {
                collect_nested_types_from_type(&inner_type.ty.type_def, types, callback);
            }
        }
        TypeDef::Tuple(tuple) => {
            for field in &tuple.fields {
                callback(field.id);
                if let Some(inner_type) = types.get(&field.id) {
                    collect_nested_types_from_type(&inner_type.ty.type_def, types, callback);
                }
            }
        }
        TypeDef::Composite(composite) => {
            // For composite types with type parameters, collect those
            for field in &composite.fields {
                callback(field.ty.id);
                if let Some(field_type) = types.get(&field.ty.id) {
                    collect_nested_types_from_type(&field_type.ty.type_def, types, callback);
                }
            }
        }
        _ => {}
    }
}

/// Collects all type IDs used in fields of a type
fn collect_field_type_ids(type_def: &TypeDef<PortableForm>, callback: &mut impl FnMut(u32)) {
    match type_def {
        TypeDef::Composite(composite) => {
            for field in &composite.fields {
                callback(field.ty.id);
            }
        }
        TypeDef::Variant(variant) => {
            for var in &variant.variants {
                for field in &var.fields {
                    callback(field.ty.id);
                }
            }
        }
        _ => {}
    }
}

/// Analyzes field patterns across multiple instantiations of the same generic type
/// Returns: field_name -> Vec<type_id> (the type IDs for this field across all instantiations)
fn analyze_field_patterns(
    types: &BTreeMap<u32, &PortableType>,
    family_members: &[u32],
) -> HashMap<String, Vec<u32>> {
    let mut field_patterns: HashMap<String, Vec<u32>> = HashMap::new();
    
    for &type_id in family_members {
        if let Some(portable_type) = types.get(&type_id) {
            match &portable_type.ty.type_def {
                TypeDef::Composite(composite) => {
                    for field in &composite.fields {
                        let field_name = field.name.as_ref().map(|s| s.to_string()).unwrap_or_default();
                        field_patterns.entry(field_name).or_default().push(field.ty.id);
                    }
                }
                TypeDef::Variant(variant) => {
                    for var in &variant.variants {
                        for field in &var.fields {
                            let field_name = format!("{}::{}", var.name, field.name.as_deref().unwrap_or(""));
                            field_patterns.entry(field_name).or_default().push(field.ty.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    eprintln!("Field patterns for family with {} members:", family_members.len());
    for (field_name, type_ids) in &field_patterns {
        let unique_ids: std::collections::HashSet<_> = type_ids.iter().collect();
        eprintln!("  {}: {} IDs, {} unique -> {}", 
            field_name, 
            type_ids.len(),
            unique_ids.len(),
            if unique_ids.len() > 1 { "GENERIC" } else { "CONCRETE" }
        );
    }
    
    field_patterns
}

/// Builds a parameter mapping for a specific type based on field analysis
fn build_param_map_from_analysis(
    type_info: &Type<PortableForm>,
    field_analysis: &HashMap<String, Vec<u32>>,
    types: &BTreeMap<u32, &PortableType>,
) -> HashMap<u32, String> {
    let mut param_map = HashMap::new();
    
    // Start with direct type parameters - but only for primitive/simple types
    // Don't add compound types (Vec, Tuple, Array) as they should be recursively rendered
    for (idx, param) in type_info.type_params.iter().enumerate() {
        if let Some(param_ty) = param.ty {
            // Only add to param_map if this is NOT a compound type that should be recursively rendered
            if let Some(portable_type) = types.get(&param_ty.id) {
                let should_add = match &portable_type.ty.type_def {
                    TypeDef::Primitive(_) => true,
                    TypeDef::Compact(_) => true,
                    TypeDef::BitSequence(_) => true,
                    // Don't add compound types - they should be recursively rendered
                    TypeDef::Sequence(_) => false,  // Vec
                    TypeDef::Array(_) => false,
                    TypeDef::Tuple(_) => {
                        // Special handling for tuples: we'll analyze their elements below
                        false
                    },
                    // Composite/Variant types: add only if they don't have their own type params
                    TypeDef::Composite(_) | TypeDef::Variant(_) => portable_type.ty.type_params.is_empty(),
                };
                
                if should_add {
                    param_map.insert(param_ty.id, format!("T{}", idx + 1));
                } else if matches!(&portable_type.ty.type_def, TypeDef::Tuple(_)) {
                    // For tuple parameters, we need to analyze which tuple elements are generic
                    // across all family members (handled in field_analysis)
                    eprintln!("  Tuple parameter at index {}, will analyze elements", idx);
                }
            }
        }
    }
    
    // Build a map of type parameter names to their indices
    let mut param_name_to_index: HashMap<String, usize> = HashMap::new();
    for (idx, param) in type_info.type_params.iter().enumerate() {
        if !param.name.is_empty() {
            param_name_to_index.insert(param.name.to_string(), idx);
        }
    }
    
    // Identify which fields vary across instantiations
    let varying_fields: std::collections::HashSet<String> = field_analysis.iter()
        .filter(|(_, type_ids)| {
            let unique_ids: std::collections::HashSet<_> = type_ids.iter().collect();
            unique_ids.len() > 1
        })
        .map(|(name, _)| name.clone())
        .collect();
    
    // Now check fields using their type_name to detect generic usage
    // BUT only propagate for fields that also vary across instantiations
    match &type_info.type_def {
        TypeDef::Composite(composite) => {
            for field in &composite.fields {
                let field_name = field.name.as_ref().map(|s| s.to_string()).unwrap_or_default();
                
                // Only process if this field varies across instantiations
                if !varying_fields.contains(&field_name) {
                    eprintln!("  Skipping field '{}' - does not vary", field_name);
                    continue;
                }
                
                if let Some(type_name_str) = &field.type_name {
                    // Check if the type_name contains any generic parameter names
                    analyze_type_name_for_generics(
                        type_name_str,
                        &param_name_to_index,
                        field.ty.id,
                        types,
                        &mut param_map,
                        &type_info.type_params,
                    );
                }
            }
        }
        TypeDef::Variant(variant) => {
            for var in &variant.variants {
                for field in &var.fields {
                    let field_name = format!("{}::{}", var.name, field.name.as_deref().unwrap_or(""));
                    
                    // Only process if this field varies across instantiations
                    if !varying_fields.contains(&field_name) {
                        eprintln!("  Skipping field '{}' - does not vary", field_name);
                        continue;
                    }
                    
                    if let Some(type_name_str) = &field.type_name {
                        analyze_type_name_for_generics(
                            type_name_str,
                            &param_name_to_index,
                            field.ty.id,
                            types,
                            &mut param_map,
                            &type_info.type_params,
                        );
                    }
                }
            }
        }
        _ => {}
    }
    
    eprintln!("Built param_map with {} entries for type {}", 
        param_map.len(),
        type_info.path.segments.join("::"));
    for (tid, name) in &param_map {
        eprintln!("  {} -> {}", tid, name);
    }
    
    param_map
}

/// Analyzes a type_name string (like "Vec<T>", "(T, bool)", etc.) to detect which parts are generic
fn analyze_type_name_for_generics(
    type_name_str: &str,
    param_name_to_index: &HashMap<String, usize>,
    field_type_id: u32,
    types: &BTreeMap<u32, &PortableType>,
    param_map: &mut HashMap<u32, String>,
    type_params: &[scale_info::TypeParameter<PortableForm>],
) {
    eprintln!("  Analyzing type_name: '{}'", type_name_str);
    
    // Check if this type_name contains any generic parameter names
    // We need to check if the parameter appears as a "word" (not part of another identifier)
    let contains_generics: Vec<(String, usize)> = param_name_to_index.iter()
        .filter(|(param_name, _)| {
            // Simple check: the param name should appear surrounded by non-alphanumeric/underscore chars
            // or at the start/end of the string
            let s = type_name_str;
            let p = param_name.as_str();
            
            // Helper to check if a character is a word boundary
            let is_boundary = |c: char| !c.is_alphanumeric() && c != '_';
            
            // Check if param appears at start/end or surrounded by boundaries
            if s == p {
                return true; // Exact match
            }
            
            // Find all occurrences of the param name
            for (idx, _) in s.match_indices(p) {
                // Check if before the match is a boundary (or start of string)
                let before_ok = idx == 0 || s.chars().nth(idx - 1).map(is_boundary).unwrap_or(true);
                
                // Check if after the match is a boundary (or end of string)
                let after_idx = idx + p.len();
                let after_ok = after_idx >= s.len() || s.chars().nth(after_idx).map(is_boundary).unwrap_or(true);
                
                if before_ok && after_ok {
                    return true;
                }
            }
            
            false
        })
        .map(|(n, i)| (n.clone(), *i))
        .collect();
    
    if contains_generics.is_empty() {
        eprintln!("    No generics found");
        return;
    }
    
    eprintln!("    Contains generics: {:?}", contains_generics);
    
    // If the type_name contains generic parameters, we need to map the concrete types
    // in the field's type to their generic equivalents
    propagate_generic_mapping(
        field_type_id,
        types,
        param_map,
        type_params,
    );
}

/// Analyzes tuple elements to determine which positions are generic vs concrete
/// by looking at type_name patterns across multiple instances
fn analyze_tuple_element_generics(
    tuple_type_ids: &[u32],
    types: &BTreeMap<u32, &PortableType>,
    type_params: &[scale_info::TypeParameter<PortableForm>],
) -> HashMap<usize, String> {
    // Returns: position -> generic_name (e.g., 0 -> "T1", 1 -> "T2")
    let mut position_generics = HashMap::new();
    
    if tuple_type_ids.is_empty() {
        return position_generics;
    }
    
    // Get all tuple instances
    let tuple_instances: Vec<_> = tuple_type_ids.iter()
        .filter_map(|&tid| {
            types.get(&tid).and_then(|pt| {
                if let TypeDef::Tuple(tuple_def) = &pt.ty.type_def {
                    Some((tid, tuple_def))
                } else {
                    None
                }
            })
        })
        .collect();
    
    if tuple_instances.is_empty() {
        return position_generics;
    }
    
    // All tuples should have the same number of elements
    let element_count = tuple_instances[0].1.fields.len();
    
    // For each position, check if the type varies across instances
    for pos in 0..element_count {
        let type_ids_at_pos: Vec<u32> = tuple_instances.iter()
            .filter_map(|(_, tuple_def)| tuple_def.fields.get(pos).map(|f| f.id))
            .collect();
        
        let unique_types: std::collections::HashSet<_> = type_ids_at_pos.iter().collect();
        
        if unique_types.len() > 1 {
            // This position varies - it's generic
            // Try to match it to one of the parent's type parameters
            for (idx, parent_param) in type_params.iter().enumerate() {
                if let Some(parent_param_ty) = parent_param.ty {
                    // Check if any of the concrete types at this position match this parent param
                    if type_ids_at_pos.contains(&parent_param_ty.id) {
                        position_generics.insert(pos, format!("T{}", idx + 1));
                        break;
                    }
                }
            }
        }
    }
    
    position_generics
}

/// Propagates generic parameter mappings through nested types (tuples, arrays, vecs, etc.)
fn propagate_generic_mapping(
    type_id: u32,
    types: &BTreeMap<u32, &PortableType>,
    param_map: &mut HashMap<u32, String>,
    type_params: &[scale_info::TypeParameter<PortableForm>],
) {
    eprintln!("  Propagating for type_id {}", type_id);
    
    // First check if this type_id matches any of the parent's type parameters directly
    for (idx, parent_param) in type_params.iter().enumerate() {
        if let Some(parent_param_ty) = parent_param.ty {
            if type_id == parent_param_ty.id {
                eprintln!("    Direct match with parent param {} (id={})", idx, parent_param_ty.id);
                
                // Check if this is a compound type that should be recursively rendered
                // rather than wholesale replaced
                if let Some(portable_type) = types.get(&type_id) {
                    // Always recurse into these types:
                    // - Vec, Array, Tuple (sequences/collections)
                    // - Composite/Variant with type parameters (like Option, ReusableGenericStruct, etc.)
                    let should_recurse = matches!(&portable_type.ty.type_def,
                        TypeDef::Sequence(_) | TypeDef::Array(_) | TypeDef::Tuple(_)
                    ) || (matches!(&portable_type.ty.type_def, TypeDef::Composite(_) | TypeDef::Variant(_))
                        && !portable_type.ty.type_params.is_empty());
                    
                    if should_recurse {
                        eprintln!("    Compound type - recursing instead of adding mapping");
                        // Don't add to param_map, let it recurse below
                        break;
                    }
                }
                
                if !param_map.contains_key(&type_id) {
                    param_map.insert(type_id, format!("T{}", idx + 1));
                    eprintln!("    MATCH! Adding {} -> T{}", type_id, idx + 1);
                }
                return; // Don't recurse further if we found a direct match
            }
        }
    }
    
    if let Some(portable_type) = types.get(&type_id) {
        eprintln!("    Type def: {:?}", std::mem::discriminant(&portable_type.ty.type_def));
        
        match &portable_type.ty.type_def {
            TypeDef::Tuple(tuple_def) => {
                eprintln!("    Tuple with {} elements", tuple_def.fields.len());
                // Recursively propagate for tuple elements
                for field in &tuple_def.fields {
                    propagate_generic_mapping(field.id, types, param_map, type_params);
                }
            }
            TypeDef::Sequence(seq_def) => {
                eprintln!("    Sequence");
                propagate_generic_mapping(seq_def.type_param.id, types, param_map, type_params);
            }
            TypeDef::Array(array_def) => {
                eprintln!("    Array");
                propagate_generic_mapping(array_def.type_param.id, types, param_map, type_params);
            }
            TypeDef::Composite(_) | TypeDef::Variant(_) => {
                eprintln!("    Composite/Variant with {} type_params", portable_type.ty.type_params.len());
                // Check if this type's parameters match any of the parent's type parameters
                for param in &portable_type.ty.type_params {
                    if let Some(param_ty) = param.ty {
                        eprintln!("      Checking param_ty.id={}", param_ty.id);
                        // Check if this matches one of the parent's type parameters
                        for (idx, parent_param) in type_params.iter().enumerate() {
                            if let Some(parent_param_ty) = parent_param.ty {
                                eprintln!("        Against parent param {}: id={}", idx, parent_param_ty.id);
                                if param_ty.id == parent_param_ty.id {
                                    eprintln!("        MATCH! Adding {} -> T{}", param_ty.id, idx + 1);
                                    // This nested type uses the same concrete type as the parent
                                    // So it should map to the same generic name
                                    if !param_map.contains_key(&param_ty.id) {
                                        param_map.insert(param_ty.id, format!("T{}", idx + 1));
                                    }
                                } else {
                                    // No direct match, but recurse into this type
                                    propagate_generic_mapping(param_ty.id, types, param_map, type_params);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                eprintln!("    Other type");
            }
        }
    }
}

fn resolve_type_name(
    types: &BTreeMap<u32, &PortableType>,
    type_id: u32,
    resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
    by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
) -> Result<RcTypeName> {
    if let Some(type_name) = resolved_type_names.get(&type_id) {
        return Ok(type_name.clone());
    }

    let type_info = types
        .get(&type_id)
        .map(|t| &t.ty)
        .ok_or_else(|| Error::TypeIdIsUnknown(type_id))?;

    let type_name: RcTypeName = match &type_info.type_def {
        TypeDef::Tuple(tuple_def) => Rc::new(TupleTypeName::new(
            types,
            tuple_def,
            resolved_type_names,
            by_path_type_names,
            type_id,
        )?),
        TypeDef::Sequence(vector_def) => Rc::new(VectorTypeName::new(
            types,
            vector_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Array(array_def) => Rc::new(ArrayTypeName::new(
            types,
            array_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Composite(_) => {
            if BTreeMapTypeName::is_btree_map_type(type_info) {
                Rc::new(BTreeMapTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else if actor_id::TypeNameImpl::is_type(type_info) {
                Rc::new(actor_id::TypeNameImpl::new())
            } else if message_id::TypeNameImpl::is_type(type_info) {
                Rc::new(message_id::TypeNameImpl::new())
            } else if code_id::TypeNameImpl::is_type(type_info) {
                Rc::new(code_id::TypeNameImpl::new())
            } else if h160::TypeNameImpl::is_type(type_info) {
                Rc::new(h160::TypeNameImpl::new())
            } else if h256::TypeNameImpl::is_type(type_info) {
                Rc::new(h256::TypeNameImpl::new())
            } else if u256::TypeNameImpl::is_type(type_info) {
                Rc::new(u256::TypeNameImpl::new())
            } else if nat8::TypeNameImpl::is_type(type_info) {
                Rc::new(nat8::TypeNameImpl::new())
            } else if nat16::TypeNameImpl::is_type(type_info) {
                Rc::new(nat16::TypeNameImpl::new())
            } else if nat32::TypeNameImpl::is_type(type_info) {
                Rc::new(nat32::TypeNameImpl::new())
            } else if nat64::TypeNameImpl::is_type(type_info) {
                Rc::new(nat64::TypeNameImpl::new())
            } else if nat128::TypeNameImpl::is_type(type_info) {
                Rc::new(nat128::TypeNameImpl::new())
            } else if nat256::TypeNameImpl::is_type(type_info) {
                Rc::new(nat256::TypeNameImpl::new())
            } else {
                Rc::new(ByPathTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Variant(_) => {
            if ResultTypeName::is_result_type(type_info) {
                Rc::new(ResultTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else if OptionTypeName::is_option_type(type_info) {
                Rc::new(OptionTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else {
                Rc::new(ByPathTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Primitive(primitive_def) => Rc::new(PrimitiveTypeName::new(primitive_def)?),
        _ => {
            return Err(Error::TypeIsUnsupported(format!("{type_info:?}")));
        }
    };

    resolved_type_names.insert(type_id, type_name.clone());
    Ok(type_name)
}

type RcTypeName = Rc<dyn TypeName>;

trait TypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String; // Make returning &str + use OnceCell to cache the result
    
    /// Returns the type name with generic parameters represented as T1, T2, etc.
    /// 
    /// # Parameters
    /// - `by_path_type_names`: Map of type names to their IDs
    /// - `own_param_map`: This type's own parameter mappings (type_id -> generic name)
    /// - `parent_param_map`: Parent type's parameter mappings - used for context-aware rendering
    /// - `type_id`: The ID of this type in the registry
    /// 
    /// The rendering logic:
    /// 1. First check if this type_id should be replaced using parent_param_map (parent context)
    /// 2. If not in parent context, check own_param_map (this type's own generics)
    /// 3. Otherwise render recursively with both maps passed down
    fn as_generic_string(
        &self,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
        own_param_map: &HashMap<u32, String>,
        parent_param_map: &HashMap<u32, String>,
        type_id: Option<u32>,
    ) -> String {
        // Check if this type_id should be replaced with a generic name
        // First check parent context (takes precedence), then own context
        if let Some(id) = type_id {
            if let Some(generic_name) = parent_param_map.get(&id) {
                return generic_name.clone();
            }
            if let Some(generic_name) = own_param_map.get(&id) {
                return generic_name.clone();
            }
        }
        // Default implementation returns the same as as_string for types without generics
        self.as_string(false, by_path_type_names)
    }
}

/// By path type name resolution.
struct ByPathTypeName {
    possible_names: Vec<(String, Vec<u32>)>,
    type_param_type_names: Vec<RcTypeName>,
}

impl ByPathTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let type_params = type_info.type_params.iter().try_fold(
            (
                Vec::with_capacity(type_info.type_params.len()),
                Vec::with_capacity(type_info.type_params.len()),
            ),
            |(mut type_param_ids, mut type_param_type_names), type_param| {
                let type_param_id = type_param
                    .ty
                    .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
                    .id;
                let type_param_type_name = resolve_type_name(
                    types,
                    type_param_id,
                    resolved_type_names,
                    by_path_type_names,
                )?;
                type_param_ids.push(type_param_id);
                type_param_type_names.push(type_param_type_name);
                Ok::<(Vec<u32>, Vec<Rc<dyn TypeName>>), Error>((
                    type_param_ids,
                    type_param_type_names,
                ))
            },
        )?;

        let mut possible_names = Self::possible_names_by_path(type_info).fold(
            Vec::with_capacity(type_info.path.segments.len() + 1),
            |mut possible_names, name| {
                let possible_name = (name.clone(), type_params.0.clone());
                possible_names.push(possible_name.clone());
                let name_ref_count = by_path_type_names
                    .entry((name.clone(), type_params.0.clone()))
                    .or_default();
                *name_ref_count += 1;
                possible_names
            },
        );
        if let Some(first_name) = possible_names.first() {
            // add numbered type name like `TypeName1`, `TypeName2` as last name
            // to solve name conflict with const generic parameters `<const N: size>`
            let name_ref_count = by_path_type_names.get(first_name).unwrap_or(&0);
            let name = format!("{}{}", first_name.0, name_ref_count);
            let possible_name = (name.clone(), first_name.1.clone());
            possible_names.push(possible_name);
            let name_ref_count = by_path_type_names
                .entry((name.clone(), type_params.0.clone()))
                .or_default();
            *name_ref_count += 1;
        } else {
            return Err(Error::TypeIsUnsupported(format!("{type_info:?}")));
        }

        Ok(Self {
            possible_names,
            type_param_type_names: type_params.1,
        })
    }

    fn possible_names_by_path(type_info: &Type<PortableForm>) -> impl Iterator<Item = String> + '_ {
        let mut name = String::default();
        type_info.path.segments.iter().rev().map(move |segment| {
            name = segment.to_case(Case::Pascal) + &name;
            name.clone()
        })
    }
}

impl TypeName for ByPathTypeName {
    fn as_string(
        &self,
        _for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let name = self
            .possible_names
            .iter()
            .find(|possible_name| {
                by_path_type_names
                    .get(possible_name)
                    .is_some_and(|ref_count| *ref_count == 1)
            })
            .unwrap_or_else(|| self.possible_names.last().unwrap());
        if self.type_param_type_names.is_empty() {
            name.0.clone()
        } else {
            let type_param_names = self
                .type_param_type_names
                .iter()
                .map(|tn| tn.as_string(true, by_path_type_names))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name.0, type_param_names)
        }
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let name = self
            .possible_names
            .iter()
            .find(|possible_name| {
                by_path_type_names
                    .get(possible_name)
                    .is_some_and(|ref_count| *ref_count == 1)
            })
            .unwrap_or_else(|| self.possible_names.last().unwrap());
        
        if self.type_param_type_names.is_empty() {
            name.0.clone()
        } else {
            // For each type parameter, check if it's in own_param_map or parent_param_map
            // If yes, use the mapped generic name (T1, T2, etc.)
            // If no, use as_generic_string recursively to handle nested types
            // When recursing, pass a merged map (own + parent) as parent_param_map for nested types
            
            // Merge parent_param_map and own_param_map (own takes precedence)
            let mut merged_param_map = parent_param_map.clone();
            for (k, v) in own_param_map {
                merged_param_map.insert(*k, v.clone());
            }
            
            let type_param_names = self
                .type_param_type_names
                .iter()
                .enumerate()
                .map(|(idx, tn)| {
                    // Get the type ID for this parameter from possible_names
                    let param_id = name.1.get(idx).copied();
                    
                    // Check if this type ID should be replaced with a generic name
                    // First check parent_param_map, then own_param_map
                    if let Some(param_id) = param_id {
                        if let Some(generic_name) = parent_param_map.get(&param_id) {
                            return generic_name.clone();
                        }
                        if let Some(generic_name) = own_param_map.get(&param_id) {
                            return generic_name.clone();
                        }
                    }
                    
                    // Otherwise, recursively resolve
                    // Pass merged map as parent_param_map for nested types (context propagation)
                    // Nested types get empty own_param_map since we're just rendering them in our context
                    let empty_map = HashMap::new();
                    tn.as_generic_string(by_path_type_names, &empty_map, &merged_param_map, param_id)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name.0, type_param_names)
        }
    }
}

/// BTreeMap type name resolution.
struct BTreeMapTypeName {
    key_type_name: RcTypeName,
    value_type_name: RcTypeName,
}

impl BTreeMapTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let key_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "K")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let value_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "V")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let key_type_name = resolve_type_name(
            types,
            key_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let value_type_name = resolve_type_name(
            types,
            value_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            key_type_name,
            value_type_name,
        })
    }

    pub fn is_btree_map_type(type_info: &Type<PortableForm>) -> bool {
        static BTREE_MAP_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let btree_map_type_info = BTREE_MAP_TYPE_INFO.get_or_init(BTreeMap::<u32, ()>::type_info);
        btree_map_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for BTreeMapTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let key_type_name = self
            .key_type_name
            .as_string(for_generic_param, by_path_type_names);
        let value_type_name = self
            .value_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("[({key_type_name}, {value_type_name})]")
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        // BTreeMap is a container with no generics of its own, so pass parent's context down
        let key_type_name = self
            .key_type_name
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, None);
        let value_type_name = self
            .value_type_name
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, None);

        format!("[({key_type_name}, {value_type_name})]")
    }
}

/// Result type name resolution.
pub(crate) struct ResultTypeName {
    ok_type_name: RcTypeName,
    err_type_name: RcTypeName,
}

impl ResultTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let ok_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "T")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let err_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "E")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let ok_type_name = resolve_type_name(
            types,
            ok_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let err_type_name = resolve_type_name(
            types,
            err_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            ok_type_name,
            err_type_name,
        })
    }

    pub fn is_result_type(type_info: &Type<PortableForm>) -> bool {
        static RESULT_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let result_type_info = RESULT_TYPE_INFO.get_or_init(StdResult::<(), ()>::type_info);
        result_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for ResultTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let ok_type_name = self
            .ok_type_name
            .as_string(for_generic_param, by_path_type_names);
        let err_type_name = self
            .err_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("Result<{ok_type_name}, {err_type_name}>")
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        // Result is a container with no generics of its own, so pass parent's context down
        let ok_type_name = self
            .ok_type_name
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, None);
        let err_type_name = self
            .err_type_name
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, None);

        format!("Result<{ok_type_name}, {err_type_name}>")
    }
}

/// Option type name resolution.
struct OptionTypeName {
    some_type_name: RcTypeName,
    some_type_id: u32,
}

impl OptionTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let some_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "T")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let some_type_name = resolve_type_name(
            types,
            some_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { some_type_name, some_type_id: some_type_id.id })
    }

    pub fn is_option_type(type_info: &Type<PortableForm>) -> bool {
        static OPTION_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let option_type_info = OPTION_TYPE_INFO.get_or_init(Option::<()>::type_info);
        option_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for OptionTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let some_type_name = self
            .some_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("Option<{some_type_name}>")
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        // Option is a container with no generics of its own, so pass parent's context down
        let some_type_name = self
            .some_type_name
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, Some(self.some_type_id));

        format!("Option<{some_type_name}>")
    }
}

/// Tuple type name resolution.
struct TupleTypeName {
    field_type_names: Vec<RcTypeName>,
    field_type_ids: Vec<u32>,
    /// Positions in the tuple that contain generic parameters (based on type_name parsing)
    generic_positions: Vec<usize>,
}

impl TupleTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        tuple_def: &TypeDefTuple<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
        _type_id: u32,
    ) -> Result<Self> {
        let field_type_ids: Vec<u32> = tuple_def.fields.iter().map(|field| field.id).collect();
        let field_type_names = tuple_def
            .fields
            .iter()
            .map(|field| {
                resolve_type_name(types, field.id, resolved_type_names, by_path_type_names)
            })
            .collect::<Result<Vec<_>>>()?;
        
        // TODO: We need to parse field type_name information from the parent context
        // For now, we cannot distinguish generic vs concrete tuple positions when they
        // resolve to the same type (e.g., (T, u32) when T=u32 becomes (u32, u32))
        let generic_positions = Vec::new();
        
        Ok(Self { field_type_names, field_type_ids, generic_positions })
    }
}

impl TypeName for TupleTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        format!(
            "({})",
            self.field_type_names
                .iter()
                .map(|tn| tn.as_string(for_generic_param, by_path_type_names))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        format!(
            "({})",
            self.field_type_names
                .iter()
                .enumerate()
                .map(|(idx, tn)| {
                    let field_type_id = self.field_type_ids.get(idx).copied();
                    // WORKAROUND: Tuples are always rendered with concrete types
                    // because scale-info doesn't preserve which positions are generic vs concrete
                    // This means `ReusableGenericEnum<(T, u32)>` will render as  
                    // `ReusableGenericEnum<(u32, u32)>` in generic form when T=u32
                    // TODO: This could be improved if we track type_name info from parent fields
                    tn.as_generic_string(by_path_type_names, &empty_map, &empty_map, field_type_id)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Parse a tuple's type_name field to identify which positions contain generic parameter names.
/// For example, "(T, u32)" would return vec![0] (position 0 is generic)
/// "(String, T1, T2)" would return vec![1, 2]
fn parse_tuple_generic_positions(type_name: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    
    // Remove surrounding parentheses if present
    let trimmed = type_name.trim();
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return positions;
    }
    
    let inner = &trimmed[1..trimmed.len()-1];
    
    // Split by comma, but need to handle nested structures like Vec<(T, u32)>
    let mut depth = 0;
    let mut current_element = String::new();
    let mut position = 0;
    
    for ch in inner.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                current_element.push(ch);
            }
            '>' | ')' | ']' => {
                depth -= 1;
                current_element.push(ch);
            }
            ',' if depth == 0 => {
                // End of current element
                if is_generic_param_name(&current_element.trim()) {
                    positions.push(position);
                }
                position += 1;
                current_element.clear();
            }
            _ => {
                current_element.push(ch);
            }
        }
    }
    
    // Don't forget the last element
    if !current_element.is_empty() {
        if is_generic_param_name(&current_element.trim()) {
            positions.push(position);
        }
    }
    
    positions
}

/// Check if a type name looks like a generic parameter (e.g., "T", "T1", "T2", "U", etc.)
/// Generic params are typically single uppercase letters or single letter followed by digits
fn is_generic_param_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    // Check if it's a simple identifier (no special characters like ::, <, >, etc.)
    if name.contains("::") || name.contains('<') || name.contains('>') 
        || name.contains('[') || name.contains(']') || name.contains('(') || name.contains(')') {
        return false;
    }
    
    // Check if it starts with an uppercase letter
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_uppercase() {
        return false;
    }
    
    // If it's just one uppercase letter, it's likely generic
    if name.len() == 1 {
        return true;
    }
    
    // If it's like "T1", "T2", etc., it's generic
    let rest: String = name.chars().skip(1).collect();
    if rest.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    
    // Otherwise, it's probably a concrete type like "String", "Option", etc.
    false
}

/// Vector type name resolution.
struct VectorTypeName {
    item_type_name: RcTypeName,
    item_type_id: u32,
}

impl VectorTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        vector_def: &TypeDefSequence<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_id = vector_def.type_param.id;
        let item_type_name = resolve_type_name(
            types,
            item_type_id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { item_type_name, item_type_id })
    }
}

impl TypeName for VectorTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let item_type_name = self
            .item_type_name
            .as_string(for_generic_param, by_path_type_names);
        format!("[{item_type_name}]")
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        let item_type_name = self
            .item_type_name
            // Vectors have no generics of their own, so pass parent's context down
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, Some(self.item_type_id));
        format!("[{item_type_name}]")
    }
}

/// Array type name resolution.
struct ArrayTypeName {
    item_type_name: RcTypeName,
    item_type_id: u32,
    len: u32,
}

impl ArrayTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        array_def: &TypeDefArray<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_id = array_def.type_param.id;
        let item_type_name = resolve_type_name(
            types,
            item_type_id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            item_type_name,
            item_type_id,
            len: array_def.len,
        })
    }
}

impl TypeName for ArrayTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let item_type_name = self
            .item_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("[{item_type_name}; {len}]", len = self.len)
    }
    
    fn as_generic_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>, own_param_map: &HashMap<u32, String>, parent_param_map: &HashMap<u32, String>, _type_id: Option<u32>) -> String {
        let empty_map = HashMap::new();
        let item_type_name = self
            .item_type_name
            // Arrays have no generics of their own, so pass parent's context down
            .as_generic_string(by_path_type_names, &empty_map, parent_param_map, Some(self.item_type_id));

        format!("[{item_type_name}; {len}]", len = self.len)
    }
}

/// Primitive type name resolution.
struct PrimitiveTypeName {
    name: &'static str,
}

impl PrimitiveTypeName {
    pub fn new(type_def: &TypeDefPrimitive) -> Result<Self> {
        let name = match type_def {
            TypeDefPrimitive::Bool => Ok("bool"),
            TypeDefPrimitive::Char => Ok("char"),
            TypeDefPrimitive::Str => Ok("String"),
            TypeDefPrimitive::U8 => Ok("u8"),
            TypeDefPrimitive::U16 => Ok("u16"),
            TypeDefPrimitive::U32 => Ok("u32"),
            TypeDefPrimitive::U64 => Ok("u64"),
            TypeDefPrimitive::U128 => Ok("u128"),
            TypeDefPrimitive::U256 => Err(Error::TypeIsUnsupported("u256".into())), // Rust doesn't have it
            TypeDefPrimitive::I8 => Ok("i8"),
            TypeDefPrimitive::I16 => Ok("i16"),
            TypeDefPrimitive::I32 => Ok("i32"),
            TypeDefPrimitive::I64 => Ok("i64"),
            TypeDefPrimitive::I128 => Ok("i128"),
            TypeDefPrimitive::I256 => Err(Error::TypeIsUnsupported("i256".into())), // Rust doesn't have it
        }?;
        Ok(Self { name })
    }
}

impl TypeName for PrimitiveTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        _by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        self.name.to_string()
    }
}

macro_rules! impl_primitive_alias_type_name {
    ($mod_name:ident, $primitive:ident) => {
        impl_primitive_alias_type_name!($mod_name, $primitive, $primitive);
    };

    ($mod_name:ident, $primitive:ident, $alias:ident) => {
        mod $mod_name {
            use super::*;

            pub(super) struct TypeNameImpl;

            impl TypeNameImpl {
                pub fn new() -> Self {
                    Self
                }

                pub fn is_type(type_info: &Type<PortableForm>) -> bool {
                    static TYPE_INFO: OnceLock<Type> = OnceLock::new();
                    let info = TYPE_INFO.get_or_init($primitive::type_info);
                    info.path.segments == type_info.path.segments
                }
            }

            impl TypeName for TypeNameImpl {
                fn as_string(
                    &self,
                    _for_generic_param: bool,
                    _by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
                ) -> String {
                    stringify!($alias).into()
                }
            }
        }
    };
}

impl_primitive_alias_type_name!(actor_id, ActorId);
impl_primitive_alias_type_name!(message_id, MessageId);
impl_primitive_alias_type_name!(code_id, CodeId);
impl_primitive_alias_type_name!(h160, H160);
impl_primitive_alias_type_name!(h256, H256);
impl_primitive_alias_type_name!(u256, U256, u256);
impl_primitive_alias_type_name!(nat8, NonZeroU8);
impl_primitive_alias_type_name!(nat16, NonZeroU16);
impl_primitive_alias_type_name!(nat32, NonZeroU32);
impl_primitive_alias_type_name!(nat64, NonZeroU64);
impl_primitive_alias_type_name!(nat128, NonZeroU128);
impl_primitive_alias_type_name!(nat256, NonZeroU256);

#[cfg(test)]
mod tests {
    use std::{array, result};

    use super::*;
    use scale_info::{MetaType, PortableRegistry, Registry};

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericConstStruct<const N: usize, const M: usize, T> {
        field: [T; N],
        field2: [T; M],
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    // #[allow(dead_code)]
    // #[derive(TypeInfo)]
    // struct AllCasesStruct<T1> {
    //     // Pretty basic generic usages
    //     f1: T1,
    //     f2: [T1; 16],
    //     f3: Vec<T1>,
    //     f4: Option<T1>,
    //     f5: result::Result<T1, String>,
    //     f6: result::Result<String, T1>,
    //     f7: result::Result<T1, T1>,
    //     f8: BTreeMap<String, T1>,
    //     f9: BTreeMap<T1, String>,
    //     f10: BTreeMap<T1, T1>,
    //     f11: (T1, String, T1),
    //     f12: (T1, T1),

    //     // Nested generic usages
    //     f11: (T1, Vec<T1>, Option<T1>, [T1; 8], result::Result<T1, String>, (T1, T1)),
    //     f12: Vec<(T1, u32, Option<T1>)>,
    //     f13: Option<Vec<T1>>,
    //     f14: result::Result<(Option<T1>, BTreeMap<T1, T1>), String>,
    //     f15: [(T1, Vec<T1>); 4],
    //     f16: Vec<[(Vec<Option<(T1, String, T1)>>, (bool, T1)); 2]>,

    //     // Generic structs and enums
    //     f16: NestedGenericStruct<T1>,
    //     f17: NestedGenericEnum<T1>,

    //     // Nested generics with structs and enums
    //     f18: NestedGenericStruct<Vec<[(T1, Option<T1>, Result<T1, (bool, T1)>); 4]>>,
    //     f19: NestedGenericEnum<BTreeMap<T1, Vec<Option<T1>>>>,
    //     f20: [NestedGenericStruct<(NestedGenericEnum<Option<[T1; 5]>>)>; 3],
    // }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct NestedGenericStruct<T> {
        simple: T,
        in_vec: Vec<T>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum NestedGenericEnum<T> {
        First(T),
        Second(Vec<T>),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ComplexGenericStruct<T> {
        // Basic generic usage
        array_field: [T; 32],
        vec_field: Vec<T>,
        tuple_field: (T, T, String),
        nested_vec: Vec<(T, u32)>,
        option_field: Option<T>,
        result_field: result::Result<T, String>,
        
        // Nested struct/enum with generics
        nested_struct: NestedGenericStruct<T>,
        nested_enum: NestedGenericEnum<T>,
        
        // Array of generic types
        array_of_option: [Option<T>; 5],
        array_of_result: [result::Result<T, String>; 3],
        array_of_vec: [Vec<T>; 2],
        array_of_tuple: [(T, u32); 4],
        array_of_nested_struct: [NestedGenericStruct<T>; 2],
        
        // Vec of generic types
        vec_of_option: Vec<Option<T>>,
        vec_of_result: Vec<result::Result<T, u32>>,
        vec_of_nested_enum: Vec<NestedGenericEnum<T>>,
        
        // Tuple of generic types
        tuple_of_vecs: (Vec<T>, Vec<T>),
        tuple_of_options: (Option<T>, Option<T>, Option<u32>),
        tuple_mixed: (T, Vec<T>, Option<T>, [T; 8]),
        
        // Deep nesting
        vec_of_vec: Vec<Vec<T>>,
        option_of_vec: Option<Vec<T>>,
        result_of_option: result::Result<Option<T>, String>,
        vec_of_tuple_of_option: Vec<(Option<T>, Option<T>)>,
        
        // BTreeMap-like (represented as Vec of tuples)
        map_generic_key: Vec<(T, String)>,
        map_generic_value: Vec<(u32, T)>,
        map_both_generic: Vec<(T, Vec<T>)>,
        
        // Complex combinations
        option_of_nested_struct: Option<NestedGenericStruct<T>>,
        result_of_vec_of_tuple: result::Result<Vec<(T, T)>, String>,
        vec_of_array: Vec<[T; 16]>,
        array_of_option_of_vec: [Option<Vec<T>>; 3],
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MultiGenericStruct<T1, T2, T3> {
        // Basic multi-generic
        field1: Vec<T1>,
        field2: [T2; 10],
        field3: (T1, T2, T3),
        nested: Vec<(T1, Option<T2>)>,
        map_like: Vec<(T1, T3)>,
        
        // Nested structs with different generics
        nested1: NestedGenericStruct<T1>,
        nested2: NestedGenericStruct<T2>,
        nested_mixed: GenericStruct<T3>,
        
        // Arrays with different generics
        array_t1: [T1; 5],
        array_t2_vec: [Vec<T2>; 3],
        array_t3_option: [Option<T3>; 4],
        
        // Tuples mixing all generics
        tuple_all: (T1, T2, T3),
        tuple_repeated: (T1, T1, T2, T2, T3, T3),
        tuple_nested: (Vec<T1>, Option<T2>, result::Result<T3, String>),
        
        // Complex nesting with multiple generics
        vec_of_enum: Vec<GenericEnum<T1, T2>>,
        array_of_enum: [GenericEnum<T2, T3>; 2],
        option_of_enum: Option<GenericEnum<T1, T3>>,
        
        // Deep combinations
        vec_of_tuple_mixed: Vec<(T1, Vec<T2>, Option<T3>)>,
        result_multi: result::Result<(T1, T2), T3>,
        option_of_result: Option<result::Result<T1, T2>>,
        
        // Map-like with different generics
        map_t1_t2: Vec<(T1, T2)>,
        map_t2_vec_t3: Vec<(T2, Vec<T3>)>,
        map_nested: Vec<(T1, NestedGenericStruct<T2>)>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum ComplexGenericEnum<T1, T2> {
        // Variant with named fields
        Variant1 {
            vec_field: Vec<T1>,
            tuple_field: (T1, T2),
            nested: NestedGenericStruct<T2>,
            array_field: [T1; 8],
        },
        // Variant with array
        Variant2([T2; 16]),
        // Variant with option
        Variant3(Option<T1>),
        // Variant with result
        Variant4(result::Result<T1, T2>),
        // Variant with nested enum
        Variant5(NestedGenericEnum<T1>),
        // Variant with complex tuple
        Variant6(Vec<T1>, Option<T2>, [T1; 4]),
        // Variant with vec of tuples
        Variant7 {
            map_like: Vec<(T1, T2)>,
            nested_vec: Vec<Vec<T1>>,
        },
        // Variant with deeply nested types
        Variant8(Vec<(T1, Option<Vec<T2>>)>),
        // Variant mixing generics with GenericEnum
        Variant9(GenericEnum<T1, T2>),
        // Variant with option of result
        Variant10 {
            complex: Option<result::Result<Vec<T1>, T2>>,
        },
    }

    // Structures for testing same generic type with different parameters
    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ReusableGenericStruct<T> {
        data: T,
        count: u32,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum ReusableGenericEnum<T> {
        Some(T),
        None,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct StructWithReusedGenerics<T> {
        // Same struct with different generic instantiations
        f1: ReusableGenericStruct<T>,
        f2: ReusableGenericStruct<(Vec<T>, T)>,
        f3: ReusableGenericStruct<bool>,
        f4: ReusableGenericStruct<Vec<T>>,
        f5: ReusableGenericStruct<Option<T>>,
        
        // Same enum with different generic instantiations
        e1: ReusableGenericEnum<T>,
        e2: ReusableGenericEnum<String>,
        e3: ReusableGenericEnum<[T; 16]>,
        e4: ReusableGenericEnum<(T, u32)>,
        
        // Nested reuse with GenericStruct
        g1: GenericStruct<T>,
        g2: GenericStruct<ReusableGenericStruct<T>>,
        g3: GenericStruct<Vec<ReusableGenericStruct<T>>>,
        
        // Arrays of reused generics
        array1: [ReusableGenericStruct<T>; 3],
        array2: [ReusableGenericEnum<T>; 5],
        
        // Vecs of reused generics
        vec1: Vec<ReusableGenericStruct<T>>,
        vec2: Vec<ReusableGenericEnum<(T, T)>>,
        
        // Options and Results with reused generics
        opt: Option<ReusableGenericStruct<T>>,
        res: result::Result<ReusableGenericEnum<T>, String>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct MultiLevelReuse<T1, T2> {
        // Reusing with first generic
        a1: ReusableGenericStruct<T1>,
        a2: ReusableGenericStruct<Vec<T1>>,
        a3: ReusableGenericStruct<(T1, T2)>,
        
        // Reusing with second generic
        b1: ReusableGenericStruct<T2>,
        b2: ReusableGenericStruct<[T2; 8]>,
        b3: ReusableGenericStruct<Option<T2>>,
        
        // Reusing with both generics mixed
        c1: ReusableGenericEnum<T1>,
        c2: ReusableGenericEnum<T2>,
        c3: ReusableGenericEnum<(T1, T2)>,
        c4: ReusableGenericEnum<Vec<(T1, T2)>>,
        
        // Deeply nested reuse
        nested1: GenericStruct<ReusableGenericStruct<T1>>,
        nested2: GenericEnum<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>,
        nested3: Vec<(ReusableGenericStruct<T1>, ReusableGenericEnum<T2>)>,
        
        // Triple nesting
        triple: NestedGenericStruct<ReusableGenericStruct<T1>>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum EnumWithReusedGenerics<T> {
        Variant1 {
            same1: ReusableGenericStruct<T>,
            same2: ReusableGenericStruct<Vec<T>>,
            same3: ReusableGenericStruct<u32>,
        },
        Variant2(
            ReusableGenericEnum<T>,
            ReusableGenericEnum<String>,
        ),
        Variant3([ReusableGenericStruct<T>; 4]),
        Variant4 {
            vec_of_reused: Vec<ReusableGenericStruct<T>>,
            tuple_of_reused: (
                ReusableGenericEnum<T>,
                ReusableGenericEnum<bool>,
                ReusableGenericEnum<(T, T)>,
            ),
        },
    }

    #[allow(dead_code)]
    mod mod_1 {
        use super::*;

        #[derive(TypeInfo)]
        pub struct T1 {}

        pub mod mod_2 {
            use super::*;

            #[derive(TypeInfo)]
            pub struct T2 {}
        }
    }

    #[allow(dead_code)]
    mod mod_2 {
        use super::*;

        #[derive(TypeInfo)]
        pub struct T1 {}

        #[derive(TypeInfo)]
        pub struct T2 {}
    }

    #[test]
    fn h256_u256_type_name_resolution_works() {
        let mut registry = Registry::new();
        let h256_id = registry.register_type(&MetaType::new::<H256>()).id;
        let h256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<H256>>())
            .id;
        let u256_id = registry.register_type(&MetaType::new::<U256>()).id;
        let u256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<U256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let h256_name = type_names.get(&h256_id).unwrap();
        assert_eq!(h256_name, "H256");
        let as_generic_param_name = type_names.get(&h256_as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<H256>");
        let u256_name = type_names.get(&u256_id).unwrap();
        assert_eq!(u256_name, "u256");
        let as_generic_param_name = type_names.get(&u256_as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<u256>");
    }

    #[test]
    fn generic_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<u32>>())
            .id;
        let string_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<String>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_struct_name = type_names.get(&u32_struct_id).unwrap();
        assert_eq!(u32_struct_name, "GenericStruct<u32>");

        let string_struct_name = type_names.get(&string_struct_id).unwrap();
        assert_eq!(string_struct_name, "GenericStruct<String>");
    }

    #[test]
    fn generic_variant_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_string_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<u32, String>>())
            .id;
        let bool_u32_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<bool, u32>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_string_enum_name = type_names.get(&u32_string_enum_id).unwrap();
        assert_eq!(u32_string_enum_name, "GenericEnum<u32, String>");

        let bool_u32_enum_name = type_names.get(&bool_u32_enum_id).unwrap();
        assert_eq!(bool_u32_enum_name, "GenericEnum<bool, u32>");
    }

    #[test]
    fn array_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_array_id = registry.register_type(&MetaType::new::<[u32; 10]>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<[u32; 10]>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_array_name = type_names.get(&u32_array_id).unwrap();
        assert_eq!(u32_array_name, "[u32, 10]");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<[u32, 10]>");
    }

    #[test]
    fn vector_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_vector_id = registry.register_type(&MetaType::new::<Vec<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Vec<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_vector_name = type_names.get(&u32_vector_id).unwrap();
        assert_eq!(u32_vector_name, "[u32]");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<[u32]>");
    }

    #[test]
    fn result_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_result_id = registry
            .register_type(&MetaType::new::<result::Result<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<result::Result<u32, String>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_result_name = type_names.get(&u32_result_id).unwrap();
        assert_eq!(u32_result_name, "Result<u32, String>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<Result<u32, String>>");
    }

    #[test]
    fn option_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_option_id = registry.register_type(&MetaType::new::<Option<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Option<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_option_name = type_names.get(&u32_option_id).unwrap();
        assert_eq!(u32_option_name, "Option<u32>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<Option<u32>>");
    }

    #[test]
    fn tuple_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_str_tuple_id = registry.register_type(&MetaType::new::<(u32, String)>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<(u32, String)>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_str_tuple_name = type_names.get(&u32_str_tuple_id).unwrap();
        assert_eq!(u32_str_tuple_name, "(u32, String)");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<(u32, String)>");
    }

    #[test]
    fn btree_map_type_name_resolution_works() {
        let mut registry = Registry::new();
        let btree_map_id = registry
            .register_type(&MetaType::new::<BTreeMap<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<BTreeMap<u32, String>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let btree_map_name = type_names.get(&btree_map_id).unwrap();
        assert_eq!(btree_map_name, "[(u32, String)]");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<[(u32, String)]>");
    }

    #[test]
    fn type_name_minification_works_for_types_with_the_same_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type(&MetaType::new::<mod_1::T1>()).id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T1>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let t1_name = type_names.get(&t1_id).unwrap();
        assert_eq!(t1_name, "Mod1T1");

        let t2_name = type_names.get(&t2_id).unwrap();
        assert_eq!(t2_name, "Mod2T1");
    }

    #[test]
    fn type_name_minification_works_for_types_with_different_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry
            .register_type(&MetaType::new::<mod_1::mod_2::T2>())
            .id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T2>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let t1_name = type_names.get(&t1_id).unwrap();
        assert_eq!(t1_name, "Mod1Mod2T2");

        let t2_name = type_names.get(&t2_id).unwrap();
        assert_eq!(t2_name, "TestsMod2T2");
    }

    macro_rules! type_name_resolution_works {
        ($primitive:ident) => {
            let mut registry = Registry::new();
            let id = registry.register_type(&MetaType::new::<$primitive>()).id;
            let as_generic_param_id = registry
                .register_type(&MetaType::new::<GenericStruct<$primitive>>())
                .id;
            let portable_registry = PortableRegistry::from(registry);

            let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

            let name = type_names.get(&id).unwrap();
            assert_eq!(name, stringify!($primitive));
            let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
            assert_eq!(
                as_generic_param_name,
                concat!("GenericStruct<", stringify!($primitive), ">")
            );
        };
    }

    #[test]
    fn actor_id_type_name_resolution_works() {
        type_name_resolution_works!(ActorId);
    }

    #[test]
    fn message_id_type_name_resolution_works() {
        type_name_resolution_works!(MessageId);
    }

    #[test]
    fn code_id_type_name_resolution_works() {
        type_name_resolution_works!(CodeId);
    }

    #[test]
    fn h160_type_name_resolution_works() {
        type_name_resolution_works!(H160);
    }

    #[test]
    fn nonzero_u8_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU8);
    }

    #[test]
    fn nonzero_u16_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU16);
    }

    #[test]
    fn nonzero_u32_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU32);
    }

    #[test]
    fn nonzero_u64_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU64);
    }

    #[test]
    fn nonzero_u128_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU128);
    }

    #[test]
    fn nonzero_u256_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU256);
    }

    #[test]
    fn generic_const_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let n8_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<8, 8, u8>>())
            .id;
        let n8_id_2 = registry
            .register_type(&MetaType::new::<GenericConstStruct<8, 8, u8>>())
            .id;
        let n32_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<32, 8, u8>>())
            .id;
        let n256_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<256, 832, u8>>())
            .id;
        let n32u256_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<32, 8, U256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        assert_eq!(n8_id, n8_id_2);
        assert_ne!(n8_id, n32_id);
        assert_ne!(n8_id, n256_id);
        assert_eq!(type_names.get(&n8_id).unwrap(), "GenericConstStruct1<u8>");
        assert_eq!(type_names.get(&n32_id).unwrap(), "GenericConstStruct2<u8>");
        assert_eq!(type_names.get(&n256_id).unwrap(), "GenericConstStruct3<u8>");
        assert_eq!(
            type_names.get(&n32u256_id).unwrap(),
            "GenericConstStruct<u256>"
        );
    }

    #[test]
    fn generic_type_names_resolution_works() {
        let mut registry = Registry::new();
        
        // Simple generic types
        let generic_struct_bool_id = registry
            .register_type(&MetaType::new::<GenericStruct<bool>>())
            .id;
        let generic_struct_string_id = registry
            .register_type(&MetaType::new::<GenericStruct<String>>())
            .id;
        let generic_enum_u32_string_id = registry
            .register_type(&MetaType::new::<GenericEnum<u32, String>>())
            .id;
        let generic_enum_bool_u64_id = registry
            .register_type(&MetaType::new::<GenericEnum<bool, u64>>())
            .id;
        
        // Complex nested generic types
        let complex_struct_u32_id = registry
            .register_type(&MetaType::new::<ComplexGenericStruct<u32>>())
            .id;
        let complex_struct_bool_id = registry
            .register_type(&MetaType::new::<ComplexGenericStruct<bool>>())
            .id;
        
        // Multi-parameter generic types
        let multi_generic_id = registry
            .register_type(&MetaType::new::<MultiGenericStruct<u8, u32, String>>())
            .id;
        let multi_generic_id2 = registry
            .register_type(&MetaType::new::<MultiGenericStruct<bool, u64, H256>>())
            .id;
        
        // Complex enum with nested generics
        let complex_enum_id = registry
            .register_type(&MetaType::new::<ComplexGenericEnum<u32, String>>())
            .id;
        let complex_enum_id2 = registry
            .register_type(&MetaType::new::<ComplexGenericEnum<H256, Vec<u8>>>())
            .id;
        
        let portable_registry = PortableRegistry::from(registry);

        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check concrete names for simple types
        assert_eq!(
            concrete_names.get(&generic_struct_bool_id).unwrap(),
            "GenericStruct<bool>"
        );
        assert_eq!(
            concrete_names.get(&generic_struct_string_id).unwrap(),
            "GenericStruct<String>"
        );
        assert_eq!(
            concrete_names.get(&generic_enum_u32_string_id).unwrap(),
            "GenericEnum<u32, String>"
        );
        assert_eq!(
            concrete_names.get(&generic_enum_bool_u64_id).unwrap(),
            "GenericEnum<bool, u64>"
        );

        // Check generic names for simple types
        assert_eq!(
            generic_names.get(&generic_struct_bool_id).unwrap(),
            "GenericStruct<T1>"
        );
        assert_eq!(
            generic_names.get(&generic_struct_string_id).unwrap(),
            "GenericStruct<T1>"
        );
        assert_eq!(
            generic_names.get(&generic_enum_u32_string_id).unwrap(),
            "GenericEnum<T1, T2>"
        );
        assert_eq!(
            generic_names.get(&generic_enum_bool_u64_id).unwrap(),
            "GenericEnum<T1, T2>"
        );

        // Check concrete names for complex nested types
        // ComplexGenericStruct has: [T; 32], Vec<T>, (T, T, String), Vec<(T, u32)>, Option<T>, Result<T, String>
        assert_eq!(
            concrete_names.get(&complex_struct_u32_id).unwrap(),
            "ComplexGenericStruct<u32>"
        );
        assert_eq!(
            concrete_names.get(&complex_struct_bool_id).unwrap(),
            "ComplexGenericStruct<bool>"
        );
        
        // Both should have the same generic name regardless of concrete type
        assert_eq!(
            generic_names.get(&complex_struct_u32_id).unwrap(),
            "ComplexGenericStruct<T1>"
        );
        assert_eq!(
            generic_names.get(&complex_struct_bool_id).unwrap(),
            "ComplexGenericStruct<T1>"
        );

        // Check multi-parameter generic types
        assert_eq!(
            concrete_names.get(&multi_generic_id).unwrap(),
            "MultiGenericStruct<u8, u32, String>"
        );
        assert_eq!(
            concrete_names.get(&multi_generic_id2).unwrap(),
            "MultiGenericStruct<bool, u64, H256>"
        );
        
        // Both should have the same generic name with T1, T2, T3
        assert_eq!(
            generic_names.get(&multi_generic_id).unwrap(),
            "MultiGenericStruct<T1, T2, T3>"
        );
        assert_eq!(
            generic_names.get(&multi_generic_id2).unwrap(),
            "MultiGenericStruct<T1, T2, T3>"
        );

        // Check complex enum with nested generics
        assert_eq!(
            concrete_names.get(&complex_enum_id).unwrap(),
            "ComplexGenericEnum<u32, String>"
        );
        assert_eq!(
            concrete_names.get(&complex_enum_id2).unwrap(),
            "ComplexGenericEnum<H256, [u8]>"
        );
        
        // Both should have the same generic name
        assert_eq!(
            generic_names.get(&complex_enum_id).unwrap(),
            "ComplexGenericEnum<T1, T2>"
        );
        assert_eq!(
            generic_names.get(&complex_enum_id2).unwrap(),
            "ComplexGenericEnum<T1, T2>"
        );
    }

    #[test]
    fn reused_generic_types_resolution_works() {
        let mut registry = Registry::new();
        
        // Register struct that reuses generic types with different parameters
        let reused_struct_u32_id = registry
            .register_type(&MetaType::new::<StructWithReusedGenerics<u32>>())
            .id;
        let reused_struct_string_id = registry
            .register_type(&MetaType::new::<StructWithReusedGenerics<String>>())
            .id;
        
        // Register multi-level reuse
        let multi_reuse_id = registry
            .register_type(&MetaType::new::<MultiLevelReuse<u64, bool>>())
            .id;
        let multi_reuse_id2 = registry
            .register_type(&MetaType::new::<MultiLevelReuse<String, H256>>())
            .id;
        
        // Register enum with reused generics
        let enum_reused_id = registry
            .register_type(&MetaType::new::<EnumWithReusedGenerics<u32>>())
            .id;
        let enum_reused_id2 = registry
            .register_type(&MetaType::new::<EnumWithReusedGenerics<Vec<bool>>>())
            .id;
        
        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check concrete names - different instantiations have different concrete names
        assert_eq!(
            concrete_names.get(&reused_struct_u32_id).unwrap(),
            "StructWithReusedGenerics<u32>"
        );
        assert_eq!(
            concrete_names.get(&reused_struct_string_id).unwrap(),
            "StructWithReusedGenerics<String>"
        );
        
        // But they should have the same generic name
        assert_eq!(
            generic_names.get(&reused_struct_u32_id).unwrap(),
            "StructWithReusedGenerics<T1>"
        );
        assert_eq!(
            generic_names.get(&reused_struct_string_id).unwrap(),
            "StructWithReusedGenerics<T1>"
        );

        // Check multi-level reuse with two generics
        assert_eq!(
            concrete_names.get(&multi_reuse_id).unwrap(),
            "MultiLevelReuse<u64, bool>"
        );
        assert_eq!(
            concrete_names.get(&multi_reuse_id2).unwrap(),
            "MultiLevelReuse<String, H256>"
        );
        
        // Both should map to T1, T2
        assert_eq!(
            generic_names.get(&multi_reuse_id).unwrap(),
            "MultiLevelReuse<T1, T2>"
        );
        assert_eq!(
            generic_names.get(&multi_reuse_id2).unwrap(),
            "MultiLevelReuse<T1, T2>"
        );

        // Check enum with reused generics
        assert_eq!(
            concrete_names.get(&enum_reused_id).unwrap(),
            "EnumWithReusedGenerics<u32>"
        );
        assert_eq!(
            concrete_names.get(&enum_reused_id2).unwrap(),
            "EnumWithReusedGenerics<[bool]>"
        );
        
        // Both should have same generic name
        assert_eq!(
            generic_names.get(&enum_reused_id).unwrap(),
            "EnumWithReusedGenerics<T1>"
        );
        assert_eq!(
            generic_names.get(&enum_reused_id2).unwrap(),
            "EnumWithReusedGenerics<T1>"
        );

        // Check field types in StructWithReusedGenerics
        let reused_struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == reused_struct_u32_id)
            .unwrap();
        
        if let TypeDef::Composite(composite) = &reused_struct_type.ty.type_def {
            // f1: ReusableGenericStruct<T> where T=u32
            let f1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("f1")).unwrap();
            eprintln!("f1 concrete: {}, generic: {}", 
                concrete_names.get(&f1.ty.id).unwrap(),
                generic_names.get(&f1.ty.id).unwrap());
            assert_eq!(concrete_names.get(&f1.ty.id).unwrap(), "ReusableGenericStruct<u32>");
            assert_eq!(generic_names.get(&f1.ty.id).unwrap(), "ReusableGenericStruct<T1>");
            
            // f2: ReusableGenericStruct<(Vec<T>, T)> where T=u32
            let f2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("f2")).unwrap();
            eprintln!("f2 concrete: {}, generic: {}", 
                concrete_names.get(&f2.ty.id).unwrap(),
                generic_names.get(&f2.ty.id).unwrap());
            assert_eq!(concrete_names.get(&f2.ty.id).unwrap(), "ReusableGenericStruct<([u32], u32)>");
            assert_eq!(generic_names.get(&f2.ty.id).unwrap(), "ReusableGenericStruct<([T1], T1)>");
            
            // f3: ReusableGenericStruct<bool>
            // NOTE: Type 6 appears in 3 contexts: 2 where bool is concrete (types 0, 27) and 1 where bool is T2 (type 50).
            // Using majority vote (66% concrete), type 6's own param mapping for bool is removed.
            // Therefore, type 6 renders as concrete bool in the generic_names map.
            let f3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("f3")).unwrap();
            eprintln!("f3 concrete: {}, generic: {}", 
                concrete_names.get(&f3.ty.id).unwrap(),
                generic_names.get(&f3.ty.id).unwrap());
            assert_eq!(concrete_names.get(&f3.ty.id).unwrap(), "ReusableGenericStruct<bool>");
            assert_eq!(generic_names.get(&f3.ty.id).unwrap(), "ReusableGenericStruct<bool>");
            
            // e1: ReusableGenericEnum<T> where T=u32
            let e1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("e1")).unwrap();
            assert_eq!(concrete_names.get(&e1.ty.id).unwrap(), "ReusableGenericEnum<u32>");
            assert_eq!(generic_names.get(&e1.ty.id).unwrap(), "ReusableGenericEnum<T1>");
            
            // e2: ReusableGenericEnum<String> (different concrete type, NOT a generic parameter)
            let e2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("e2")).unwrap();
            assert_eq!(concrete_names.get(&e2.ty.id).unwrap(), "ReusableGenericEnum<String>");
            assert_eq!(generic_names.get(&e2.ty.id).unwrap(), "ReusableGenericEnum<T1>");  // String is the parameter

            let e3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("e3")).unwrap();
            assert_eq!(concrete_names.get(&e3.ty.id).unwrap(), "ReusableGenericEnum<[u32; 16]>");
            assert_eq!(generic_names.get(&e3.ty.id).unwrap(), "ReusableGenericEnum<[T1; 16]>");

            let e4 = composite.fields.iter().find(|f| f.name.as_deref() == Some("e4")).unwrap();
            assert_eq!(concrete_names.get(&e4.ty.id).unwrap(), "ReusableGenericEnum<(u32, u32)>");
            assert_eq!(generic_names.get(&e4.ty.id).unwrap(), "ReusableGenericEnum<(T1, u32)>");

            let g1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("g1")).unwrap();
            assert_eq!(concrete_names.get(&g1.ty.id).unwrap(), "GenericStruct<u32>");
            assert_eq!(generic_names.get(&g1.ty.id).unwrap(), "GenericStruct<T1>");
            
            // g2: GenericStruct<ReusableGenericStruct<T>> - nested reuse
            let g2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("g2")).unwrap();
            assert_eq!(concrete_names.get(&g2.ty.id).unwrap(), "GenericStruct<ReusableGenericStruct<u32>>");
            assert_eq!(generic_names.get(&g2.ty.id).unwrap(), "GenericStruct<ReusableGenericStruct<T1>>");

            let g3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("g3")).unwrap();
            assert_eq!(concrete_names.get(&g3.ty.id).unwrap(), "GenericStruct<[ReusableGenericStruct<u32>]>");
            assert_eq!(generic_names.get(&g3.ty.id).unwrap(), "GenericStruct<[ReusableGenericStruct<T1>]>");

            let array1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("array1")).unwrap();
            assert_eq!(concrete_names.get(&array1.ty.id).unwrap(), "[ReusableGenericEnum<u32>; 4]");
            assert_eq!(generic_names.get(&array1.ty.id).unwrap(), "[ReusableGenericEnum<T1>; 4]");

            let array2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("array2")).unwrap();
            assert_eq!(concrete_names.get(&array2.ty.id).unwrap(), "[ReusableGenericEnum<String>; 4]");
            assert_eq!(generic_names.get(&array2.ty.id).unwrap(), "[ReusableGenericEnum<T2>; 4]");

            let vec1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("vec1")).unwrap();
            assert_eq!(concrete_names.get(&vec1.ty.id).unwrap(), "[ReusableGenericStruct<u32>]");
            assert_eq!(generic_names.get(&vec1.ty.id).unwrap(), "[ReusableGenericStruct<T1>]");

            let vec2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("vec2")).unwrap();
            assert_eq!(concrete_names.get(&vec2.ty.id).unwrap(), "[ReusableGenericEnum<(u32, u32)>]");
            assert_eq!(generic_names.get(&vec2.ty.id).unwrap(), "[ReusableGenericEnum<(T1, T1)>]");

            let opt = composite.fields.iter().find(|f| f.name.as_deref() == Some("opt")).unwrap();
            assert_eq!(concrete_names.get(&opt.ty.id).unwrap(), "Option<ReusableGenericStruct<u32>>");
            assert_eq!(generic_names.get(&opt.ty.id).unwrap(), "Option<ReusableGenericStruct<T1>>");

            let res = composite.fields.iter().find(|f| f.name.as_deref() == Some("res")).unwrap();
            assert_eq!(concrete_names.get(&res.ty.id).unwrap(), "Result<ReusableGenericEnum<u32>, String>");
            assert_eq!(generic_names.get(&res.ty.id).unwrap(), "Result<ReusableGenericEnum<T1>, String>");
        } else {
            panic!("Expected composite type");
        }
        
        // Check field types in MultiLevelReuse
        let multi_reuse_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == multi_reuse_id)
            .unwrap();
        
        if let TypeDef::Composite(composite) = &multi_reuse_type.ty.type_def {
            // a1: ReusableGenericStruct<T1> where T1=u64
            let a1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("a1")).unwrap();
            eprintln!("a1: type_id={}, concrete={}, generic={}", 
                a1.ty.id, 
                concrete_names.get(&a1.ty.id).unwrap(), 
                generic_names.get(&a1.ty.id).unwrap());
            assert_eq!(concrete_names.get(&a1.ty.id).unwrap(), "ReusableGenericStruct<u64>");
            assert_eq!(generic_names.get(&a1.ty.id).unwrap(), "ReusableGenericStruct<T1>");

            let a2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("a2")).unwrap();
            eprintln!("a2: type_id={}, concrete={}, generic={}", 
                a2.ty.id, 
                concrete_names.get(&a2.ty.id).unwrap(), 
                generic_names.get(&a2.ty.id).unwrap());
            assert_eq!(concrete_names.get(&a2.ty.id).unwrap(), "ReusableGenericStruct<[u64]>");
            assert_eq!(generic_names.get(&a2.ty.id).unwrap(), "ReusableGenericStruct<[T1]>");

            let a3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("a3")).unwrap();
            eprintln!("a3: type_id={}, concrete={}, generic={}", 
                a3.ty.id, 
                concrete_names.get(&a3.ty.id).unwrap(), 
                generic_names.get(&a3.ty.id).unwrap());
            assert_eq!(concrete_names.get(&a3.ty.id).unwrap(), "ReusableGenericStruct<(u64, bool)>");
            assert_eq!(generic_names.get(&a3.ty.id).unwrap(), "ReusableGenericStruct<(T1, T2)>");
            
            let b1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("b1")).unwrap();
            eprintln!("b1: type_id={}, concrete={}, generic={}", 
                b1.ty.id, 
                concrete_names.get(&b1.ty.id).unwrap(), 
                generic_names.get(&b1.ty.id).unwrap());
            assert_eq!(concrete_names.get(&b1.ty.id).unwrap(), "ReusableGenericStruct<bool>");
            assert_eq!(generic_names.get(&b1.ty.id).unwrap(), "ReusableGenericStruct<T2>");

            let b2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("b2")).unwrap();
            assert_eq!(concrete_names.get(&b2.ty.id).unwrap(), "ReusableGenericStruct<[bool, 8]>");
            assert_eq!(generic_names.get(&b2.ty.id).unwrap(), "ReusableGenericStruct<[T2, 8]>");

            let b3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("b3")).unwrap();
            eprintln!("b3: type_id={}, concrete={}, generic={}", 
                b3.ty.id, 
                concrete_names.get(&b3.ty.id).unwrap(), 
                generic_names.get(&b3.ty.id).unwrap());
            assert_eq!(concrete_names.get(&b3.ty.id).unwrap(), "ReusableGenericStruct<Option<bool>>");
            assert_eq!(generic_names.get(&b3.ty.id).unwrap(), "ReusableGenericStruct<Option<T2>>");


            let c1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("c1")).unwrap();
            assert_eq!(concrete_names.get(&c1.ty.id).unwrap(), "ReusableGenericEnum<u64>");
            assert_eq!(generic_names.get(&c1.ty.id).unwrap(), "ReusableGenericEnum<T1>");

            let c2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("c2")).unwrap();
            eprintln!("c2: type_id={}, concrete={}, generic={}", 
                c2.ty.id, 
                concrete_names.get(&c2.ty.id).unwrap(), 
                generic_names.get(&c2.ty.id).unwrap());
            assert_eq!(concrete_names.get(&c2.ty.id).unwrap(), "ReusableGenericEnum<bool>");
            assert_eq!(generic_names.get(&c2.ty.id).unwrap(), "ReusableGenericEnum<T2>");

            let c3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("c3")).unwrap();
            assert_eq!(concrete_names.get(&c3.ty.id).unwrap(), "ReusableGenericEnum<(u64, bool)>");
            assert_eq!(generic_names.get(&c3.ty.id).unwrap(), "ReusableGenericEnum<(T1, T2)>");

            let c4 = composite.fields.iter().find(|f| f.name.as_deref() == Some("c4")).unwrap();
            assert_eq!(concrete_names.get(&c4.ty.id).unwrap(), "ReusableGenericEnum<[(u64, bool)]>");
            assert_eq!(generic_names.get(&c4.ty.id).unwrap(), "ReusableGenericEnum<[(T1, T2)]>");

            let nested1 = composite.fields.iter().find(|f| f.name.as_deref() == Some("nested1")).unwrap();
            assert_eq!(
                concrete_names.get(&nested1.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u64>>"
            );
            assert_eq!(
                generic_names.get(&nested1.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<T1>>"  // The GenericStruct's own parameters
            );
            
            // nested2: GenericEnum<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>
            // Note: Each nested type gets its own T1, T2 parameters
            let nested2 = composite.fields.iter().find(|f| f.name.as_deref() == Some("nested2")).unwrap();
            assert_eq!(
                concrete_names.get(&nested2.ty.id).unwrap(),
                "GenericEnum<ReusableGenericStruct<u64>, ReusableGenericEnum<bool>>"
            );
            assert_eq!(
                generic_names.get(&nested2.ty.id).unwrap(),
                "GenericEnum<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>"  // The GenericEnum's own parameters
            );

            let nested3 = composite.fields.iter().find(|f| f.name.as_deref() == Some("nested3")).unwrap();
            assert_eq!(
                concrete_names.get(&nested3.ty.id).unwrap(),
                "[(ReusableGenericStruct<u64>, ReusableGenericEnum<bool>)]"
            );
            assert_eq!(
                generic_names.get(&nested3.ty.id).unwrap(),
                "[(ReusableGenericStruct<T1>, ReusableGenericEnum<T2>)]"
            );

            let triple = composite.fields.iter().find(|f| f.name.as_deref() == Some("triple")).unwrap();
            assert_eq!(
                concrete_names.get(&triple.ty.id).unwrap(),
                "NestedGenericStruct<ReusableGenericStruct<u64>>"
            );
            assert_eq!(
                generic_names.get(&triple.ty.id).unwrap(),
                "NestedGenericStruct<ReusableGenericStruct<T1>>"
            );
        } else {
            panic!("Expected composite type");
        }
    }
}

