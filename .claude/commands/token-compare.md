# Token Efficiency Comparison

Compare token counts between equivalent programs in LLML and other languages.

## Arguments
- `$ARGUMENTS`: A program description (e.g., "fibonacci function") or a specific LLML file to compare

## Instructions

1. **Identify the program**: If a file is provided, read it. Otherwise, write an LLML implementation of the described program.

2. **Write equivalents**: Create equivalent implementations in:
   - Python
   - Rust
   - TypeScript/JavaScript

3. **Count tokens**: For each version, use `python3` with a rough BPE token estimate:
   ```python
   # Rough estimate: split on whitespace and punctuation boundaries
   import re
   tokens = re.findall(r'\w+|[^\w\s]', source)
   print(f"{len(tokens)} tokens")
   ```

4. **Compare**: Present a table:
   | Language | Tokens | Lines | Characters |
   |----------|--------|-------|------------|
   | LLML     | ...    | ...   | ...        |
   | Python   | ...    | ...   | ...        |
   | Rust     | ...    | ...   | ...        |
   | TypeScript | ... | ...   | ...        |

5. **Analysis**: Explain where LLML saves tokens and why.
