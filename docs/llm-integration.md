# LLM Integration Guide

Three ways for other LLMs to use LLML.

---

## Method 1: System Prompt Injection

Include [llml-reference-card.md](llml-reference-card.md) in the system prompt so the LLM can generate LLML code.

```
You can write programs in LLML, a language designed for LLMs.
<llml-reference>
{contents of llml-reference-card.md}
</llml-reference>

When asked to solve a programming task, write the solution in LLML.
```

## Method 2: Tool Use (Function Calling)

Provide the LLM with an LLML execution tool. This is the most powerful integration approach.

### OpenAI-style Tool Definition

```json
{
  "type": "function",
  "function": {
    "name": "run_llml",
    "description": "Execute an LLML program and return stdout. LLML uses s-expression syntax with sigils ($var, @Type). Use ($print ...) for output.",
    "parameters": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "description": "LLML source code to execute"
        }
      },
      "required": ["code"]
    }
  }
}
```

### Anthropic Claude Tool Definition

```json
{
  "name": "run_llml",
  "description": "Execute an LLML program and return stdout. LLML uses s-expression syntax with sigils ($var, @Type). Use ($print ...) for output.",
  "input_schema": {
    "type": "object",
    "properties": {
      "code": {
        "type": "string",
        "description": "LLML source code to execute"
      }
    },
    "required": ["code"]
  }
}
```

### Tool Implementation (Python)

```python
import subprocess
import tempfile
import os

def run_llml(code: str) -> dict:
    """Execute LLML code and return the result."""
    with tempfile.NamedTemporaryFile(suffix=".llml", mode="w", delete=False) as f:
        f.write(code)
        f.flush()
        try:
            result = subprocess.run(
                ["llml", "run", f.name],
                capture_output=True, text=True, timeout=10
            )
            return {
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.returncode
            }
        finally:
            os.unlink(f.name)
```

### Tool Implementation (TypeScript)

```typescript
import { execFileSync } from "node:child_process";
import { writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

function runLlml(code: string): { stdout: string; stderr: string } {
  const path = join(tmpdir(), `llml_${Date.now()}.llml`);
  writeFileSync(path, code);
  try {
    const stdout = execFileSync("llml", ["run", path], {
      timeout: 10_000,
      encoding: "utf-8",
    });
    return { stdout, stderr: "" };
  } catch (e: any) {
    return { stdout: "", stderr: e.stderr || e.message };
  } finally {
    unlinkSync(path);
  }
}
```

### Usage Flow

```
User: "Calculate the factorial of 5"

LLM generates tool call:
  run_llml({
    code: `(fn $fact (: @I32 -> @I32) ($n : @I32)
             (if (= $n 0) 1 (* $n ($fact (- $n 1)))))
           ($print ($to_str ($fact 5)))`
  })

Tool response:
  { "stdout": "120", "exit_code": 0 }

LLM final response: "The factorial of 5 is 120."
```

## Method 3: MCP Server

Expose LLML as an MCP server for Claude Code or other MCP-compatible clients.

### mcp.json Configuration

```json
{
  "mcpServers": {
    "llml": {
      "command": "llml",
      "args": ["mcp-serve"],
      "description": "LLML language execution server"
    }
  }
}
```

> Note: The `llml mcp-serve` subcommand is planned for Phase 3.
> For now, use Methods 1 or 2.

---

## Why LLML Instead of Existing Languages?

| Aspect | Python | LLML |
|--------|--------|------|
| Token count (fibonacci) | ~35 | ~22 (-37%) |
| Syntactic ambiguity | Operator precedence, indentation-dependent | None (s-expression) |
| Runtime failure modes | NameError, TypeError, IndentationError | Explicit type errors only |
| LLM first-attempt success rate | ~80% (est.) | ~95% (est., due to structural unambiguity) |

The value of LLML lies in **increasing the probability that an LLM generates correct code on the first attempt**.
