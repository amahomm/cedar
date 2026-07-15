# Test Pair 06: Extension Function Calls

## Feature Coverage
- `ip()` extension function
- `decimal()` extension function
- `datetime()` extension function
- `duration()` extension function
- Extension values in attributes
- Extension values inside sets
- Extension values inside nested records

## Notes
- Extension calls: `fn("arg")` → `{ "__extn": { "fn": "fn", "arg": "arg" } }`
- The `fn` field in `__extn` is the function name (e.g., `ip`, `decimal`, `datetime`, `duration`)
- The `arg` field is always a string in JSON (the extension function parses it)
- Extension values can appear anywhere a value can: top-level attrs, inside sets, inside records
- Method calls like `ip("...").isLoopback()` are NOT valid per RFC #110
- Only constructor functions are valid (creating values, not computing on them)
