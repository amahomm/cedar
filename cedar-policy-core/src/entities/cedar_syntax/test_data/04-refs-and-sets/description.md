# Test Pair 04: Entity References and Sets

## Feature Coverage
- Entity reference as attribute value
- Set (array) of primitives
- Set of entity references
- Empty set
- Nested sets

## Notes
- Entity refs in attributes: `User::"alice"` → `{ "__entity": { "type": "User", "id": "alice" } }`
- Sets of entity refs: `[User::"a", User::"b"]` → array of `__entity` objects
- Sets of primitives: `["x", "y"]` → JSON array of strings
- Empty sets: `[]` → `[]`
- Nested sets: `[[1, 2], [3, 4]]` → nested JSON arrays
- Entity refs in `in` clause produce `parents` array; entity refs in attributes produce `__entity` escapes
