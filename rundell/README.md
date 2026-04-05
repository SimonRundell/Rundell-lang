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
- [What's New in 0.1.5](#whats-new-in-015)
- [What's New in 0.1.4](#whats-new-in-014)
- [What's New in 0.1.3](#whats-new-in-013)

---

## Quick Start

```bash
# Clone and build
git clone <repo-url>
cd rundell
cargo build --release

# Run the hello world example
./target/release/rundell examples/hello_world.run

# Run a GUI example (auto-launches GUI when forms are used)
./target/release/rundell examples/gui_hello.run

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

GUI programs can be run with `rundell` directly. The CLI auto-launches the GUI
runtime when a program uses forms or dialogs. You can still run the GUI binary
explicitly if you prefer:

```bash
# Run a GUI program via CLI (auto GUI)
rundell my_form.run

# Run a GUI program explicitly
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
Rundell 0.1.5  —  type 'exit' or Ctrl+D to quit
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
| File I/O | `define text as string = read_text("notes.txt").` |

### GUI system

| Feature | Syntax |
|---|---|
| Define a form | `define myForm as form --> ... <--` |
| Add a control | `define myLabel as form\label.` |
| Set a property | `set myLabel\value = "Hello".` |
| Set position | `set myLabel\position = 10px, 10px, 200px, 30px.` *(top, left, width, height)* |
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
| [`examples/datetime_basics.run`](examples/datetime_basics.run) | Datetime literals, arithmetic, and formatting |
| [`examples/execute_basics.run`](examples/execute_basics.run) | Execute external programs via PATH or relative paths |
| [`examples/execute_cross_platform.run`](examples/execute_cross_platform.run) | Execute external programs on different OSes |
| [`examples/contact_book.run`](examples/contact_book.run) | In-memory contact list |
| [`examples/error_handling_demo.run`](examples/error_handling_demo.run) | try/catch/finally patterns |
| [`examples/file_io_text_json.run`](examples/file_io_text_json.run) | Read/write UTF-8 text and JSON |
| [`examples/file_io_csv.run`](examples/file_io_csv.run) | Read/write CSV data |
| [`examples/timer_basic.run`](examples/timer_basic.run) | Event timer (headless) demo |

### GUI programs (run with `rundell-gui`)

| File | Description |
|---|---|
| [`examples/gui_hello.run`](examples/gui_hello.run) | Greeting form — GUI Hello World |
| [`examples/gui_calculator.run`](examples/gui_calculator.run) | Calculator with form controls |
| [`examples/gui_temperature.run`](examples/gui_temperature.run) | Temperature converter form |
| [`examples/gui_contact_book.run`](examples/gui_contact_book.run) | Contact book with listbox |
| [`examples/gui_dialogs.run`](examples/gui_dialogs.run) | Message box, open-file, save-file dialogs |
| [`examples/gui_timer.run`](examples/gui_timer.run) | Timer-driven GUI label updates |
| [`examples/gui_clock.run`](examples/gui_clock.run) | On-screen clock with event timer |

---

## Further Reading

- [**Tutorial**](docs/TUTORIAL.md) — step-by-step introduction for new users
- [**Language Reference**](docs/LANGUAGE_REFERENCE.md) — complete specification with examples
- [**Examples**](examples/) — ready-to-run sample programs (console and GUI)

---

## What's New in 0.1.4

- Named event timers (`eventtimer`) with `interval`, `running`, and `event`
- Duration literals for timers: `500ms`, `2s`, `1m`, `1h`
- Headless timer dispatch in CLI mode
- GUI example: timer-driven label updates
- GUI controls now support `font` and `fontsize`
- Designer: Select controls can edit and emit `items`

## What's New in 0.1.5

- New `datetime` type with ISO 8601 literals (pipe-delimited)
- Datetime built-ins: `now`, `day`, `month`, `year`, `hour`, `minute`, `second`, `dateformat`
- Datetime arithmetic support (`datetime +/- integer`, `datetime - datetime`)
- Execute external programs with `execute(path)`
- New examples: datetime basics, execute basics, and GUI clock

## What's New in 0.1.3

- Auto GUI launch from `rundell` when forms/dialogs are used
- File I/O built-ins for UTF-8 text, JSON, and CSV
- GUI control `textalign` property (left/center/right)
- Designer enhancements: tabs, editor, file menu, events, undo/delete
- Form runtime fixes: proper control positioning and live input updates
- Dialogs now use native OS dialogs for file/message boxes

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
