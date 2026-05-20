# Lintbook Rule Status

The active lintbook rule path is Markdown plus Datafox:

- source rule descriptions live in `.lintbook/rules/*.md`
- generated Datafox queries live in `.lintbook/gen/*.df`
- `lintbook compile` validates active rules and writes `.lintbook/gen/*.json`
- `lintbook compile --agent codex` asks Codex to generate missing `.lintbook/gen/*.df` files before compiling
- `lintbook check` runs generated rules without invoking an agent

The previous Rust-implemented language-specific lint crates are parked in the repository for reference and gradual conversion. They are intentionally excluded from the workspace and are not run by `lintbook check`.

Built-in rules are now embedded as Markdown intent plus Datafox assets under `crates/lintbook-rules/builtin`. They are compiled in memory from the embedded assets at runtime and can be listed with `lintbook lints`.

## Active Built-In Rule Facts

Rust custom rules can query these facts:

```text
node(Node, Kind, StartLine, StartColumn, EndLine, EndColumn)
span(Node, StartByte, EndByte)
location(Entity, Line, Column)
text(Node, Text)
trimmedText(Node, TrimmedText)
lowerText(Node, LowercaseText)
collapsedText(Node, WhitespaceCollapsedText)
literal(Node, Kind, RawText, NormalizedText)
intLiteralValue(Node, Value)
child(Parent, Child, Index)
argument(ArgumentsNode, ArgumentNode, Index)
statementExpression(StatementNode, ExpressionNode)
assignment(Node, Left, Right)
comparison(Node, Left, Operator, Right)
rangeBounds(Node, Left, Right)
unitLike(Node)
extremeValue(Node)
moduleDecl(Node, Name)
attributeName(AttributeNode, Name)
attributeOf(TargetNode, AttributeNode, Name)
mistypedLiteralSuffix(Node)
possibleMissingComma(Node)
invisibleCharacter(Node, Name, Codepoint)
parent(Child, Parent)
field(Parent, FieldName, Child)
named(Node)
descendant(Ancestor, Descendant)
nextSibling(Left, Right)
previousSibling(Right, Left)
nextCodeSibling(Left, Right)
sibling(Parent, Left, Right)
lineGap(Left, Right, BlankLineCount)
line(Line, LineNumber, Text, StartByte, EndByte)
nextLine(Line, NextLine)
previousLine(NextLine, Line)
```

`Node`, `Parent`, `Child`, `Ancestor`, and `Descendant` are integer tree-sitter node ids. `Line` is a synthetic negative integer id. Line and column positions are 1-based. `trimmedText` removes leading and trailing whitespace, `lowerText` stores ASCII-lowercased text, and `literal` classifies tree-sitter literal nodes with normalized values. Sibling and line-order facts are adjacent relationships.

`collapsedText` normalizes whitespace for text equality checks. `intLiteralValue` parses integer literal values after removing underscores and Rust integer suffixes. `assignment`, `comparison`, and `rangeBounds` expose common Rust expression operands. `moduleDecl`, `attributeName`, and `attributeOf` expose module declarations and attribute ownership. `unitLike`, `extremeValue`, `mistypedLiteralSuffix`, `possibleMissingComma`, and `invisibleCharacter` are derived helper facts for source-shape checks that are awkward to express as pure joins. `nextCodeSibling` skips comments and anonymous punctuation; `lineGap` counts blank physical lines between adjacent sibling nodes.

## Datafox Syntax

Each `.lintbook/gen/<slug>.df` file contains one Datafox query set, with no Markdown fences, comments, or prose. Use `;` to separate multiple queries that should emit the same lint.

```text
query_set ::= query (";" query)* [";"]
query ::= clause ("," clause)*
clause ::= atom | builtin | "!" atom
atom ::= predicate "(" [term ("," term)*] ")"
builtin ::= term ("=" | "!=" | ">" | ">=" | "<" | "<=") term
builtin ::= ("contains" | "startsWith" | "endsWith" | "matchesRegex" | "notContains" | "notStartsWith" | "notEndsWith" | "notMatchesRegex" | "before" | "after") "(" term "," term ")"
term ::= Variable | "_" | integer | string | bare-lowercase-constant | single-quoted-string
```

Variables start with an uppercase ASCII letter. Lowercase bare identifiers in term position are string constants. `_` is a wildcard. Negated atoms and builtins must come after fact clauses that bind all variables they use.

## Examples

```text
node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")
```

```text
node(Node, "function_item", _, _, _, _), field(Node, "name", Name), text(Name, "main")
```

```text
node(Node, "call_expression", StartLine, _, EndLine, _), EndLine > StartLine
```

```text
line(Node, _, Text, _, _), contains(Text, "////")
```

## Testing Rules

Use `dump-ast` to inspect tree-sitter node kinds and fields:

```sh
cargo run --quiet -p lintbook-cli --bin lintbook -- dump-ast --lang rust path/to/example.rs
```

After writing a `.df`, run:

```sh
cargo run --quiet -p lintbook-cli --bin lintbook -- compile
```

Then check at least one positive example and one negative example:

```sh
cargo run --quiet -p lintbook-cli --bin lintbook -- check --output json positive.rs
cargo run --quiet -p lintbook-cli --bin lintbook -- check --output json negative.rs
```

Positive examples exit nonzero when the rule matches; inspect stdout and confirm the rule id appears.
