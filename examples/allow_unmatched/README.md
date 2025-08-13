# `%allow_unmatched` Scanner Directive Example

This example demonstrates the use of the `%allow_unmatched` scanner directive in Parol.

## Grammar

See [`allow_unmatched.par`](allow_unmatched.par).

## Test Cases

- `matched.txt`: All input is matched (`1+2+3`)
- `unmatched_start.txt`: Unmatched input at the start (`x1+2`)
- `unmatched_middle.txt`: Unmatched input in the middle (`1+x2`)
- `unmatched_end.txt`: Unmatched input at the end (`1+2x`)
- `unmatched_only.txt`: Only unmatched input (`xyz`)
- `whitespace_only.txt`: Only whitespace, this should fail parsing

With `%allow_unmatched`, unmatched input is skipped and parsing continues. Without it, unmatched input triggers an error.

## To compare behavior

1. Run the parser with this grammar and each test file.
2. Remove `%allow_unmatched` from the grammar and re-run to see errors on unmatched input.