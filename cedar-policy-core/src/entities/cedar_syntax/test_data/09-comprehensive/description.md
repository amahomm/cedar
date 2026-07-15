# Test Pair 09: Comprehensive Realistic Scenario

## Feature Coverage
- All features combined in a realistic application model
- Multiple namespaces
- Deep hierarchies
- Complex attribute types
- Extension values for network/time data
- Tags for metadata
- Comments (should be ignored by parser)

## Notes
- This test combines all previously tested features in a realistic application model
- Comments (`//`) in the Cedar syntax are ignored and don't appear in JSON output
- Multiple namespace blocks produce entities with different type prefixes
- Cross-namespace entity references are fully qualified in both Cedar syntax and JSON
- Action hierarchy uses the same `in` mechanism as data entities
- Validates that the parser handles real-world complexity correctly
