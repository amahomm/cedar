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

use std::path::Path;

use cedar_policy::{Entities, Schema};
use clap::ValueEnum;
use miette::{IntoDiagnostic, Result, WrapErr};

/// Format for entity data files
#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum EntitiesFormat {
    /// Cedar entity data syntax (RFC 104)
    #[cfg(feature = "cedar-entity-syntax")]
    Cedar,
    /// JSON entity format
    #[default]
    Json,
}

/// Load an `Entities` object from the given filename, format, and optional schema.
///
/// If format is `Json`, parses as JSON. If format is `Cedar`, parses as
/// Cedar entity data syntax.
pub(crate) fn load_entities(
    entities_filename: impl AsRef<Path>,
    format: EntitiesFormat,
    schema: Option<&Schema>,
) -> Result<Entities> {
    let path = entities_filename.as_ref();

    match format {
        #[cfg(feature = "cedar-entity-syntax")]
        EntitiesFormat::Cedar => load_cedar_entities(path, schema),
        EntitiesFormat::Json => load_json_entities(path, schema),
    }
}

/// Load entities from a JSON file
fn load_json_entities(path: &Path, schema: Option<&Schema>) -> Result<Entities> {
    let f = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to open entities file {}", path.display()))?;
    Entities::from_json_file(f, schema)
        .wrap_err_with(|| format!("failed to parse entities from file {}", path.display()))
}

/// Load entities from a Cedar entity data syntax file
#[cfg(feature = "cedar-entity-syntax")]
fn load_cedar_entities(path: &Path, schema: Option<&Schema>) -> Result<Entities> {
    let src = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to read entities file {}", path.display()))?;
    Entities::from_cedar_str(&src, schema)
        .wrap_err_with(|| format!("failed to parse entities from file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::{render_err, TEMPFILE_FILTER};
    use std::io::Write;

    #[test]
    fn error_on_invalid_entity_data() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"not valid json").unwrap();
        let err = load_entities(f.path(), EntitiesFormat::Json, None).unwrap_err();
        insta::with_settings!({filters => vec![TEMPFILE_FILTER]}, {
            insta::assert_snapshot!(render_err(&err), @r"
            × failed to parse entities from file <TEMPFILE>
            ├─▶ error during entity deserialization
            ╰─▶ expected ident at line 1 column 2
            ");
        });
    }

    #[test]
    fn error_on_ill_typed_entities() {
        let (schema, _) = Schema::from_cedarschema_str("entity Photo;").unwrap();
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(br#"[{"uid":{"__entity":{"type":"Album","id":"a"}},"attrs":{},"parents":[]}]"#)
            .unwrap();
        let err = load_entities(f.path(), EntitiesFormat::Json, Some(&schema)).unwrap_err();
        insta::with_settings!({filters => vec![TEMPFILE_FILTER]}, {
            insta::assert_snapshot!(render_err(&err), @r#"
            × failed to parse entities from file <TEMPFILE>
            ├─▶ error during entity deserialization
            ╰─▶ entity `Album::"a"` has type `Album` which is not declared in the schema
            "#);
        });
    }

    #[test]
    fn error_on_missing_entity_file() {
        let err =
            load_entities("/tmp/nonexistent_cedar_test_file.json", EntitiesFormat::Json, None)
                .unwrap_err();
        insta::assert_snapshot!(render_err(&err), @r"
        × failed to open entities file /tmp/nonexistent_cedar_test_file.json
        ╰─▶ No such file or directory (os error 2)
        ");
    }

    #[cfg(feature = "cedar-entity-syntax")]
    #[test]
    fn load_cedar_entities_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(br#"instance User::"alice"; instance User::"bob";"#)
            .unwrap();
        let entities = load_entities(f.path(), EntitiesFormat::Cedar, None).unwrap();
        assert_eq!(entities.iter().count(), 2);
    }

    #[cfg(feature = "cedar-entity-syntax")]
    #[test]
    fn error_on_invalid_cedar_entities() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"this is not valid cedar entity syntax $$$$")
            .unwrap();
        let err = load_entities(f.path(), EntitiesFormat::Cedar, None).unwrap_err();
        let rendered = render_err(&err);
        assert!(
            rendered.contains("failed to parse entities from file"),
            "Expected error message, got: {rendered}"
        );
    }
}
