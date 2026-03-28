# Rundell

Rundell is a strongly-typed, structured, imperative, interpreted programming language designed to be English-readable. Every statement reads almost like plain prose, making programs easy to follow even for people who are new to programming.

```
define name as string = "World".
print "Hello, " + name + "!" + newline().
```

---

## Contents

- [Quick Start](#quick-start)
- [Building from Source](#building-from-source)
- [Running Programs](#running-programs)
- [REPL](#repl)
- [Language at a Glance](#language-at-a-glance)
- [Further Reading](#further-reading)

---

## Quick Start

```bash
# Clone and build
git clone <repo-url>
cd rundell
cargo build --release

# Run the hello world example
./target/release/rundell examples/hello_world.run
```

---

## Building from Source

Requires Rust stable (1.70 or later). Install it from [rustup.rs](https://rustup.rs).

```bash
cargo build --workspace        # debug build
cargo build --release          # optimised release binary
cargo test --workspace         # run all tests
```

The release binary is written to `target/release/rundell` (or `rundell.exe` on Windows).

---

## Running Programs

```bash
rundell my_program.run
```

Rundell source files use the `.run` extension and must be UTF-8 encoded.

---

## REPL

Launch the interactive Read-Eval-Print Loop by running `rundell` with no arguments:

```
$ rundell
Rundell 0.1.0  —  type 'exit' or Ctrl+D to quit
> define x as integer = 10.
> print string(x * 2) + newline().
20
```

Multi-line input is supported — the REPL accumulates lines until it sees a statement terminator (`.`) or a closing `<--`.

---

## Language at a Glance

| Feature | Syntax |
|---|---|
| Variable declaration | `define age as integer = 21.` |
| Assignment | `set age = age + 1.` |
| Print | `print "Hello" + newline().` |
| Input | `receive name with prompt "Your name: ".` |
| Conditional | `if (age >= 18) --> ... <--` |
| Switch | `switch grade --> A : ... else : ... <--` |
| For loop | `for i loops (1, 10, 1) --> ... <--` |
| While loop | `while count > 0 --> ... <--` |
| For each | `for each item in list --> ... <--` |
| Function | `define add(a as integer, b as integer) returns integer --> ... <--` |
| Error handling | `try --> ... catch (TypeError) --> ... finally --> ... <--` |
| Module import | `import "myModule".` |

---

## Further Reading

- [**Tutorial**](docs/TUTORIAL.md) — step-by-step introduction for new users
- [**Language Reference**](docs/LANGUAGE_REFERENCE.md) — complete specification with examples
- [**Examples**](examples/) — ready-to-run sample programs
