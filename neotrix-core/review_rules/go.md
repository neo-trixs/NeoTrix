# Go Code Review Rules

## Error Handling
- Never ignore returned errors; always check them
- Use errors.Is() / errors.As() for sentinel/type error matching
- Return early on error; avoid deep if-else nesting
- Wrap errors with fmt.Errorf("context: %w", err)

## Concurrency
- Never leak goroutines; ensure they terminate on context cancel
- Pass context.Context as the first parameter in public functions
- Never copy sync.Mutex or sync.RWMutex by value
- Use sync.Map only for specialized hot-path access

## Interfaces
- Prefer small interfaces (1-3 methods, io.Reader style)
- Accept interfaces, return structs
- Define interfaces where they are consumed, not produced

## Cleanup
- Always use defer for resource cleanup (files, locks, connections)
- Defer in LIFO order; be aware of execution ordering
- Close http.Response.Body even on error paths

## Style
- Follow gofmt/golint conventions
- Avoid init() functions except for registration patterns
- Use table-driven tests with t.Run subtests
