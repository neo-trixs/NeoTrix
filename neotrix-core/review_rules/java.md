# Java Code Review Rules

## Null Safety
- Prefer Optional over nullable returns in public APIs
- Use @Nullable/@NonNull annotations on public method parameters
- Use Objects.requireNonNull() for constructor DI parameters
- Never return null from a Collection-returning method; return empty collection

## Streams & Lambdas
- Prefer Stream API for collection transformations; loops for complex logic
- Avoid side-effects in stream lambdas (forEach with mutation)
- Use method references over lambdas where applicable
- Prefer primitive streams (IntStream, LongStream) for numeric operations

## Exception Handling
- Prefer unchecked exceptions for programming errors
- Never swallow exceptions in empty catch blocks
- Use try-with-resources for all Closeable resources
- Log exceptions at the boundary, not at every layer

## Thread Safety
- Prefer ConcurrentHashMap over synchronized HashMap
- Use synchronized blocks on private lock objects, not on public references
- Prefer java.util.concurrent locks over raw synchronized where fairness matters
- Never call foreign methods while holding a lock

## Style
- Follow Google Java Style or project conventions
- Minimize Lombok usage; @Data and @Builder are preferred
- Prefer constructor injection over field injection in Spring
