# nt_world_dom_agent — Flat DOM Tree Agent

## Motivation

The `nt_world_browse` module has browser automation (BrowserCircuit, session management,
stealth fingerprinting, humanization) but **zero DOM agent capability**. The page-agent
project (23.9k⭐, [github.com/page-agent/page-agent](https://github.com/page-agent/page-agent))
uses FlatDomTree to compress interactive DOM elements into a flat text map with stable
indices, enabling LLM agents to perceive and interact with web pages.

NeoTrix needs this same capability for its autonomous browsing, scraping, and world
interaction pipelines.

## Design

### Architecture

```
HTML string
    │
    ▼
DomAgent::compress_dom()
    │
    ├─ HtmlScanner — lightweight tag scanner (no external parser)
    ├─ Attribute parser (id, class, name, type, href, role, etc.)
    ├─ Interactive element filter (INTERACTIVE_TAGS + role/contenteditable/tabindex)
    └─ Text extraction (HTML-decoded, whitespace-normalized)
    │
    ▼
FlatDomTree
    ├─ elements: Vec<DomElement>  ← stable-indexed, document-order
    ├─ count: usize
    ├─ get(index) → Option<&DomElement>
    └─ to_text_map() → "flat text map" string
```

### Stable Index Guarantee

Indices are assigned by a simple counter (`elemidx`) in document order during a
single left-to-right scan. The same HTML input always produces the same output,
so indices are stable across compression runs as long as the DOM hasn't changed.

### Interactive Element Detection

| Category | Tags / Conditions |
|----------|------------------|
| Native interactive | `a`, `button`, `input`, `select`, `textarea`, `label`, `option`, `details`, `summary`, `menuitem` |
| Role-mapped | `div`/`span` with `role="button"`, `role="link"`, `role="option"`, `role="tab"`, `role="menuitem"`, `role="combobox"` |
| Editable | `contenteditable="true"` |
| Focusable | `tabindex >= 0` |
| Special input types | `type="checkbox"`, `type="radio"`, `type="submit"`, `type="button"`, `type="range"`, `type="file"` |

### Text Map Format

```
[0]<button> "Submit" [id="submit"]
[1]<a> "Next Page" [href="/next"]
[2]<input> [type="text" name="search" placeholder="Search..."]
[3]<select> [name="country"]
[4]<option> "US" [value="en"]
[5]<textarea> "Hello world" [name="bio"]
```

## Implementation

- **File**: `nt_world_dom_agent.rs` (self-contained, zero external deps — only `std`)
- **Test count**: 15 tests covering empty DOM, buttons, links, inputs, selects,
  textareas, stable indices, script/style skipping, entity decoding, role attributes,
  text map format, index lookup, checkboxes, self-closing tags, nested elements

## Future Work

- Add `click(index)` / `type(index, text)` / `select(index, value)` action methods
- Add attribute-aware filtering by `id`, `name`, `type`, `placeholder`
- Add visibility/offset computation for viewport-aware interaction
- Integrate with `nt_world_browse::BrowserCircuit` for real page interaction
- Support `aria-*` attributes for accessibility-aware navigation
