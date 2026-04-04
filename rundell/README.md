# Rundell

Rundell is a strongly-typed, structured, imperative, interpreted programming language designed to be English-readable. Every statement reads almost like plain prose, making programs easy to follow even for people new to programming.

```
define name as string = "World".
print "Hello, " + name + "!" + newline().
```

Rundell also has a full graphical UI system. Programs can open windows, collect input, and react to button clicks — all written in the same readable style.

```
define myForm as form -->
    set form\title = "Greeting".
    define nameBox as form\textbox.
    define okBtn   as form\button.
    set nameBox\position = 10px, 10px, 300px, 28px.
    set nameBox\placeholder = "Enter your name".
    set okBtn\position  = 10px, 50px, 100px, 30px.
    set okBtn\caption   = "Say Hello".
    set okBtn\click     = greet().
<--

define greet() returns null -->
    print "Hello, " + myForm\nameBox\value + "!" + newline().
    return null.
<--

rootWindow\myForm\show().
```

---

## Contents

- [Quick Start](#quick-start)
- [Building from Source](#building-from-source)
- [Running Programs](#running-programs)
- [GUI Programs](#gui-programs)
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

# Run a GUI example
cargo run -p rundell-gui -- examples/gui_hello.run

# Open the visual form designer
cargo run -p rundell-gui -- --design
```

---

## Building from Source

Requires Rust stable (1.70 or later). Install it from [rustup.rs](https://rustup.rs).

```bash
cargo build --workspace        # debug build
cargo build --release          # optimised release binary
cargo test --workspace         # run all tests
```

The CLI binary is written to `target/release/rundell` (or `rundell.exe` on Windows).
The GUI binary is `target/release/rundell-gui`.

---

## Running Programs

```bash
rundell my_program.run
```

Rundell source files use the `.run` extension and must be UTF-8 encoded.

---

## GUI Programs

GUI programs use `rundell-gui` instead of `rundell`:

```bash
# Run a GUI program
cargo run -p rundell-gui -- my_form.run

# Open the visual form designer
cargo run -p rundell-gui -- --design
```

The form designer lets you build forms visually and generate the equivalent Rundell source code.

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

### Core language

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

### GUI system

| Feature | Syntax |
|---|---|
| Define a form | `define myForm as form --> ... <--` |
| Add a control | `define myLabel as form\label.` |
| Set a property | `set myLabel\value = "Hello".` |
| Set position | `set myLabel\position = 10px, 10px, 200px, 30px.` |
| Show (modeless) | `rootWindow\myForm\show().` |
| Show (modal) | `rootWindow\myForm\show(modal).` |
| Close | `rootWindow\myForm\close().` |
| Read a control | `print myForm\myLabel\value.` |
| Bind an event | `set myButton\click = handleClick().` |
| File dialog | `set path = dialog\openfile("Open", "*.run").` |
| Message box | `set ans = dialog\message("Q", "Sure?", yesno).` |

---

## Examples

### Console programs

| File | Description |
|---|---|
| [`examples/hello_world.run`](examples/hello_world.run) | Classic Hello World |
| [`examples/fizzbuzz.run`](examples/fizzbuzz.run) | FizzBuzz loop |
| [`examples/fibonacci.run`](examples/fibonacci.run) | Fibonacci sequence |
| [`examples/calculator.run`](examples/calculator.run) | Console integer calculator |
| [`examples/grade_calculator.run`](examples/grade_calculator.run) | Grade average and letter grade |
| [`examples/temperature_converter.run`](examples/temperature_converter.run) | Console °C ↔ °F converter |
| [`examples/string_utils.run`](examples/string_utils.run) | String manipulation demos |
| [`examples/contact_book.run`](examples/contact_book.run) | In-memory contact list |
| [`examples/error_handling_demo.run`](examples/error_handling_demo.run) | try/catch/finally patterns |

### GUI programs (run with `rundell-gui`)

| File | Description |
|---|---|
| [`examples/gui_hello.run`](examples/gui_hello.run) | Greeting form — GUI Hello World |
| [`examples/gui_calculator.run`](examples/gui_calculator.run) | Calculator with form controls |
| [`examples/gui_temperature.run`](examples/gui_temperature.run) | Temperature converter form |
| [`examples/gui_contact_book.run`](examples/gui_contact_book.run) | Contact book with listbox |
| [`examples/gui_dialogs.run`](examples/gui_dialogs.run) | Message box, open-file, save-file dialogs |

---

## Further Reading

- [**Tutorial**](docs/TUTORIAL.md) — step-by-step introduction for new users
- [**Language Reference**](docs/LANGUAGE_REFERENCE.md) — complete specification with examples
- [**Examples**](examples/) — ready-to-run sample programs (console and GUI)

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
