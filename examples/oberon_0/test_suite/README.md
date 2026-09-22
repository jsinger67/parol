# Oberon-0 sample suite

This directory contains a compact set of source files used to exercise the Oberon-0 grammar in a small parser-focused test suite.

Files:
- valid_basic.mod: minimal module, variable declaration, assignment, and end-of-module structure
- valid_if_elsif.mod: conditionals including `ELSIF` and `ELSE`
- valid_array_record.mod: record fields, array access, and loops
- invalid_missing_end.mod: intentionally invalid module to confirm parser error handling
