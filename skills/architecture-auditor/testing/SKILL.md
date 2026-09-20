---
name: testing
description: "Comprehensive testing strategy — TDD, integration testing, property testing, and test architecture"
version: "1.0.0"
author: "NeoTrix"
triggers: "test|testing|integration test|property test|mock|fixture|coverage|spec|qa"
---

# Testing — Comprehensive Test Strategy

Systematic approach to testing across all layers of the architecture. Covers unit tests, integration tests, property-based testing, and test architecture decisions.

## Testing Pyramid

```
        /  E2E  \        ← few, slow, high confidence
       / Integration \   ← moderate, test boundaries
      /   Unit Tests   \ ← many, fast, focused
```

## Test Categories

### Unit Tests
- **Scope**: Single function or method
- **Speed**: < 10ms per test
- **Isolation**: No network, no disk, no external services
- **Pattern**: Arrange → Act → Assert
- **Naming**: `test_<unit>_<scenario>_<expected>`

### Integration Tests
- **Scope**: Multiple components working together
- **Speed**: < 1s per test
- **Isolation**: Use test containers or in-memory substitutes
- **Pattern**: Given → When → Then
- **Boundaries**: Database, filesystem, network, external APIs

### Property-Based Tests
- **Scope**: Invariants that hold for all inputs
- **Tool**: `proptest`, `quickcheck`, `arbtest`
- **Pattern**: Define properties, let framework generate inputs
- **Use cases**: Parsers, serializers, data transformations, algorithms

### Contract Tests
- **Scope**: API contracts between services
- **Pattern**: Consumer-driven contracts
- **Tool**: `pact`, schema validation

## Test Architecture Patterns

### Test Fixtures
```rust
// Shared test setup
fn setup_test_db() -> SqlitePool {
    SqlitePool::connect(":memory:").unwrap()
}

// Scoped cleanup
#[test]
fn test_with_temp_dir() {
    let dir = tempfile::tempdir().unwrap();
    // test code...
} // dir dropped, cleanup automatic
```

### Test Doubles
| Type | Purpose | When to Use |
|------|---------|-------------|
| Stub | Return canned data | Dependency not worth testing |
| Mock | Verify interactions | Testing collaboration |
| Spy | Record calls | Checking call patterns |
| Fake | Working simplified impl | Integration tests |

### Test Data Builders
```rust
fn user_builder() -> UserBuilder {
    UserBuilder::default()
        .name("test-user")
        .email("test@example.com")
}
```

## Coverage Strategy

| Layer | Target | Focus |
|-------|--------|-------|
| Core logic | > 90% | Business rules, invariants |
| Adapters | > 70% | Edge cases, error paths |
| UI | > 50% | Critical user flows |
| Scripts/tools | > 60% | Happy paths |

## Anti-Patterns

- **Test coverage gambling**: High coverage ≠ quality tests
- **Mock hell**: Mocking everything makes tests fragile
- **Test interdependence**: Tests must run in any order
- **Brittle assertions**: Testing implementation details
- **Slow feedback**: Tests taking > 10s break flow

## Integration Points

- **tdd/**: Red-Green-Refactor micro-loop
- **architecture-auditor**: Test health metrics (coverage, flakiness)
- **CI/CD**: Tests gate deployments
- **Observability**: Test results feed metrics pipeline
