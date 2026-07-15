# Test Pair 08: Tags

## Feature Coverage
- `tags` clause with string values
- `tags` clause with entity reference values
- `tags` clause combined with attributes and parents
- Tags with extension values
- Empty tags

## Notes
- `tags { ... }` produces a `"tags"` key in the JSON entity object
- Tag values use the same encoding as attribute values (`__entity`, `__extn` escapes)
- Tags can appear without attributes: `instance X::"y" tags { ... };`
- Tags can appear with both attributes and parents: ordering is `in ... = { } tags { };`
- Empty tags `tags {}` → `"tags": {}` (explicitly present but empty)
- Entities without `tags` clause don't have a `"tags"` key in JSON (or it's omitted)
- Tag keys are always strings (like record keys)
