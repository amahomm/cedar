# Test Pair 07: Namespaces

## Feature Coverage
- `namespace` block wrapping entity instances
- Nested namespace paths (e.g., `Auth::IAM`)
- Fully qualified entity types inside namespaces
- References to entities in other namespaces
- Mix of namespaced and unnamespaced declarations

## Notes
- `namespace Foo { instance Bar::"x"; }` → entity type becomes `Foo::Bar`
- Nested namespaces: `namespace A::B { instance C::"x"; }` → type is `A::B::C`
- Entities inside a namespace can use short names for same-namespace types
- Cross-namespace references use fully qualified names: `PhotoApp::Album::"vacation"`
- Entities outside any namespace block use their type name as-is
- The JSON `type` field always contains the fully qualified type name
- Parent references inside namespaces are also namespace-resolved
