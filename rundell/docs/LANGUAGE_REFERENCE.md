# Rundell Language Reference

This document is the complete reference for the Rundell language.

---

## Contents

1. [Source Files](#1-source-files)
2. [Statements and Terminators](#2-statements-and-terminators)
3. [Comments](#3-comments)
4. [Identifiers](#4-identifiers)
5. [Data Types](#5-data-types)
6. [Literals](#6-literals)
7. [Variables](#7-variables)
8. [Assignment](#8-assignment)
9. [Operators](#9-operators)
10. [Type Casting](#10-type-casting)
11. [Built-in Functions](#11-built-in-functions)
12. [Input and Output](#12-input-and-output)
13. [Conditionals](#13-conditionals)
14. [Loops](#14-loops)
15. [Functions](#15-functions)
16. [Collections (json)](#16-collections-json)
17. [Error Handling](#17-error-handling)
18. [Modules](#18-modules)
19. [GUI — Forms](#19-gui--forms)
20. [GUI — Controls](#20-gui--controls)
21. [GUI — Object Paths](#21-gui--object-paths)
22. [GUI — Events](#22-gui--events)
23. [GUI — System Dialogs](#23-gui--system-dialogs)
24. [GUI — The Form Designer](#24-gui--the-form-designer)
25. [REST — Credentials](#25-rest--credentials)
26. [REST — Query Definitions](#26-rest--query-definitions)
27. [REST — Calling Queries with await](#27-rest--calling-queries-with-await)
28. [REST — Error Handling with attempt/catch](#28-rest--error-handling-with-attemptcatch)
29. [REST — Credential Storage](#29-rest--credential-storage)
30. [Keywords](#30-keywords)
31. [Error Types](#31-error-types)

---

## 1. Source Files

- Extension: `.run`
- Encoding: UTF-8
- Line endings: LF or CRLF (both accepted)

---

## 2. Statements and Terminators

Every statement ends with a full stop `.`

```
print "Hello" + newline().
define x as integer = 42.
```

A statement may span multiple lines — the interpreter keeps reading until it finds the terminating `.`

**Decimal vs terminator disambiguation:** a `.` surrounded on both sides by digits is a decimal point inside a number literal. Any other `.` is a statement terminator.

```
define pi as float = 3.14.   # 3.14 → decimal point;  trailing . → terminator
```

---

## 3. Comments

```
# This is a comment — everything to the end of the line is ignored.
define x as integer = 5.   # inline comment after terminator
```

---

## 4. Identifiers

- Characters: `a-z`, `A-Z`, `0-9`, `_`
- Must begin with a letter
- Leading underscore `_` is **forbidden**
- Case-sensitive: `myVar` ≠ `MyVar`

`rootWindow` is a reserved global identifier — it cannot be redefined or assigned to.

---

## 5. Data Types

| Type | Description | Example literal |
|---|---|---|
| `integer` | 64-bit signed integer | `42`, `-7` |
| `float` | 64-bit IEEE 754 double | `3.14`, `-0.5` |
| `string` | UTF-8 text | `"hello"`, `'world'` |
| `currency` | Fixed 2 decimal places (stored as integer cents) | `9.99`, `1000.00` |
| `boolean` | Logical true/false | `true`, `false`, `yes`, `no` |
| `json` | Hierarchical key-value collection | `{ "key": [1, 2, 3] }` |
| `datetime` | ISO 8601 datetime with optional timezone offset | `|2026-04-04 17:41:44|`, `|2026-04-04T17:41:44-12:00|` |

---

## 6. Literals

### Integer
```
42
-7
0
```

### Float
```
3.14
-0.5
1.0
```

### String
Delimited by `"` or `'`. The opening delimiter determines the boundary; the other character is allowed freely inside.

```
"This is a 'fine' string."
'This is a "fine" string.'
```

Escape sequences:

| Sequence | Meaning |
|---|---|
| `\n` | Newline |
| `\r` | Carriage return |
| `\t` | Tab |
| `\'` | Literal `'` |
| `\"` | Literal `"` |
| `\\` | Literal `\` |

> **Important:** `\` has no special meaning outside string literals. Inside a string, `\` followed by an unrecognised character is passed through literally. File paths written inside strings (e.g. `"C:\Users\Simon"`) therefore work as expected.

### Currency
```
9.99
1000.00
0.50
```
Always stored and displayed to exactly 2 decimal places.

### Datetime
Delimited by `|` and written in ISO 8601 form. The `T` separator and timezone offset are optional.

```
|2026-04-04 17:41:44|
|2026-04-04T17:41:44-12:00|
```

### Boolean

| True | False |
|---|---|
| `true` | `false` |
| `TRUE` | `FALSE` |
| `yes` | `no` |
| `YES` | `NO` |

### Null
```
null
```
The value of an uninitialised variable.

### Pixel value (GUI only)
```
10px
200px
```
Used exclusively in `position` assignments inside form definitions. Parsed as an unsigned integer.

---

## 7. Variables

### Declaration
```
define <name> as [constant] [global] <type> [= <expression>].
```

```
define score        as integer = 100.
define name         as string  = "Simon".
define pi           as constant float = 3.14159.
define sessionCount as global integer = 0.
define uninitialised as string.          # value is null
```

- `constant` — immutable after declaration; attempting `set` raises a TypeError.
- `global` — visible everywhere in the program. Must appear at the top level.
- Without an initial value the variable starts as `null`.
- Re-declaring the same name in the same scope is an error.

---

## 8. Assignment

```
set <name> = <expression>.
set <name>++.                 # increment integer by 1
set <name>--.                 # decrement integer by 1
```

For GUI object-path assignment see [§21](#21-gui--object-paths).

---

## 9. Operators

### Arithmetic

| Operator | Meaning | Notes |
|---|---|---|
| `+` | Addition / concatenation | String + string → concatenation |
| `-` | Subtraction | |
| `*` | Multiplication | |
| `/` | Division | integer ÷ integer → integer (truncates toward zero); any float operand → float |
| `%` | Modulo (remainder) | Integers only |
| `**` | Exponentiation | Right-associative |

Datetime arithmetic:
- `datetime + integer` → datetime (integer is milliseconds)
- `datetime - integer` → datetime (integer is milliseconds)
- `datetime - datetime` → integer (milliseconds)

### Comparison (return boolean)

`==`  `!=`  `<`  `<=`  `>`  `>=`

### Logical

| Operator | Meaning |
|---|---|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT (prefix) |

### Null check

```
<expr> is null
<expr> is not null
```

### Precedence (highest → lowest)

1. Unary `not`, unary `-`
2. `**`
3. `*`  `/`  `%`
4. `+`  `-`
5. `<`  `<=`  `>`  `>=`  `==`  `!=`
6. `and`
7. `or`
8. `is null` / `is not null`

---

## 10. Type Casting

```
cast(<expression>, <targetType>)
```

| From | To | Notes |
|---|---|---|
| `integer` | `float` | Exact |
| `integer` | `string` | `"42"` |
| `integer` | `boolean` | `0 → false`, non-zero → `true` |
| `integer` | `currency` | Adds `.00` |
| `float` | `string` | Default float display |
| `float` | `currency` | Rounds to 2 dp |
| `float` | `integer` | Truncates toward zero |
| `boolean` | `string` | `"true"` or `"false"` |
| any | `string` | Always permitted |
| `string` | `datetime` | ISO 8601 format required |
| `datetime` | `string` | ISO 8601 output |

A cast that cannot succeed (e.g. `cast("hello", integer)`) raises a TypeError.

---

## 11. Built-in Functions

All built-ins are expressions and can appear anywhere a value is expected.

| Function | Returns | Description |
|---|---|---|
| `newline()` | string | Returns `"\n"` |
| `length(expr)` | integer | String character count, or collection element/key count |
| `cast(expr, type)` | type | Type conversion — see §10 |
| `string(expr)` | string | Shorthand for `cast(expr, string)` |
| `abs(expr)` | number | Absolute value |
| `floor(expr)` | integer | Round down |
| `ceil(expr)` | integer | Round up |
| `round(expr, dp)` | float | Round to `dp` decimal places |
| `substr(str, start, len)` | string | 0-based substring, Unicode-safe |
| `upper(str)` | string | Convert to uppercase |
| `lower(str)` | string | Convert to lowercase |
| `trim(str)` | string | Strip leading and trailing whitespace |
| `execute(path)` | null | Execute a program or script; stdout/stderr forward to CLI |
| `os()` | string | Returns `"windows"`, `"macos"`, `"linux"`, or `"unknown"` |
| `now()` | datetime | Current local datetime with offset |
| `day(datetime)` | integer | Day of month (1-31) |
| `month(datetime)` | integer | Month (1-12) |
| `year(datetime)` | integer | Year (4-digit) |
| `hour(datetime)` | integer | Hour (0-23) |
| `minute(datetime)` | integer | Minute (0-59) |
| `second(datetime)` | integer | Second (0-59) |
| `dateformat(format, datetime)` | string | Format datetime using ISO tokens (`YYYY`, `MM`, `DD`, `HH`, `mm`, `SS`, `ZZ`) |
| `timestamp(datetime)` | integer | Milliseconds since Unix epoch |
| `fromtimestamp(ms)` | datetime | Datetime from Unix epoch milliseconds (UTC) |
| `append(collection, value)` | null | Append element to a json array (mutates in place) |
| `read_text(path)` | string | Read a UTF-8 text file |
| `write_text(path, content)` | null | Write a UTF-8 text file (overwrites) |
| `read_json(path)` | json | Read and parse JSON from a file |
| `write_json(path, value)` | null | Write JSON to a file (pretty-printed) |
| `read_csv(path, has_headers)` | json | Read CSV into a json array |
| `write_csv(path, rows, include_headers)` | null | Write CSV from a json array |

---

`execute(path)` rules:
- If `path` contains `/` or `\`, it is resolved relative to the running `.run` file directory.
- If `path` contains no separators, `$PATH` is searched.
- Mixing `/` and `\` in the same path is a syntax error.

## 12. Input and Output

### Print
```
print <expression>.
```
Writes the string representation of the expression to stdout. No newline is appended automatically — use `newline()` explicitly.

### Receive (input)
```
receive <identifier> [with prompt <stringExpression>].
```
Reads one line from stdin into the named variable. The optional `with prompt` clause prints the prompt string before waiting. The input is automatically coerced to the variable's declared type; a coercion failure raises TypeError.

### File I/O

All file paths are strings. Relative paths resolve against the running `.run` file directory (or the current working directory in REPL mode).

```
define text as string = read_text("notes.txt").
write_text("out.txt", text + newline()).

define data as json = read_json("data.json").
write_json("copy.json", data).

define rows as json = read_csv("people.csv", true).
write_csv("people_out.csv", rows, true).
```

CSV rules:
- `read_csv(..., true)` returns a json array of objects (header names become keys).
- `read_csv(..., false)` returns a json array of arrays.
- `write_csv(..., true)` expects a json array of objects.
- `write_csv(..., false)` expects a json array of arrays.

---

## 13. Conditionals

### if / else if / else

```
if (<condition>) -->
    <statements>
else if (<condition>) -->
    <statements>
else -->
    <statements>
<--
```

Parentheses around the condition are optional. A single `<--` closes the entire chain.

### switch

```
switch <expression> -->
    <case> : <statement>.
    <case> : <statement>.
    else   : <statement>.
<--
```

- Cases are tested top-to-bottom; the first match wins (no fall-through).
- Cases can be **grouped** (stacked without a body) to share the next body.
- `else` is the default.
- Case patterns may be exact values or comparison expressions (`< 18`, `>= 65`, etc.).

---

## 14. Loops

### for (counted)

```
for <variable> loops (<start>, <end>, <increment>) -->
    <statements>
<--
```

The loop variable must be a pre-declared `integer`. The range is **inclusive** of both ends.

### while

```
while <condition> -->
    <statements>
<--
```

### for each (collection iterator)

```
for each <variable> in <collectionExpression> -->
    <statements>
<--
```

The iteration variable is implicitly declared. The collection must evaluate to a `json` array.

---

## 15. Functions

### Declaration

```
define <name>(<param> as <type>, ...) returns <type> -->
    <statements>
<--
```

Use `returns null` for procedures that return no value.

### Rules

- Functions must be declared before they are called.
- Parameters are local and immutable within the body.
- Variables declared inside a function are local to it.
- Globals may be read inside a function.
- Recursion is permitted.

---

## 16. Collections (json)

The `json` type is a free-form hierarchical key-value store mirroring JSON exactly.

### Declaration

```
define myData as json = {
    "items": [ "alpha", "beta", "gamma" ]
}.
```

### Access

```
myData["items"][0]    # → "alpha"
```

### Mutation

```
set myData["newKey"] = "newValue".
remove myData["oldKey"].
append(myData["items"], "delta").
```

### Building JSON from variables

JSON literal syntax requires literal values — variable references are not permitted inside `{ }`. Build JSON objects by declaring with placeholders and using `set`:

```
define entry as json = { "name": "PLACEHOLDER" }.
set entry["name"] = myNameVariable.
```

---

## 17. Error Handling

```
try -->
    <statements>
catch (<ErrorType>) -->
    <statements>
finally -->
    <statements>
<--
```

- Multiple `catch` clauses are allowed.
- `finally` is optional; it always runs.
- A single `<--` closes the entire structure.
- If no `catch` matches, the error propagates upward.

---

## 18. Modules

```
import "<path>".
```

- Must appear at the **top** of the file, before any declarations.
- The path is relative to the importing file's directory; omit the `.run` extension.
- All global variables and functions from the imported file become available.
- Circular imports are detected and cause an error.

---

## 19. GUI — Forms

> GUI programs run with `rundell-gui` rather than `rundell`.

### Form definition

```
define <name> as form -->
    <form-level property assignments>
    <control declarations>
    <control property assignments>
<--
```

### Form-level properties

| Property | Type | Default | Description |
|---|---|---|---|
| `title` | string | `""` | Title-bar text |
| `width` | pixels | `800px` | Form width |
| `height` | pixels | `600px` | Form height |
| `resizable` | boolean | `false` | User can resize |
| `backgroundcolor` | string (`#RRGGBB`) | `"#A2A2A2"` | Background fill |
| `textcolor` | string | `"#000000"` | Default text colour |
| `textbackground` | string | `"#FFFFFF"` | Default text background |

Inside the form body, reference the form itself with `form`:

```
set form\title  = "My Window".
set form\width  = 600px.
set form\height = 400px.
```

### Showing and closing forms

```
rootWindow\myForm\show().           # modeless — execution continues
rootWindow\myForm\show(modal).      # modal — blocks until form is closed
rootWindow\myForm\close().          # close from a callback
```

`rootWindow` is the built-in global root. You cannot redefine it.

Modal forms block the interpreter thread until the form is closed (maximum 30 seconds before a timeout warning; the form remains open).

---

## 20. GUI — Controls

### Declaring a control

```
define <name> as form\<type>.
```

Control types: `label`, `textbox`, `button`, `radiobutton`, `checkbox`, `switch`, `select`, `listbox`.

### Shared properties

| Property | Type | Default | Description |
|---|---|---|---|
| `position` | `top, left, width, height` | `0px, 0px, 100px, 30px` | Absolute pixel position (Y, X order) |
| `visible` | boolean | `true` | Whether the control is rendered |
| `enabled` | boolean | `true` | Whether the control accepts input |

#### label

| Property | Type | Default |
|---|---|---|
| `value` | string | `""` |
| `textcolor` | string | `"#000000"` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `left` |

No events.

#### textbox

| Property | Type | Default |
|---|---|---|
| `value` | string | `""` |
| `textcolor` | string | `"#000000"` |
| `textbackground` | string | `"#FFFFFF"` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `left` |
| `readonly` | boolean | `false` |
| `maxlength` | integer | (none) |
| `placeholder` | string | `""` |
| `autorefresh` | boolean | `true` |

Events: `change` (fires on each keystroke).

#### button

| Property | Type | Default |
|---|---|---|
| `caption` | string | `""` |
| `textcolor` | string | `"#000000"` |
| `backgroundcolor` | string | `"#E0E0E0"` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `center` |

Events: `click`.

#### radiobutton

| Property | Type | Default |
|---|---|---|
| `caption` | string | `""` |
| `group` | string | `""` |
| `checked` | boolean | `false` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `left` |

Radio buttons sharing the same `group` are mutually exclusive — setting one to `checked = true` automatically clears the others.

Events: `change`.

#### checkbox

| Property | Type | Default |
|---|---|---|
| `caption` | string | `""` |
| `checked` | boolean | `false` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `left` |

Events: `change`.

#### switch

Rendered as a toggle button. `checked = true` represents On/Yes.

| Property | Type | Default |
|---|---|---|
| `caption` | string | `""` |
| `checked` | boolean | `false` |
| `font` | string | `"default"` |
| `fontsize` | integer | `12` |
| `textalign` | enum (`left`, `center`, `right`) | `left` |

Events: `change`.

#### select

| Property | Type | Default | Notes |
|---|---|---|---|
| `items` | json array or csv | `[]` | `["A", "B"]` or `"A, B"` |
| `value` | string | (none) | The currently selected item text (read) |
| `font` | string | `"default"` | Text font family |
| `fontsize` | integer | `12` | Text size in pixels |
| `textalign` | enum (`left`, `center`, `right`) | `left` | Align selected text |

Events: `change`.

#### listbox

| Property | Type | Default | Description |
|---|---|---|---|
| `datasource` | json | `null` | Array variable. Rows from `"rows"`, `"records"`, or top-level array. |
| `columns` | json array | `[]` | Field names to display as columns |
| `imagecolumn` | string | `""` | Field whose value is a base64-encoded PNG/JPEG |
| `font` | string | `"default"` | Row text font family |
| `fontsize` | integer | `12` | Row text size in pixels |
| `multiselect` | boolean | `false` | Allow multiple row selection |
| `value` | json | `null` | Selected record(s) as json (read) |
| `rowheight` | integer | `24` | Row height in pixels |
| `headervisible` | boolean | `true` | Show column headers |

Events: `change` (selection changes), `select` (row double-clicked).

#### eventtimer

Defines a named timer that can invoke a callback at a fixed interval while
`running = true`.

| Property | Type | Default | Notes |
|---|---|---|---|
| `interval` | duration | `0` | `500ms`, `2s`, `1m`, `1h` (no spaces) |
| `running` | boolean | `false` | Starts the timer when `true` |
| `event` | function | (none) | Callback function (zero args, returns null) |

Example:

```
define myTimer as eventtimer -->
    set myTimer\interval = 10s.
    set myTimer\event = myCallback().
<--

define myCallback() returns null -->
    print "Timer fired" + newline().
    return null.
<--

set myTimer\running = true.
```

---

## 21. GUI — Object Paths

An **object path** navigates the form hierarchy using `\` as a separator:

```
rootWindow\<formName>\<controlName>\<property>
```

The leading `rootWindow\` is optional for forms registered in the current program:

```
myForm\myLabel\value
rootWindow\myForm\myLabel\value   # equivalent
```

**Reading** a property (in an expression):

```
define v as string = myForm\myTextbox\value.
print myForm\myLabel\value.
```

**Writing** a property (`set` statement):

```
set myForm\myLabel\value = "Updated text".
set myForm\myButton\enabled = false.
```

Object-path reads always return a `string`. Use `cast()` when a numeric type is needed.

Inside a form definition body, `form` refers to the form being declared:

```
set form\title = "My Form".
```

---

## 22. GUI — Events

Bind a control event to a zero-argument function:

```
set <control>\<event> = <functionName>().
```

The function must be declared with `returns null` and no parameters:

```
define handleClick() returns null -->
    # event handler body
    return null.
<--
```

| Control | Event | Fires when |
|---|---|---|
| button | `click` | Clicked |
| textbox | `change` | Text edited (on each keystroke) |
| radiobutton | `change` | Checked state changes |
| checkbox | `change` | Checked state changes |
| switch | `change` | Toggle state changes |
| select | `change` | Selection changes |
| listbox | `change` | Selection changes |
| listbox | `select` | Row double-clicked |

---

## 23. GUI — System Dialogs

The `dialog` namespace provides native OS dialogs. All calls block the interpreter thread.

### dialog\openfile(title, filter)

Opens a file-open dialog. Returns the selected path as a `string`, or `""` if cancelled.

```
set path = dialog\openfile("Open File", "Rundell Files (*.run)").
```

### dialog\savefile(title, filter)

Opens a file-save dialog. Returns the chosen path as a `string`, or `""` if cancelled.

```
set savePath = dialog\savefile("Save As", "Rundell Files (*.run)").
```

### dialog\message(title, message, kind)

Displays a modal message box. `kind` is one of `ok`, `okcancel`, `yesno`.

Returns `"ok"`, `"cancel"`, `"yes"`, or `"no"`.

```
set answer = dialog\message("Confirm", "Delete this record?", yesno).
if (answer == "yes") --> deleteRecord(). <--
```

### dialog\colorpicker(initial)

Opens a colour picker. `initial` is a `"#RRGGBB"` string. Returns the chosen colour, or `initial` if cancelled.

```
set colour = dialog\colorpicker("#FF0000").
set myForm\myLabel\textcolor = colour.
```

---

## 24. GUI — The Form Designer

Launch the visual designer with:

```bash
cargo run -p rundell-gui -- --design
```

The designer provides:

- **Controls palette** (left panel) — click a control type to place it on the canvas.
- **Design canvas** (centre) — drag controls to reposition; click to select.
- **Properties inspector** (right panel) — edit properties for the selected control.
- **Code panel** (bottom) — click **Generate Code** to produce a Rundell form definition, then **Copy to Clipboard** or **Save to File**.

The generated `.run` file contains a syntactically valid `define ... as form --> ... <--` block.

---

---

## 25. REST — Credentials

A `credentials` definition holds authentication values for one or more REST queries. Values are read at runtime from the encrypted `.rundell.env` file using the `env()` built-in — plain-text secrets must never appear in `.run` source files.

```
define <name> as credentials -->
    set <name>\token          = env("<KEY_NAME>").
    set <name>\authentication = env("<KEY_NAME>").
<--
```

Both properties are optional. A credentials block with no properties is valid for public APIs that need no authentication.

| Property | Header sent | Description |
|---|---|---|
| `token` | `Authorization: Bearer <value>` | JWT or API bearer token |
| `authentication` | `X-Rundell-Auth: <value>` | Custom authentication value |

```
define myCredentials as credentials -->
    set myCredentials\token          = env("MY_API_TOKEN").
    set myCredentials\authentication = env("MY_API_SECRET").
<--
```

See [§29](#29-rest--credential-storage) for how to store values in `.rundell.env`.

---

## 26. REST — Query Definitions

A `query` definition declares a named, parameterised REST call. It is registered at definition time and executed only when called with `await`.

```
define <name>(<params>) as query returns json -->
    set <name>\method      = GET.            # or POST — mandatory
    set <name>\endpoint    = <expression>.   # mandatory
    set <name>\credentials = <name>.         # optional
    set <name>\timeout     = <integer>.      # optional, milliseconds
    define queryParams as json = { ... }.    # POST only
<--
```

- `returns json` is **mandatory**. Omitting it is a parse error.
- `method` is **mandatory**. Omitting it is a parse error.
- `endpoint` is **mandatory**. Omitting it is a parse error.
- `queryParams` is only valid when `method = POST`. Using it with `GET` is a parse error.
- The default timeout is **10 000 ms**. Override per-query with `set <name>\timeout = <ms>.`

### GET query — no parameters

```
define getAllUsers() as query returns json -->
    set getAllUsers\method      = GET.
    set getAllUsers\endpoint    = "https://api.example.com/users".
    set getAllUsers\credentials = myCredentials.
<--
```

### POST query — with a parameter

```
define getUser(uid as integer) as query returns json -->
    set getUser\method      = POST.
    set getUser\endpoint    = "https://api.example.com/user".
    set getUser\credentials = myCredentials.
    define queryParams as json = {
        "uid": uid
    }.
<--
```

`queryParams` is a reserved identifier inside a query block. It builds the JSON POST body. Variable references (such as `uid` above) are resolved at call time from the query's parameters.

### Public API — no credentials

```
define getRandomDog() as query returns json -->
    set getRandomDog\method   = GET.
    set getRandomDog\endpoint = "https://dog.ceo/api/breeds/image/random".
<--
```

---

## 27. REST — Calling Queries with await

Use the `await` keyword to call a query. The call blocks the program until the HTTP response is received (or the timeout expires) and returns the parsed JSON body as a `json` value.

```
set <variable> = await <queryName>(<args>).
```

`await` is only valid as the right-hand side of a `set` statement. It is a parse error in any other position.

### Calling a query into a variable

```
define result as json.
set result = await getUser(42).
print result.
```

### Direct binding to a GUI control

`await` can be the RHS of an object-path assignment, allowing direct binding of a query result to a control property:

```
set myForm\myListbox\datasource = await getAllUsers().
```

---

## 28. REST — Error Handling with attempt/catch

The `attempt / catch` construct handles query-related errors without terminating the program.

```
attempt -->
    <statements>
<-- catch <identifier> -->
    <statements>
<--
```

The `catch` block is **mandatory**. The identifier is bound to an error object with these properties:

| Property | Type | Description |
|---|---|---|
| `<name>\message` | string | Human-readable error description |
| `<name>\statusCode` | integer | HTTP status code (`0` if no response received) |
| `<name>\endpoint` | string | The endpoint URL that was called |

```
attempt -->
    define result as json.
    set result = await getUser(99).
    print result.
<-- catch queryError -->
    print "Error: " + queryError\message + newline().
    print "Status: " + queryError\statusCode + newline().
<--

print "Execution continues here after a caught error.".
```

### What attempt/catch intercepts

`attempt / catch` intercepts only query-related errors:

- Network failure before any response was received
- HTTP error status codes (4xx, 5xx)
- Response timeout
- Response body that is not valid JSON
- Undefined query name called with `await`
- Undefined credentials referenced by a query
- `env()` key not found or decryption failure

**Type errors, division by zero, null errors, and all other runtime errors propagate through `attempt / catch` unaffected.** Use the standard `try / catch` construct for those.

---

## 29. REST — Credential Storage

Credentials are stored encrypted in a `.rundell.env` file in the same directory as the `.run` program. The encryption key is derived from the machine identity — credential files should not be shared or copied between machines.

### CLI commands

Use the `rundell` binary to manage credentials:

```bash
rundell --env-set KEY_NAME value      # encrypt and store a credential
rundell --env-list                    # list stored key names (not values)
rundell --env-delete KEY_NAME         # remove a credential
```

`--env-set` does not echo the value after storing it. The `.rundell.env` file is always located in the **current working directory** when these commands are run, so run them from the same directory as your `.run` program.

### env() built-in

Inside a credentials block (or anywhere in a program), `env()` reads a decrypted value from the adjacent `.rundell.env` file:

```
set myCredentials\token = env("MY_API_TOKEN").
```

`env()` takes a single string argument (the key name). It raises `EnvKeyNotFound` if the key is absent and `EnvDecryptionFailed` if the data is corrupt. Both errors are catchable by `attempt / catch`.

### Complete workflow

```bash
# 1. Store the credentials (run from your project directory)
rundell --env-set MY_API_TOKEN eyJhbGciOiJIUzI1NiJ9...
rundell --env-set MY_API_SECRET s3cr3tv4lu3

# 2. Verify they are stored
rundell --env-list
```

```
# 3. Use them in your program
define myCredentials as credentials -->
    set myCredentials\token          = env("MY_API_TOKEN").
    set myCredentials\authentication = env("MY_API_SECRET").
<--
```

---

## 30. Keywords

The following identifiers are reserved and may not be used as variable or function names:

```
define  as  constant  global  set  return  import
if  else  switch  for  while  each  in  loops
true  false  yes  no  TRUE  FALSE  YES  NO
null  and  or  not  is
print  receive  with  prompt
try  catch  finally
integer  float  string  currency  boolean  json
datetime
cast  length  newline  abs  floor  ceil  round
substr  upper  lower  trim  now  day  month  year  hour  minute  second
dateformat  timestamp  fromtimestamp  execute  os  append  remove
returns

# GUI keywords
form  show  close  modal  dialog
label  textbox  button  radiobutton  checkbox  select  listbox
autorefresh  datasource  columns

# REST keywords
query  credentials  await  attempt
method  endpoint  token  authentication  timeout
GET  POST  env

# Reserved globals
rootWindow
```

---

## 31. Error Types

### Standard errors (caught by try/catch)

| Type | Raised when |
|---|---|
| `TypeError` | Type mismatch, invalid cast, assigning to a constant, wrong operand types |
| `NullError` | Using a `null` variable in arithmetic or other operations |
| `IndexError` | Collection index out of bounds |
| `DivisionError` | Division or modulo by zero |
| `IOError` | Input/output failure |
| `PermissionError` | Insufficient permissions to execute a program or script |
| `RuntimeError` | Catch-all for any other runtime error; also raised on invalid object paths |

### Query errors (caught by attempt/catch)

| Type | Raised when |
|---|---|
| `QueryTimeout` | The HTTP request exceeded the timeout duration |
| `QueryNetworkError` | A network-level failure occurred before any response |
| `QueryHttpError` | The server responded with a 4xx or 5xx status code |
| `QueryInvalidJson` | The response body could not be parsed as JSON |
| `UndefinedQuery` | `await` was called on a name that has no query definition |
| `UndefinedCredentials` | A query references a credentials name that has not been defined |
| `EnvKeyNotFound` | `env()` was called but the key is not in `.rundell.env` |
| `EnvDecryptionFailed` | `env()` found the key but decryption failed (wrong machine or corrupt file) |
