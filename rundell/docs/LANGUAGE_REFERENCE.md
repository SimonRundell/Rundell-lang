# Rundell Language Reference

This document is the complete reference for the Rundell 0.1.0 language.

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
19. [Keywords](#19-keywords)
20. [Error Types](#20-error-types)

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

Comments may appear anywhere except inside a string literal.

---

## 4. Identifiers

- Characters: `a-z`, `A-Z`, `0-9`, `_`
- Must begin with a letter
- Leading underscore `_` is **forbidden**
- Case-sensitive: `myVar` ≠ `MyVar`

Naming styles all work: `camelCase`, `snake_case`, `PascalCase`, `UPPER_CASE`.

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

Multi-line strings are allowed — the raw newlines are included in the value.

### Currency
```
9.99
1000.00
0.50
```
Always stored and displayed to exactly 2 decimal places.

### Boolean
All of the following are accepted:

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

---

## 7. Variables

### Declaration
```
define <name> as [constant] [global] <type> [= <expression>].
```

```
define score       as integer = 100.
define name        as string  = "Simon".
define pi          as constant float = 3.14159.
define sessionCount as global integer = 0.
define uninitialised as string.          # value is null
```

- `constant` — immutable after declaration; attempting `set` raises a TypeError.
- `global` — visible everywhere in the program. Global declarations must appear at the top level (outside any function).
- Without an initial value the variable starts as `null`.
- Re-declaring the same name in the same scope is an error.

---

## 8. Assignment

```
set <name> = <expression>.
set <name>++.                 # increment integer by 1
set <name>--.                 # decrement integer by 1
```

```
set score = score + 10.
set i++.
set i--.
```

`++` and `--` are only valid on `integer` variables. Assigning a value of the wrong type raises a TypeError unless `cast()` is used.

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

### Comparison (return boolean)

`==`  `!=`  `<`  `<=`  `>`  `>=`

### Logical

| Operator | Meaning |
|---|---|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT (prefix) |

### Null check (special expression form)

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

Parentheses override precedence in the usual way.

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

A cast that cannot succeed (e.g. `cast("hello", integer)`) raises a TypeError.

```
define value as integer = 5.
define asFloat as float = cast(value, float).
print string(asFloat) + newline().   # → 5.0
```

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
| `append(collection, value)` | null | Append element to a json array (mutates in place) |

---

## 12. Input and Output

### Print
```
print <expression>.
```
Writes the string representation of the expression to stdout. **No newline is appended automatically** — use `newline()` explicitly.

```
print "Hello, World!" + newline().
print string(42) + newline().
```

### Receive (input)
```
receive <identifier> [with prompt <stringExpression>].
```
Reads one line from stdin into the named variable. The optional `with prompt` clause prints the prompt string before waiting (without a newline). The input is automatically coerced to the variable's declared type; a coercion failure raises TypeError.

```
define name as string.
define age  as integer.

receive name with prompt "Enter your name: ".
receive age  with prompt "Enter your age: ".
```

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

### switch

```
switch <expression> -->
    <case> : <statement>.
    <case> : <statement>.
    else   : <statement>.
<--
```

- Cases are tested top-to-bottom; the first match wins (no fall-through).
- Cases can be **grouped** by stacking them without a body; they share the next body.
- `else` is the default (required).
- Case patterns may be exact values or comparison expressions (`< 18`, `>= 65`, etc.).

```
switch age -->
    < 13  : print "Child" + newline().
    < 18  : print "Teenager" + newline().
    18    :
    19    : print "Newly adult (18 or 19)" + newline().
    else  : print "Adult" + newline().
<--
```

---

## 14. Loops

### for (counted)

```
for <variable> loops (<start>, <end>, <increment>) -->
    <statements>
<--
```

- The loop variable must be declared as `integer` before the loop.
- The range is **inclusive** of both start and end.
- `start`, `end`, and `increment` may be literals or integer expressions.

```
define i as integer.

for i loops (1, 5, 1) -->
    print string(i) + newline().
<--
# output: 1 2 3 4 5
```

### while

```
while <condition> -->
    <statements>
<--
```

```
define count as integer = 0.

while count < 3 -->
    set count++.
    print string(count) + newline().
<--
# output: 1 2 3
```

### for each (collection iterator)

```
for each <variable> in <collectionExpression> -->
    <statements>
<--
```

- The iteration variable is implicitly declared — do **not** pre-declare it.
- The collection expression must be a `json` array.
- Each element is available as a `json` value inside the body.

```
for each item in data["items"] -->
    print item["name"] + newline().
<--
```

---

## 15. Functions

### Declaration

```
define <name>(<param> as <type>, ...) returns <type> -->
    <statements>
<--
```

Use `returns null` for procedures that return no value.

```
define add(a as integer, b as integer) returns integer -->
    return a + b.
<--

define greet(name as string) returns null -->
    print "Hello, " + name + "!" + newline().
    return null.
<--
```

### Calling

```
set result = add(3, 4).
greet("World").
print string(add(10, 20)) + newline().
```

- Functions must be declared before they are called (top-down order).
- Parameters are local and immutable within the body.
- Variables declared inside a function are local to it.
- Globals may be read inside a function.
- Recursion is permitted.
- A `returns null` function that reaches the end of its body without a `return` statement implicitly returns `null`.

---

## 16. Collections (json)

The `json` type is a free-form hierarchical key-value store whose structure mirrors JSON exactly. It can hold objects, arrays, strings, numbers, booleans, and nulls.

### Declaration

```
define myData as json = {
    "retrieved": [
        { "recordId": 1, "firstName": "Simon", "age": 58 },
        { "recordId": 2, "firstName": "James", "age": 25 }
    ]
}.
```

### Access

Both named-key and positional access are valid:

```
myData["retrieved"][0]["firstName"]   # → "Simon"  (string key lookup)
myData[0][0][0]                       # → "Simon"  (positional: 0-based, key-order)
```

### Mutation

```
set myData["newKey"] = "newValue".       # add or update a key
remove myData["oldKey"].                 # remove a key
append(myData["retrieved"], newRecord).  # append to an array
```

### Building JSON from variables

JSON literal syntax requires literal JSON values — variable references are not permitted inside `{ }`. To build a JSON object whose values come from variables, declare the object with placeholder values and then use `set` to fill them in:

```
define name  as string = "Alice".
define phone as string = "555-0101".

define entry as json = { "name": "PLACEHOLDER", "phone": "PLACEHOLDER" }.
set entry["name"]  = name.
set entry["phone"] = phone.
```

### length()

```
length(myData)              # number of top-level keys in the object
length(myData["retrieved"]) # number of elements in the array
```

---

## 17. Error Handling

```
try -->
    <statements>
catch (<ErrorType>) -->
    <statements>
catch (<ErrorType>) -->
    <statements>
finally -->
    <statements>
<--
```

- Multiple `catch` clauses are allowed.
- `finally` is optional; it always runs regardless of whether an error occurred.
- A single `<--` closes the entire structure.
- If no `catch` matches the raised error, the error propagates upward.

```
try -->
    receive age with prompt "Enter your age: ".
    set age = cast(age, integer).
    print "You are " + string(age) + " years old." + newline().
catch (TypeError) -->
    print "Please enter a whole number." + newline().
catch (RuntimeError) -->
    print "Something went wrong." + newline().
finally -->
    print "Done." + newline().
<--
```

---

## 18. Modules

```
import "<path>".
```

- Must appear at the **top** of the file, before any declarations.
- The path is relative to the importing file's directory.
- The `.run` extension is **omitted** in the import statement.
- All global variables and functions defined in the imported file become available.
- Circular imports are detected and cause a parse error.

```
import "mathUtils".
import "lib/strings".
```

---

## 19. Keywords

The following identifiers are reserved and may not be used as variable or function names:

```
define  as  constant  global  set  return  import
if  else  switch  for  while  each  in  loops
true  false  yes  no  TRUE  FALSE  YES  NO
null  and  or  not  is
print  receive  with  prompt
try  catch  finally
integer  float  string  currency  boolean  json
cast  length  newline  abs  floor  ceil  round
substr  upper  lower  trim  append  remove
returns
TypeError  NullError  IndexError  DivisionError  IOError  RuntimeError
```

---

## 20. Error Types

| Type | Raised when |
|---|---|
| `TypeError` | Type mismatch, invalid cast, assigning to a constant, wrong operand types |
| `NullError` | Using a `null` variable in arithmetic or other operations |
| `IndexError` | Collection index out of bounds |
| `DivisionError` | Division or modulo by zero |
| `IOError` | Input/output failure |
| `RuntimeError` | Catch-all for any other runtime error |
