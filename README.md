# Rundell

![Rundell logo](graphics/full_logo.png)

Rundell is a strongly-typed, structured, imperative, interpreted programming language designed to be English-readable. Every statement reads almost like plain prose, making programs easy to follow even for people who are new to programming.

```
define name as string = "World".
print "Hello, " + name + "!" + newline().
```

---

## Repository layout

```
Rundell-lang/
├── rundell/          ← Rust interpreter (Cargo workspace)
│   ├── crates/
│   │   ├── rundell-lexer/
│   │   ├── rundell-parser/
│   │   ├── rundell-interpreter/
│   │   └── rundell-cli/
│   ├── docs/
│   │   ├── TUTORIAL.md
│   │   └── LANGUAGE_REFERENCE.md
│   ├── examples/     ← ready-to-run .run programs
│   └── tests/        ← integration test .run files
├── Design/           ← language specification documents
└── graphics/         ← logos and artwork
```

---

## Quick Start

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [rustup.rs](https://rustup.rs).

### 2. Build

```bash
cd rundell
cargo build --release
```

The binary is written to `rundell/target/release/rundell` (or `rundell.exe` on Windows).

### 3. Run a program

```bash
./target/release/rundell examples/hello_world.run
```

### 4. Start the REPL

```bash
./target/release/rundell
```

```
Rundell 0.1.0  —  type 'exit' or Ctrl+D to quit
> print "Hello!" + newline().
Hello!
```

---

## Documentation

| Document | Description |
|---|---|
| [Tutorial](rundell/docs/TUTORIAL.md) | Step-by-step introduction for new users |
| [Language Reference](rundell/docs/LANGUAGE_REFERENCE.md) | Complete language specification with examples |
| [Examples](rundell/examples/) | Ready-to-run sample programs |

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

## Building and Testing

```bash
cd rundell

cargo build --workspace          # debug build
cargo build --release            # optimised release binary
cargo test --workspace           # run all unit and integration tests
cargo clippy -- -D warnings      # lint
cargo fmt                        # format
```

---

## Licence

See [LICENSE](LICENSE) for details.
