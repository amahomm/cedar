# Test Pair 05: Nested Records

## Feature Coverage
- Record-valued attributes (nested records)
- Deeply nested records (3 levels)
- Records containing entity refs, sets, and other records
- Mix of attribute types at different nesting levels

## Notes
- Nested records in Cedar `{ "key": { ... } }` map directly to nested JSON objects
- Entity refs inside nested records still use `__entity` escape
- Sets inside nested records still produce JSON arrays
- Three levels of nesting: `preferences.notifications.channels`
- Records can contain heterogeneous values at any level
