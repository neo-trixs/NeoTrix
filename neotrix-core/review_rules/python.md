# Python Code Review Rules

## Safety
- All SQL queries must use parameterized queries (avoid f-string/format in SQL)
- Never use pickle.load() on untrusted data
- Avoid eval()/exec() with user-supplied input
- Validate deserialized YAML (prefer safe_load over load)

## Type Hints
- All public functions must have type annotations
- Use Optional[T] instead of default None without type
- Enable mypy strict mode in CI
- Use Protocol for structural subtyping

## Async
- Never pass raw coroutines; always await them
- Use asyncio.gather() with error handling for concurrent tasks
- Avoid mixing asyncio and threading in the same module
- Use anyio/trio for structured concurrency

## Performance
- Prefer list comprehensions over map()/filter() with lambdas
- Use generator expressions for large sequences
- Prefer local imports inside hot functions only when necessary

## Style
- Follow PEP 8 (ruff/flake8 compliant)
- Use pathlib over os.path
- Use f-strings over % formatting or .format()
