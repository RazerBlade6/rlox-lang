# Test cases

Each test is a script plus a `.expected` file with the exact stdout it should produce. They're run by `tests/lox.rs`.

| File | Run as |
|---|---|
| `name.rlox` | `rlox-lang name.rlox` (file mode) |
| `name.repl` | piped into the REPL line by line (use this for REPL-only bugs such as R3) |
| `name.expected` | expected stdout for either of the above |

- Subfolders are fine (e.g. `scanner/`, `parser/`, `resolver/`), and the folder appears in the test name.
- Lox errors are printed to stdout, so error cases are tested the same way: put the error message in `.expected`.
- REPL output includes the banner and `>>> ` prompts. Easiest is to bless it (see below) and then check the result.
- Crashes (panics, stack overflows) and timeouts always fail, whatever `.expected` says.

```sh
cargo test                            # everything
cargo test --test lox -- resolver     # only names containing "resolver"
RLOX_BLESS=1 cargo test --test lox    # record actual output into .expected files
RLOX_TIMEOUT=2 cargo test             # per-case timeout in seconds (default 5)
```

Blessing records what the interpreter does *now*. For a bug test, write `.expected` by hand with the correct output, so the test stays red until the bug is fixed.
