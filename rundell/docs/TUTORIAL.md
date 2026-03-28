# Rundell Tutorial

This tutorial takes you from zero to a working knowledge of Rundell. Work through the sections in order — each one builds on the last.

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
14. [Putting It All Together](#14-putting-it-all-together)

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
- `newline()` returns a newline character (`\n`). Because `print` never adds a newline automatically, you always add it yourself.
- `+` between two strings concatenates them.
- Every statement ends with a full stop `.`

---

## 2. Variables and Types

Rundell is **strongly typed** — every variable has a fixed type that you declare up front.

```
define greeting as string  = "Hello".
define count    as integer = 10.
define price    as currency = 4.99.
define ratio    as float   = 0.75.
define active   as boolean = true.
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

Add `global` to declare a variable that is visible everywhere, including inside functions. Global declarations must be at the top level of your file.

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

The `with prompt` clause prints the prompt text before waiting for input. The input is automatically converted to the variable's declared type — so `receive age` (an integer variable) converts the typed text to an integer for you. If the user types something that can't be converted, a `TypeError` is raised.

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

**Integer vs float division:** when both operands are integers, `/` returns an integer (truncated toward zero). To get a float result, cast at least one operand:

```
define result as float = cast(a, float) / cast(b, float).
print string(result) + newline().   # 3.3333333333333335
```

**Currency arithmetic** works like float but always displays with two decimal places:

```
define price    as currency = 9.99.
define quantity as integer  = 3.
define total    as float    = cast(price, float) * cast(quantity, float).
print string(total) + newline().   # 29.97
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

### String concatenation

Use `+` to join strings. Both sides **must** be strings — use `string()` or `cast()` to convert other types first:

```
define age as integer = 25.
print "Age: " + string(age) + newline().
```

### Quotes

You can use `"` or `'` as delimiters. The opening character determines the boundary, so the other one is free inside:

```
print "It's a beautiful day." + newline().
print 'She said "wow".' + newline().
```

---

## 6. Booleans and Logic

```
define isAdult  as boolean = true.
define isStudent as boolean = false.
```

All four spellings of true and false work:

```
define a as boolean = true.
define b as boolean = TRUE.
define c as boolean = yes.
define d as boolean = YES.

define e as boolean = false.
define f as boolean = FALSE.
define g as boolean = no.
define h as boolean = NO.
```

### Logical operators

```
define x as boolean = true.
define y as boolean = false.

print string(x and y) + newline().   # false
print string(x or y)  + newline().   # true
print string(not x)   + newline().   # false
```

### Null check

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

print string(f) + newline().
print s + newline().
print string(b) + newline().
```

`string()` is a handy shorthand for `cast(expr, string)`:

```
print string(3.14) + newline().    # same as cast(3.14, string)
```

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

Note the single `<--` that closes the whole chain.

Conditions can combine logic:

```
define age    as integer = 20.
define hasId  as boolean = true.

if (age >= 18 and hasId) -->
    print "Entry allowed." + newline().
else -->
    print "Entry refused." + newline().
<--
```

### switch

`switch` is a cleaner way to handle many cases against a single value:

```
define day as integer = 3.

switch day -->
    1 : print "Monday" + newline().
    2 : print "Tuesday" + newline().
    3 : print "Wednesday" + newline().
    4 : print "Thursday" + newline().
    5 : print "Friday" + newline().
    6 :
    7 : print "Weekend" + newline().
    else : print "Invalid day" + newline().
<--
```

Days 6 and 7 are **grouped** — they share the `print "Weekend"` body. Cases are tested top to bottom and stop at the first match.

You can use comparison operators in case patterns:

```
define age as integer = 15.

switch age -->
    < 13  : print "Child" + newline().
    < 18  : print "Teenager" + newline().
    < 65  : print "Adult" + newline().
    else  : print "Senior" + newline().
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

The range is **inclusive** — the loop runs when `i == 5`.

Count down by using a negative increment:

```
define i as integer.

for i loops (5, 1, -1) -->
    print string(i) + newline().
<--
# output: 5  4  3  2  1
```

### while — condition loop

```
define count as integer = 1.

while count <= 5 -->
    print string(count) + newline().
    set count++.
<--
# output: 1  2  3  4  5
```

### for each — iterate a collection

```
define fruits as json = { "list": ["apple", "banana", "cherry"] }.

for each fruit in fruits["list"] -->
    print fruit + newline().
<--
# output:
# apple
# banana
# cherry
```

---

## 10. Functions

Functions let you name and reuse a block of logic.

```
define multiply(a as integer, b as integer) returns integer -->
    return a * b.
<--

print string(multiply(6, 7)) + newline().   # 42
```

A function that does not return a value declares `returns null`:

```
define printLine(text as string) returns null -->
    print text + newline().
    return null.
<--

printLine("Hello from a function!").
```

### Recursion

Functions may call themselves:

```
define factorial(n as integer) returns integer -->
    if (n <= 1) -->
        return 1.
    <--
    return n * factorial(n - 1).
<--

print string(factorial(5)) + newline().   # 120
```

### Rules

- Declare functions **before** calling them.
- Parameters are local and cannot be modified inside the function.
- Variables declared inside a function are local to it.
- Global variables can be read (but not re-declared) inside a function.

---

## 11. Collections

The `json` type stores structured data — objects, arrays, and any combination.

```
define library as json = {
    "books": [
        { "title": "Dune",              "author": "Frank Herbert",  "year": 1965 },
        { "title": "Neuromancer",       "author": "William Gibson", "year": 1984 },
        { "title": "The Left Hand of Darkness", "author": "Ursula Le Guin", "year": 1969 }
    ]
}.
```

### Reading values

```
print library["books"][0]["title"] + newline().   # Dune
print string(length(library["books"])) + newline().   # 3
```

### Iterating

```
for each book in library["books"] -->
    print book["title"] + " (" + string(book["year"]) + ")" + newline().
<--
```

### Modifying a collection

```
# Add or update a key
set library["genre"] = "Science Fiction".

# Remove a key
remove library["genre"].

# Append to an array
define newBook as json = { "title": "Foundation", "author": "Isaac Asimov", "year": 1951 }.
append(library["books"], newBook).
```

---

## 12. Error Handling

Wrap risky code in a `try` block and handle specific errors with `catch`:

```
define age as integer.

try -->
    receive age with prompt "Enter your age: ".
    if (age < 0) -->
        print "Age cannot be negative." + newline().
    else -->
        print "You are " + string(age) + " years old." + newline().
    <--
catch (TypeError) -->
    print "Please enter a whole number." + newline().
catch (NullError) -->
    print "No value was provided." + newline().
finally -->
    print "Thank you." + newline().
<--
```

**Error types:**

| Type | Raised when |
|---|---|
| `TypeError` | Type mismatch or invalid cast |
| `NullError` | Using a null variable |
| `IndexError` | Collection index out of bounds |
| `DivisionError` | Division by zero |
| `IOError` | Input/output failure |
| `RuntimeError` | Any other runtime error |

`finally` always runs, whether or not an error occurred — useful for cleanup.

---

## 13. Splitting Code into Modules

As programs grow, split them into separate `.run` files.

**maths.run**
```
define global PI as constant float = 3.14159265.

define circleArea(radius as float) returns float -->
    return PI * radius * radius.
<--

define circlePerimeter(radius as float) returns float -->
    return 2.0 * PI * radius.
<--
```

**main.run**
```
import "maths".

define r as float = 5.0.
print "Area:      " + string(round(circleArea(r), 2)) + newline().
print "Perimeter: " + string(round(circlePerimeter(r), 2)) + newline().
```

Rules:
- `import` statements go at the very top of the file, before anything else.
- The path is relative to the importing file's directory; omit the `.run` extension.
- All global variables and functions from the imported file become available.
- Circular imports are detected and reported as an error.

---

## 14. Putting It All Together

Here is a small but complete program that demonstrates most of what you have learned — a simple contact book:

```
# contact_book.run
# A tiny interactive contact book.

define contacts as json = { "entries": [] }.

define addContact(name as string, phone as string) returns null -->
    define entry as json = { "name": "PLACEHOLDER", "phone": "PLACEHOLDER" }.
    set entry["name"] = name.
    set entry["phone"] = phone.
    append(contacts["entries"], entry).
    return null.
<--

define listContacts() returns null -->
    define total as integer = length(contacts["entries"]).
    if (total == 0) -->
        print "No contacts saved." + newline().
        return null.
    <--
    print "Contacts (" + string(total) + "):" + newline().
    for each c in contacts["entries"] -->
        print "  " + c["name"] + "  —  " + c["phone"] + newline().
    <--
    return null.
<--

addContact("Alice", "555-0101").
addContact("Bob",   "555-0102").
addContact("Carol", "555-0103").

listContacts().
```

Output:
```
Contacts (3):
  Alice  —  555-0101
  Bob    —  555-0102
  Carol  —  555-0103
```

---

## What Next?

- Browse the [examples/](../examples/) folder for more ready-to-run programs.
- Read the [Language Reference](LANGUAGE_REFERENCE.md) for the complete specification.
- Try the REPL (`rundell` with no arguments) for quick experiments.
