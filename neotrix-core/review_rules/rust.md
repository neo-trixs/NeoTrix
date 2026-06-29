# Rust Code Review Rules

## Safety
- All unsafe blocks must have a // SAFETY: comment explaining invariants
- Prefer safe abstractions over unsafe code
- Check for correct use of Pin, Send, Sync in async code

## Error Handling
- Use ? operator instead of unwrap()/expect() in production code
- Custom error types should implement std::error::Error
- Match all enum variants or use catch-all pattern

## Performance
- Avoid unnecessary clones (check .clone() calls)
- Prefer &str over String in function parameters
- Use Vec::with_capacity when size is known

## Concurrency
- Verify Send + Sync bounds on shared types
- Use Arc<Mutex<T>> or tokio::sync appropriately
- Avoid holding locks across .await points

## Style
- Follow rustfmt conventions
- Public API items must have doc comments
- Group imports: std → external → crate
