# Rundell Tutorial

This tutorial takes you from zero to a working knowledge of Rundell — including its graphical UI system. Work through the sections in order; each one builds on the last.

---

## Contents

1. [Hello, World!](#1-hello-world)
2. [Variables and Types](#2-variables-and-types)
3. [Reading Input](#3-reading-input)
4. [Arithmetic](#4-arithmetic)
5. [Strings](#5-strings)
6. [Booleans and Logic](#6-booleans-and-logic)
7. [Type Casting](#7-type-casting)
8. [Making Decisions](#8-making-decisions)
9. [Loops](#9-loops)
10. [Functions](#10-functions)
11. [Collections](#11-collections)
12. [Error Handling](#12-error-handling)
13. [Splitting Code into Modules](#13-splitting-code-into-modules)
14. [Your First GUI Form](#14-your-first-gui-form)
15. [Controls and Properties](#15-controls-and-properties)
16. [Events and Callbacks](#16-events-and-callbacks)
17. [Reading Values from a Form](#17-reading-values-from-a-form)
18. [Modal vs Modeless Forms](#18-modal-vs-modeless-forms)
19. [System Dialogs](#19-system-dialogs)
20. [The Listbox and Data Binding](#20-the-listbox-and-data-binding)
21. [The Form Designer](#21-the-form-designer)
22. [Putting It All Together](#22-putting-it-all-together)
23. [Your First REST Query](#23-your-first-rest-query)
24. [Handling Query Errors](#24-handling-query-errors)
25. [Authenticated APIs](#25-authenticated-apis)
26. [REST Data in a GUI Listbox](#26-rest-data-in-a-gui-listbox)
27. [File I/O](#27-file-io)
28. [Executing External Programs](#28-executing-external-programs)
29. [Utility Built-ins](#29-utility-built-ins)
30. [Debugging Your Programs](#30-debugging-your-programs)

---

## 1. Hello, World!

Create a file called `hello.run`:

```
print "Hello, World!" + newline().
```

Run it:

```bash
rundell hello.run
```

Output:
```
Hello, World!
```

**What's happening?**

- `print` writes text to the screen.
- `"Hello, World!"` is a string literal.
- `newline()` returns a newline character. Because `print` never adds one automatically, you add it yourself.
- `+` between two strings concatenates them.
- Every statement ends with a full stop `.`

---

## 2. Variables and Types

Rundell is **strongly typed** — every variable has a fixed type declared up front.

```
define greeting as string   = "Hello".
define count    as integer  = 10.
define price    as currency = 4.99.
define ratio    as float    = 0.75.
define active   as boolean  = true.
```

The form is always:

```
define <name> as <type> [= <value>].
```

If you leave out `= <value>`, the variable starts as `null`.

```
define result as integer.   # result is null until you set it
```

### Constants

Add `constant` to make a variable immutable:

```
define PI as constant float = 3.14159.
# set PI = 3.0.   ← this would raise a TypeError
```

### Global variables

Add `global` to declare a variable visible everywhere, including inside functions:

```
define global appName as constant string = "My App".
```

---

## 3. Reading Input

Use `receive` to read a line from the user:

```
define name as string.
define age  as integer.

receive name with prompt "What is your name? ".
receive age  with prompt "How old are you? ".

print "Hello, " + name + "! You are " + string(age) + " years old." + newline().
```

The input is automatically converted to the variable's declared type. Typing something that cannot be converted raises a `TypeError`.

---

## 4. Arithmetic

```
define a as integer = 10.
define b as integer = 3.

print string(a + b) + newline().   # 13
print string(a - b) + newline().   # 7
print string(a * b) + newline().   # 30
print string(a / b) + newline().   # 3   (integer division truncates)
print string(a % b) + newline().   # 1   (remainder)
print string(2 ** 8) + newline().  # 256 (exponentiation)
```

When both operands are integers, `/` returns an integer (truncated). For a float result, cast first:

```
define result as float = cast(a, float) / cast(b, float).
print string(result) + newline().   # 3.3333333333333335
```

---

## 5. Strings

```
define s as string = "Hello, World!".

print string(length(s)) + newline().   # 13
print upper(s) + newline().            # HELLO, WORLD!
print lower(s) + newline().            # hello, world!
print substr(s, 0, 5) + newline().     # Hello  (0-based, length 5)
print trim("  padded  ") + newline().  # padded
```

Use `+` to concatenate. Both sides must be strings — convert other types with `string()` first:

```
define age as integer = 25.
print "Age: " + string(age) + newline().
```

You can use `"` or `'` as delimiters; the other is freely usable inside:

```
print "It's a beautiful day." + newline().
print 'She said "wow".' + newline().
```

---

## 6. Booleans and Logic

```
define x as boolean = true.
define y as boolean = false.

print string(x and y) + newline().   # false
print string(x or y)  + newline().   # true
print string(not x)   + newline().   # false
```

All four spellings work: `true`, `TRUE`, `yes`, `YES` (and the false equivalents).

A variable declared without an initial value is `null`. Test for it with `is null` or `is not null`:

```
define value as integer.

if (value is null) -->
    print "value has not been set yet" + newline().
<--
```

---

## 7. Type Casting

Use `cast(expression, targetType)` to convert between types:

```
define i as integer = 42.

define f as float   = cast(i, float).    # 42.0
define s as string  = cast(i, string).   # "42"
define b as boolean = cast(0, boolean).  # false
```

`string()` is a handy shorthand for `cast(expr, string)`.

---

## 8. Making Decisions

### if / else if / else

```
define score as integer = 75.

if (score >= 70) -->
    print "Distinction" + newline().
else if (score >= 40) -->
    print "Pass" + newline().
else -->
    print "Fail" + newline().
<--
```

A single `<--` closes the whole chain.

### switch

```
define day as integer = 3.

switch day -->
    1 : print "Monday" + newline().
    2 : print "Tuesday" + newline().
    3 : print "Wednesday" + newline().
    6 :
    7 : print "Weekend" + newline().
    else : print "Other" + newline().
<--
```

Days 6 and 7 are **grouped** — they share the weekend body. Cases with comparison operators also work:

```
switch age -->
    < 13  : print "Child" + newline().
    < 18  : print "Teenager" + newline().
    else  : print "Adult" + newline().
<--
```

---

## 9. Loops

### for — counted loop

```
define i as integer.

for i loops (1, 5, 1) -->
    print string(i) + newline().
<--
# output: 1  2  3  4  5
```

The range is **inclusive**. Count down with a negative increment:

```
for i loops (5, 1, -1) --> print string(i) + newline(). <--
```

### while — condition loop

```
define count as integer = 1.

while count <= 5 -->
    print string(count) + newline().
    set count++.
<--
```

### for each — iterate a collection

```
define fruits as json = { "list": ["apple", "banana", "cherry"] }.

for each fruit in fruits["list"] -->
    print fruit + newline().
<--
```

---

## 10. Functions

```
define multiply(a as integer, b as integer) returns integer -->
    return a * b.
<--

print string(multiply(6, 7)) + newline().   # 42
```

A function that returns no value declares `returns null`:

```
define printLine(text as string) returns null -->
    print text + newline().
    return null.
<--
```

Recursion is supported:

```
define factorial(n as integer) returns integer -->
    if (n <= 1) --> return 1. <--
    return n * factorial(n - 1).
<--
```

---

## 11. Collections

The `json` type stores structured data — objects, arrays, and any combination.

```
define library as json = {
    "books": [
        { "title": "Dune",       "year": 1965 },
        { "title": "Foundation", "year": 1951 }
    ]
}.

print library["books"][0]["title"] + newline().   # Dune

for each book in library["books"] -->
    print book["title"] + " (" + string(book["year"]) + ")" + newline().
<--
```

Modify a collection:

```
set library["genre"] = "Sci-Fi".     # add/update
remove library["genre"].             # remove
append(library["books"], newBook).   # append to array
```

---

## 12. Error Handling

```
define age as integer.

try -->
    receive age with prompt "Enter your age: ".
    print "You are " + string(age) + " years old." + newline().
catch (TypeError) -->
    print "Please enter a whole number." + newline().
finally -->
    print "Thank you." + newline().
<--
```

`finally` always runs, whether or not an error occurred.

---

## 13. Splitting Code into Modules

**maths.run**
```
define global PI as constant float = 3.14159265.

define circleArea(radius as float) returns float -->
    return PI * radius * radius.
<--
```

**main.run**
```
import "maths".

define r as float = 5.0.
print "Area: " + string(round(circleArea(r), 2)) + newline().
```

- `import` statements go at the very top of the file.
- The path is relative to the importing file; omit the `.run` extension.
- All global variables and functions from the imported file become available.

---

## 14. Your First GUI Form

GUI programs run with `rundell-gui` instead of `rundell`:

```bash
cargo run -p rundell-gui -- my_form.run
```

A **form** is a window with controls on it. Define one like this:

```
define myForm as form -->
    set form\title  = "My First Form".
    set form\width  = 400px.
    set form\height = 200px.
    define myLabel as form\label.
    set myLabel\position = 10px, 10px, 380px, 30px.  # top, left, width, height
    set myLabel\value    = "Welcome to Rundell!".
<--

rootWindow\myForm\show().
```

**Key points:**

- `define myForm as form --> ... <--` defines the form.
- Inside the block, `set form\property = value.` sets form-level properties like `title`, `width`, and `height`.
- `define myLabel as form\label.` declares a label control named `myLabel`.
- `set myLabel\position = top, left, width, height.` positions it using pixel values (e.g. `10px`).
- `rootWindow\myForm\show().` opens the form window.
- `rootWindow` is a built-in global — you cannot redefine it.

---

## 15. Controls and Properties

Rundell has eight built-in control types. All controls share `position`, `visible`, and `enabled` properties.

### label

Displays static text.

```
define myLabel as form\label.
set myLabel\position  = 10px, 10px, 200px, 25px.
set myLabel\value     = "Hello!".
set myLabel\textcolor = "#0000AA".
```

### textbox

Single-line text entry.

```
define myTextbox as form\textbox.
set myTextbox\position    = 10px, 10px, 250px, 28px.
set myTextbox\placeholder = "Type here…".
set myTextbox\value       = "default text".
```

### button

```
define myButton as form\button.
set myButton\position        = 10px, 50px, 120px, 32px.
set myButton\caption         = "Click Me".
set myButton\backgroundcolor = "#4C8BF5".
set myButton\textcolor       = "#FFFFFF".
```

### radiobutton

Radio buttons in the same `group` are mutually exclusive:

```
define optA as form\radiobutton.
define optB as form\radiobutton.
set optA\position = 10px, 10px, 200px, 24px.
set optA\caption  = "Option A".
set optA\group    = "myGroup".
set optB\position = 10px, 40px, 200px, 24px.
set optB\caption  = "Option B".
set optB\group    = "myGroup".
set optA\checked  = true.
```

### checkbox

```
define myCheck as form\checkbox.
set myCheck\position = 10px, 10px, 200px, 24px.
set myCheck\caption  = "I agree".
set myCheck\checked  = false.
```

### switch

A yes/no toggle rendered as a labelled toggle button:

```
define mySwitch as form\switch.
set mySwitch\position = 10px, 10px, 150px, 28px.
set mySwitch\caption  = "Enable notifications".
```

### select

A dropdown list. Set items from a JSON array or a comma-separated string:

```
define mySelect as form\select.
set mySelect\position = 10px, 10px, 200px, 28px.
set mySelect\items    = ["Option A", "Option B", "Option C"].
```

### listbox

A scrollable, multi-column data-bound list. See [Section 20](#20-the-listbox-and-data-binding).

---

## 16. Events and Callbacks

Controls fire events when the user interacts with them. Bind an event to a function name:

```
set myButton\click = handleClick().
```

The bound function must be declared with no parameters and `returns null`:

```
define handleClick() returns null -->
    print "Button was clicked!" + newline().
    return null.
<--
```

**Event names by control:**

| Control | Event | Fires when |
|---|---|---|
| button | `click` | Button is clicked |
| textbox | `change` | Text is edited (on each keystroke) |
| radiobutton | `change` | Checked state changes |
| checkbox | `change` | Checked state changes |
| switch | `change` | Toggle state changes |
| select | `change` | Selection changes |
| listbox | `change` | Selection changes |
| listbox | `select` | Row is double-clicked |

**Full example:**

```
define myForm as form -->
    set form\title  = "Event Demo".
    set form\width  = 400px.
    set form\height = 200px.
    define myLabel  as form\label.
    define myButton as form\button.
    set myLabel\position  = 10px, 10px, 380px, 25px.
    set myLabel\value     = "Not clicked yet".
    set myButton\position = 10px, 50px, 120px, 32px.
    set myButton\caption  = "Click Me".
    set myButton\click    = handleClick().
<--

define handleClick() returns null -->
    set myForm\myLabel\value = "Button was clicked!".
    return null.
<--

rootWindow\myForm\show().
```

---

## 16.1. Event Timers

Event timers call a zero-argument callback on a fixed interval while running.

```
define myTimer as eventtimer -->
    set myTimer\interval = 2s.
    set myTimer\event = onTick().
<--

define onTick() returns null -->
    print "Tick" + newline().
    return null.
<--

set myTimer\running = true.
```

---

## 17. Reading Values from a Form

Read a control's current value using the object-path syntax:

```
define enteredText as string = myForm\myTextbox\value.
print enteredText + newline().
```

You can also use object paths anywhere an expression is expected:

```
if (myForm\myCheck\checked == "true") -->
    print "Checkbox is ticked." + newline().
<--
```

> **Note:** Object-path reads always return a `string`. Convert with `cast()` when you need a numeric type.

---

## 18. Modal vs Modeless Forms

**Modeless** — the program continues executing immediately after `show()`:

```
rootWindow\myForm\show().
print "This prints straight away." + newline().
```

**Modal** — the program blocks until the form is closed:

```
rootWindow\myForm\show(modal).
print "This only prints after the form closes." + newline().
```

Close a form from within a callback:

```
define closeForm() returns null -->
    rootWindow\myForm\close().
    return null.
<--
```

---

## 19. System Dialogs

The `dialog` namespace provides native OS dialogs. All dialogs block until the user responds.

### File open

```
set myPath = dialog\openfile("Open a Rundell File", "Rundell Files (*.run)").
if (myPath is not null) -->
    print "Selected: " + myPath + newline().
<--
```

### File save

```
set savePath = dialog\savefile("Save As", "Rundell Files (*.run)").
```

### Message box

`kind` is one of: `ok`, `okcancel`, `yesno`. Returns `"ok"`, `"cancel"`, `"yes"`, or `"no"`.

```
set answer = dialog\message("Confirm", "Delete this record?", yesno).
if (answer == "yes") -->
    print "Deleted." + newline().
<--
```

### Colour picker

```
set colour = dialog\colorpicker("#FF0000").
set myForm\myLabel\textcolor = colour.
```

---

## 20. The Listbox and Data Binding

The listbox is the most powerful control. It binds directly to a Rundell `json` variable and renders one row per record.

```
define myData as json = {
    "rows": [
        { "name": "Alice", "score": 95 },
        { "name": "Bob",   "score": 82 },
        { "name": "Carol", "score": 71 }
    ]
}.

define myForm as form -->
    set form\title  = "Scores".
    set form\width  = 400px.
    set form\height = 300px.
    define myList as form\listbox.
    set myList\position    = 10px, 10px, 380px, 250px.
    set myList\datasource  = myData.
    set myList\columns     = ["name", "score"].
    set myList\rowheight   = 28.
<--

rootWindow\myForm\show().
```

**Listbox properties:**

| Property | Description |
|---|---|
| `datasource` | A `json` variable. Rows are taken from `"rows"`, `"records"`, or the top-level array. |
| `columns` | JSON array of field names to display as columns. |
| `imagecolumn` | Field name whose value is a base64-encoded PNG/JPEG image. |
| `multiselect` | `true` to allow multiple row selection. |
| `rowheight` | Row height in pixels (default 24). |
| `headervisible` | `true` (default) to show column headers. |

---

## 21. The Form Designer

The visual form designer lets you build forms without writing code. Launch it with:

```bash
cargo run -p rundell-gui -- --design
```

**Workflow:**

1. Click a control type in the **Controls** palette on the left to add it to the canvas.
2. Drag controls on the canvas to reposition them.
3. Select a control to edit its properties in the **Properties** panel on the right.
4. Click **Generate Code** in the bottom panel to produce Rundell source.
5. Click **Copy to Clipboard** or **Save to File** to export the code.

The generated file contains a valid `define ... as form --> ... <--` block that you can paste into your program and call `rootWindow\myForm\show().` on.

---

## 22. Putting It All Together

Here is a small but complete GUI program — a simple name-and-score entry form:

```
# score_entry.run
# A modal form that collects a name and score, then prints the result.

define myForm as form -->
    set form\title  = "Score Entry".
    set form\width  = 350px.
    set form\height = 220px.

    define nameLabel  as form\label.
    define nameBox    as form\textbox.
    define scoreLabel as form\label.
    define scoreBox   as form\textbox.
    define okBtn      as form\button.
    define cancelBtn  as form\button.

    set nameLabel\position  = 10px, 10px, 80px, 24px.
    set nameLabel\value     = "Name:".
    set nameBox\position    = 95px, 10px, 240px, 28px.
    set nameBox\placeholder = "Enter full name".

    set scoreLabel\position = 10px, 50px, 80px, 24px.
    set scoreLabel\value    = "Score:".
    set scoreBox\position   = 95px, 50px, 100px, 28px.
    set scoreBox\placeholder = "0-100".

    set okBtn\position      = 10px,  130px, 100px, 32px.
    set okBtn\caption       = "OK".
    set okBtn\click         = submitForm().

    set cancelBtn\position  = 120px, 130px, 100px, 32px.
    set cancelBtn\caption   = "Cancel".
    set cancelBtn\click     = cancelForm().
<--

define submitForm() returns null -->
    define name  as string = myForm\nameBox\value.
    define score as string = myForm\scoreBox\value.
    print name + " scored " + score + newline().
    rootWindow\myForm\close().
    return null.
<--

define cancelForm() returns null -->
    rootWindow\myForm\close().
    return null.
<--

rootWindow\myForm\show(modal).
print "Form closed." + newline().
```

---

## 23. Your First REST Query

Rundell can call HTTP REST APIs and work with the JSON response directly. The `query` definition declares the call; the `await` keyword executes it.

Here is a complete program that fetches a random dog image URL from a free public API and prints it:

```
# dog.run
# Fetches a random dog image URL from the dog.ceo public API.
# Run with: rundell dog.run

define getRandomDog() as query returns json -->
    set getRandomDog\method   = GET.
    set getRandomDog\endpoint = "https://dog.ceo/api/breeds/image/random".
<--

define result as json.
set result = await getRandomDog().
print result.
```

Run it:

```bash
rundell dog.run
```

Output (values vary):
```
{"message":"https://images.dog.ceo/breeds/poodle-toy/n02113624_253.jpg","status":"success"}
```

**What's happening?**

- `define getRandomDog() as query returns json --> ... <--` registers a named HTTP call. Nothing is sent over the network at this point.
- `set result = await getRandomDog().` executes the call and waits for the response. The parsed JSON body is assigned to `result`.
- `print result.` prints the full JSON object. You can also index into it: `result["message"]` gives the URL string.

### Accessing specific fields

```
define result as json.
set result = await getRandomDog().
print result["message"] + newline().
print result["status"]  + newline().
```

### POST queries with parameters

To send data in the request body, use `method = POST` and declare `queryParams`:

```
define getUser(uid as integer) as query returns json -->
    set getUser\method   = POST.
    set getUser\endpoint = "https://api.example.com/user".
    define queryParams as json = {
        "uid": uid
    }.
<--

define user as json.
set user = await getUser(42).
print user["firstName"] + " " + user["secondName"] + newline().
```

The `queryParams` block defines the JSON body sent with the request. Parameter values (like `uid`) are substituted at call time. The `Content-Type: application/json` header is added automatically.

---

## 24. Handling Query Errors

Network calls can fail: the server might be down, return an error code, or time out. The `attempt / catch` construct handles these failures gracefully.

```
# error_demo.run
define badQuery() as query returns json -->
    set badQuery\method   = GET.
    set badQuery\endpoint = "https://httpstat.us/404".
<--

attempt -->
    define result as json.
    set result = await badQuery().
    print result.
<-- catch err -->
    print "Something went wrong!" + newline().
    print "Status code: " + err\statusCode + newline().
    print "Message:     " + err\message    + newline().
    print "Endpoint:    " + err\endpoint   + newline().
<--

print "Program continues after a caught error." + newline().
```

The `catch` identifier (`err` above) exposes three properties:

| Property | Description |
|---|---|
| `err\message` | Human-readable description |
| `err\statusCode` | HTTP status code, or `0` if no response was received |
| `err\endpoint` | The URL that was called |

**Execution continues normally after the catch block**, so the program does not terminate on a caught query error.

### Timeout handling

Set a per-query timeout in milliseconds:

```
define slowQuery() as query returns json -->
    set slowQuery\method   = GET.
    set slowQuery\endpoint = "https://httpstat.us/200?sleep=15000".
    set slowQuery\timeout  = 2000.   # give up after 2 seconds
<--

attempt -->
    define result as json.
    set result = await slowQuery().
<-- catch err -->
    print "Timed out (status " + err\statusCode + ")." + newline().
<--
```

The default timeout when none is specified is **10 000 ms** (10 seconds).

### What attempt/catch does and does not catch

`attempt / catch` handles **query errors only**. It does not suppress type errors, division by zero, or other programming errors — those continue to propagate as normal. Use the standard `try / catch` for those.

---

## 25. Authenticated APIs

Most real APIs require authentication. Rundell stores credentials in an encrypted `.rundell.env` file rather than in source code, so secrets never appear in your `.run` files.

### Step 1 — store your credentials

Open a terminal in the same directory as your program and run:

```bash
rundell --env-set MY_API_TOKEN eyJhbGciOiJIUzI1NiJ9...
rundell --env-set MY_API_SECRET s3cr3tv4lu3

rundell --env-list
# MY_API_TOKEN
# MY_API_SECRET
```

The values are encrypted with a key derived from your machine identity and stored in `.rundell.env`. This file is safe to leave on disk but should not be shared or copied to another machine.

### Step 2 — declare a credentials block

```
define myCredentials as credentials -->
    set myCredentials\token          = env("MY_API_TOKEN").
    set myCredentials\authentication = env("MY_API_SECRET").
<--
```

`env("KEY_NAME")` reads and decrypts the value at runtime. Neither the key nor the credentials block ever contains a plain-text secret.

### Step 3 — attach credentials to a query

```
define getUsers() as query returns json -->
    set getUsers\method      = GET.
    set getUsers\endpoint    = "https://api.example.com/users".
    set getUsers\credentials = myCredentials.
<--
```

Rundell automatically adds:
- `Authorization: Bearer <token>` if `token` is set
- `X-Rundell-Auth: <value>` if `authentication` is set

### Putting it all together

```
# authenticated_api.run
define myCredentials as credentials -->
    set myCredentials\token = env("MY_API_TOKEN").
<--

define getUsers() as query returns json -->
    set getUsers\method      = GET.
    set getUsers\endpoint    = "https://api.example.com/users".
    set getUsers\credentials = myCredentials.
<--

attempt -->
    define users as json.
    set users = await getUsers().
    for each user in users["rows"] -->
        print user["firstName"] + " " + user["secondName"] + newline().
    <--
<-- catch err -->
    print "Failed to load users: " + err\message + newline().
<--
```

---

## 26. REST Data in a GUI Listbox

The most common pattern in Rundell GUI applications is loading REST data into a listbox. You can bind a query result directly to a listbox's `datasource` property.

```
# rest_users.run
# Run with: rundell-gui rest_users.run

define myCredentials as credentials -->
    set myCredentials\token = env("MY_API_TOKEN").
<--

define getUsers() as query returns json -->
    set getUsers\method      = GET.
    set getUsers\endpoint    = "https://api.example.com/users".
    set getUsers\credentials = myCredentials.
<--

define userForm as form -->
    set form\title  = "User List".
    set form\width  = 600px.
    set form\height = 500px.

    define loadBtn    as form\button.
    define statusLbl  as form\label.
    define userList   as form\listbox.

    set loadBtn\position   = 10px, 10px, 120px, 30px.  # top, left, width, height
    set loadBtn\caption    = "Load Users".
    set loadBtn\click      = loadUsers().

    set statusLbl\position = 140px, 10px, 440px, 30px.
    set statusLbl\value    = "Click Load Users to begin.".

    set userList\position  = 10px, 50px, 580px, 420px.
    set userList\columns   = ["firstName", "secondName", "age"].
<--

define loadUsers() returns null -->
    attempt -->
        set userForm\statusLbl\value = "Loading...".
        set userForm\userList\datasource = await getUsers().
        set userForm\statusLbl\value = "Loaded.".
    <-- catch err -->
        set userForm\statusLbl\value = "Error: " + err\message.
    <--
    return null.
<--

rootWindow\userForm\show().
```

**Key points:**

- `set userForm\userList\datasource = await getUsers().` — the query result is assigned directly to the listbox without an intermediate variable.
- The status label changes to show loading state, then `"Loaded."` on success or the error message on failure.
- All network activity happens on button click inside an `attempt / catch` block, so a failed request updates the status label rather than crashing the program.

---

## 27. File I/O

Rundell can read and write UTF-8 text, JSON, and CSV files. Paths are strings; relative paths resolve against the `.run` file directory.

### Text and JSON

```
define notePath as string = "notes.txt".
write_text(notePath, "Line one" + newline() + "Line two").

define contents as string = read_text(notePath).
print contents + newline().

define settings as json = { "theme": "light", "fontSize": 14 }.
write_json("settings.json", settings).

define loaded as json = read_json("settings.json").
print loaded["theme"] + newline().
```

### CSV

```
define rows as json = [].
append(rows, { "name": "Ada", "age": 36 }).
append(rows, { "name": "Linus", "age": 55 }).

write_csv("people.csv", rows, true).  # include headers
define loaded as json = read_csv("people.csv", true).

print loaded[0]["name"] + newline().
```

`read_csv(..., true)` returns a json array of objects; `read_csv(..., false)` returns a json array of arrays.

---

## 28. Executing External Programs

Use `execute(path)` to run a program or script. If `path` includes a directory separator, it is resolved relative to the `.run` file folder. If there is no separator, Rundell searches the `PATH`.

```
# Launches a program found on PATH (Windows example)
execute("calc.exe").
```

```
# Relative path (uses the .run file directory as the base)
execute("tools\my_tool.exe").
```

**Rules:**
- Do not mix `/` and `\` in the same path.
- If permissions are insufficient, a `PermissionError` is raised.
- Any stdout/stderr from the program is forwarded to the Rundell CLI.

---

## 29. Utility Built-ins

### Math

```
print string(min(3, 7)) + newline().
print string(max(3, 7)) + newline().
print string(clamp(10, 1, 5)) + newline().
print string(sqrt(9)) + newline().
print string(pow(2, 8)) + newline().
```

### Strings

```
define s as string = "alpha,beta,gamma".
print join(split(s, ","), "|") + newline().
print replace(s, ",", ";") + newline().
print string(startswith(s, "alpha")) + newline().
print string(endswith(s, "gamma")) + newline().
print string(contains(s, "beta")) + newline().
```

### JSON helpers

```
define obj as json = { "rows": ["a", "b", "c"], "flag": true }.
print string(has_key(obj, "rows")) + newline().
print string(length(keys(obj))) + newline().
print string(length(values(obj))) + newline().

define arr as json = obj["rows"].
remove_at(arr, 1).
print arr[1] + newline().
```

### Type and file helpers

```
print type(obj) + newline().
print string(isnull(obj)) + newline().

define dirPath as string = "tmp_rundell_demo".
mkdir(dirPath).
print string(exists(dirPath)) + newline().
delete(dirPath).
print string(exists(dirPath)) + newline().

sleep(1).
```

### Datetime helpers

```
define dt as datetime = |2026-04-05T00:00:00-05:00|.
print string(dayofweek(dt)) + newline().
print dateformat("YYYY-MM-DD", adddays(dt, 2)) + newline().
print dateformat("HH", addhours(dt, 5)) + newline().
print string(diffdays(adddays(dt, 2), dt)) + newline().
print timezone(dt) + newline().
```

---

## 30. Debugging Your Programs

The `debug` statement lets you emit timestamped diagnostic messages while your program runs, without mixing them in with `print` output.

### Writing to stdout

```
define count as integer = 42.
debug "count is " + string(count) + newline().
```

Output:
```
2026-04-24 10:11:25> count is 42
```

Every `debug` line is automatically prefixed with the current date and time in `YYYY-MM-DD HH:MM:SS>` format. No newline is added automatically — use `newline()` as shown.

### Writing to a log file

Pass an absolute file path (in parentheses) between `debug` and the message:

```
debug("C:/logs/myapp.log") "count is " + string(count) + newline().
```

- If the file does not exist it is created.
- If the file already exists the new entry is **prepended**, so the latest log entry always appears at the top of the file.

### Using debug in a function

```
define processName(name as string) returns string -->
    debug "processName called with: " + name + newline().
    define result as string = upper(name).
    debug "processName returning: " + result + newline().
    return result.
<--

define output as string = processName("simon").
print output + newline().
```

Console output:
```
2026-04-24 10:11:25> processName called with: simon
2026-04-24 10:11:25> processName returning: SIMON
SIMON
```

### File paths on Windows

Inside a Rundell string, `\n`, `\r`, and `\t` are recognised escape sequences (newline, carriage return, tab). Other backslash sequences are passed through literally, but to be safe use **forward slashes** in log file paths:

```
debug("C:/Users/Simon/Documents/app.log") "started" + newline().
```

### Tip — debug works in GUI programs too

`debug` is available in both console and GUI programs. Use it to trace event-handler calls or inspect variable values while a form is open.

---

## What Next?

- Browse the [examples/](../examples/) folder for more ready-to-run programs, including GUI and REST examples.
- Read the [Language Reference](LANGUAGE_REFERENCE.md) for the complete specification.
- Try the REPL (`rundell` with no arguments) for quick console experiments.
- Use the form designer (`rundell-gui --design`) to build forms visually.
