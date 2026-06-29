# TypeScript Code Review Rules

## Type Safety
- Never use `any`; prefer `unknown` and narrow with type guards
- Enable strict mode in tsconfig.json always
- Use branded types for IDs (type UserID = string & { readonly brand: unique symbol })
- Prefer `as const` and satisfies over type assertions

## Async & Error Handling
- Always await promises; avoid floating promises
- Wrap async route handlers with error boundary middleware
- Never ignore .catch() on Promise return values
- Use Either/Result types for recoverable errors

## React/Hooks
- List all dependencies in useEffect/useCallback deps arrays
- Never call hooks conditionally or inside loops
- Extract complex state to useReducer, not multiple useState
- Prefer React Query/SWR over manual useEffect fetching

## State Management
- Use discriminated unions for state machines (type State = Idle | Loading | Success<T> | Error)
- Avoid deeply nested state; normalize when possible
- Prefer immutable updates with spread or Immer

## Style
- Prefer named exports over default exports
- Use barrel files (index.ts) to control public API surface
- Group imports: external → internal → relative
