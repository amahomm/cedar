# Test Pair 01: Basic Entities (Minimal)

## Feature Coverage
- Simple `instance` declarations
- No attributes, no parents, no tags
- Multiple entity types

## Notes
- Bare `instance Type::"id";` with no record, parents, or tags
- Each declaration produces an entity with empty attrs, empty parents, no tags
- The absence of `tags` key in the JSON output is acceptable (serialization may omit it when empty)
