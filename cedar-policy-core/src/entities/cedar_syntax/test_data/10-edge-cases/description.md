# Test Pair 10: Edge Cases and Special Characters

## Feature Coverage
- Entity IDs with special characters (spaces, unicode, escapes)
- String values with escape sequences
- Attribute keys that look like reserved words
- Maximum integer values (i64 boundaries)
- Single-element sets and records

## Notes
- Empty entity IDs (`""`) are valid in Cedar
- String escape sequences follow Rust/JSON conventions: `\t`, `\n`, `\"`, `\\`, `\u{...}`
- Unicode escapes `\u{1F600}` should be decoded to the actual character in output
- Attribute keys are quoted strings so reserved words are valid as keys
- Cedar Long is i64: range `-9223372036854775808` to `9223372036854775807`
- Single-element sets `[42]` and single-element records `{ "k": "v" }` are valid
- JSON numeric precision: i64 values must be preserved exactly
