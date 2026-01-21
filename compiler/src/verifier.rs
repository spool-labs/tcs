//! Schema verification for TCS

use std::collections::{HashMap, HashSet};
use tcs_schema::{Definition, DefinitionKind, Schema};

use crate::error::TcsError;
use crate::utils::quote;

/// Reserved type names that cannot be used
pub const RESERVED_NAMES: &[&str] = &["ByteBuffer", "package"];

/// Native/builtin types
pub const NATIVE_TYPES: &[&str] = &[
    "bool", "byte", "int", "uint", "float", "string", "int64", "uint64",
];

/// Verify a schema for correctness
pub fn verify_schema(schema: &Schema) -> Result<(), TcsError> {
    let mut defined_types: HashSet<String> = NATIVE_TYPES.iter().map(|s| s.to_string()).collect();
    let mut definitions_map: HashMap<String, &Definition> = HashMap::new();

    // 1) Check duplicate / reserved type names
    for def in &schema.definitions {
        if defined_types.contains(&def.name) {
            return Err(TcsError::VerificationError(format!(
                "The type {} is defined twice",
                quote(&def.name)
            )));
        }
        if RESERVED_NAMES.contains(&def.name.as_str()) {
            return Err(TcsError::VerificationError(format!(
                "The type name {} is reserved",
                quote(&def.name)
            )));
        }
        defined_types.insert(def.name.clone());
        definitions_map.insert(def.name.clone(), def);
    }

    // 2) Check fields inside each non-enum definition
    for def in &schema.definitions {
        if let DefinitionKind::Enum = def.kind {
            continue;
        }
        if def.fields.is_empty() {
            continue;
        }

        // Check that each field's type is defined
        for field in &def.fields {
            if let Some(ref ty) = field.type_ {
                if !defined_types.contains(ty) {
                    return Err(TcsError::VerificationError(format!(
                        "The type {} is not defined for field {}",
                        quote(ty),
                        quote(&field.name)
                    )));
                }
            }

            // Check that fixed-size arrays only use byte type
            if let Some(size) = field.array_size {
                if field.type_.as_deref() != Some("byte") {
                    return Err(TcsError::VerificationError(format!(
                        "Fixed-size arrays are only supported for byte type, not {} in field {}",
                        quote(field.type_.as_deref().unwrap_or("unknown")),
                        quote(&field.name)
                    )));
                }
                if size == 0 {
                    return Err(TcsError::VerificationError(format!(
                        "Fixed-size array cannot have size 0 in field {}",
                        quote(&field.name)
                    )));
                }
            }
        }

        // Check field_id uniqueness and bounds
        let mut values = HashSet::new();
        for field in &def.fields {
            if values.contains(&field.field_id) {
                return Err(TcsError::VerificationError(format!(
                    "The id for field {} is used twice",
                    quote(&field.name)
                )));
            }
            if field.field_id <= 0 {
                return Err(TcsError::VerificationError(format!(
                    "The id for field {} must be positive",
                    quote(&field.name)
                )));
            }
            if field.field_id > def.fields.len() as i32 {
                return Err(TcsError::VerificationError(format!(
                    "The id for field {} cannot be larger than {}",
                    quote(&field.name),
                    def.fields.len()
                )));
            }
            values.insert(field.field_id);
        }
    }

    // 3) Check that structs do not contain themselves recursively
    let mut state: HashMap<String, u8> = HashMap::new();

    fn check_recursion(
        name: &str,
        definitions_map: &HashMap<String, &Definition>,
        state: &mut HashMap<String, u8>,
    ) -> Result<(), TcsError> {
        let definition = match definitions_map.get(name) {
            Some(def) => def,
            None => return Ok(()),
        };
        if let DefinitionKind::Struct = definition.kind {
            if let Some(&s) = state.get(name) {
                if s == 1 {
                    return Err(TcsError::VerificationError(format!(
                        "Recursive nesting of {} is not allowed",
                        quote(name)
                    )));
                } else if s == 2 {
                    return Ok(());
                }
            }
            state.insert(name.to_string(), 1);
            for field in &definition.fields {
                // Arrays are allowed to be recursive (they break the recursion)
                if !field.is_array {
                    if let Some(ref ty) = field.type_ {
                        check_recursion(ty, definitions_map, state)?;
                    }
                }
            }
            state.insert(name.to_string(), 2);
        }
        Ok(())
    }

    for def in &schema.definitions {
        check_recursion(&def.name, &definitions_map, &mut state)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_schema;
    use crate::tokenizer::tokenize_schema;

    fn verify(input: &str) -> Result<(), TcsError> {
        let tokens = tokenize_schema(input)?;
        let schema = parse_schema(&tokens)?;
        verify_schema(&schema)
    }

    #[test]
    fn test_valid_schema() {
        let input = r#"
            struct Point {
                int x;
                int y;
            }
        "#;
        assert!(verify(input).is_ok());
    }

    #[test]
    fn test_duplicate_type() {
        let input = r#"
            struct Foo { int x; }
            struct Foo { int y; }
        "#;
        let err = verify(input).unwrap_err();
        assert!(matches!(err, TcsError::VerificationError(_)));
    }

    #[test]
    fn test_undefined_type() {
        let input = r#"
            struct Foo {
                Unknown x;
            }
        "#;
        let err = verify(input).unwrap_err();
        assert!(matches!(err, TcsError::VerificationError(_)));
    }

    #[test]
    fn test_fixed_array_only_byte() {
        let input = r#"
            struct Bad {
                int[32] values;
            }
        "#;
        let err = verify(input).unwrap_err();
        assert!(matches!(err, TcsError::VerificationError(_)));
    }

    #[test]
    fn test_fixed_byte_array_ok() {
        let input = r#"
            struct Good {
                byte[32] hash;
            }
        "#;
        assert!(verify(input).is_ok());
    }
}
