# Test Pair 03: Primitive Attributes

## Feature Coverage
- Record with string, integer, and boolean attribute values
- Both `=` and bare record syntax (optional `=`)
- Empty record explicitly specified
- Negative integers (prefix `-`)

## Notes
- `= { ... }` and bare `{ ... }` (without `=`) produce identical output
- String values map directly to JSON strings
- Integer values map directly to JSON numbers (64-bit signed)
- Boolean values `true`/`false` map directly to JSON booleans
- Empty record `{}` produces empty attrs object
- Negative numbers: `-1` in Cedar → `-1` in JSON
