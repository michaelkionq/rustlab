# Notebook Future Features

## 5. Template interpolation in markdown cells

**Problem:** `fprintf` with `%,12.2f` comma formatting works in code output,
but notebook markdown/text cells cannot reference computed variables.  Users
want to embed computed values directly in prose (e.g. a summary paragraph
that includes `total_tax` formatted with commas).

**Proposed approach:**
- Add `${expr}` or `{{expr}}` template syntax in markdown blocks
- During notebook execution, evaluate interpolated expressions against the
  shared evaluator state
- Support format specifiers: `${total_tax:%,.2f}` for comma-grouped currency
- Keep raw markdown rendering for non-interpolated text (no performance cost
  when unused)

**Considerations:**
- Escaping: need `\${}` or `${{` escape for literal braces
- Error handling: undefined variable → show placeholder or error inline?
- Security: expressions are already sandboxed by the evaluator

---

## 6. Cell arrays / string arrays

**Problem:** `{'Jan','Feb','Mar',...}` syntax is not supported.  Users must
use `switch`/`case` blocks for string lookups that other tools handle with cell
arrays.

**Proposed approach — phased:**

### Phase 1: String arrays (minimal)
- Add `Value::StringArray(Vec<String>)` variant
- Parser: support `{"a", "b", "c"}` literal syntax (curly braces)
- Lexer: add `LBrace` / `RBrace` tokens
- Indexing: `sa(2)` → `"b"` (1-based)
- `length()`, `size()` support

### Phase 2: Heterogeneous cell arrays
- Add `Value::CellArray(Vec<Value>)` for mixed-type containers
- `{1, "hello", [1 2 3]}` — each element can be any Value
- `cell(n)` constructor for pre-allocated empty cell arrays
- `iscell()`, `cellfun()` builtins

### Phase 3: Cell array of strings as axis tick labels
- `bar(x, y)` with string array x-tick labels
- `plot` x-axis category support

**Scope:** This is a significant language addition touching lexer, parser,
AST, Value, and evaluator.  String arrays (Phase 1) cover ~80% of the
practical use cases with minimal complexity.
