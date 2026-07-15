# Test Pair 02: Hierarchy (Parents)

## Feature Coverage
- `in` clause with single parent
- `in` clause with multiple parents (bracket syntax)
- Parent references to different entity types
- Mix of entities with and without parents

## Notes
- Single parent: `in ParentType::"id"` → `"parents": [{ "type": "ParentType", "id": "id" }]`
- Multiple parents: `in [A::"x", B::"y"]` → `"parents": [{ "type": "A", "id": "x" }, { "type": "B", "id": "y" }]`
- Parent order in JSON should match declaration order
- Entities don't need to be declared before being referenced as parents
