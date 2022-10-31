# swc-plugin-cjs-to-esm

This is a swc plugin that transforms cjs to esm.
Written in rust and compiled to webassembly (with wasm-wasi target).

## Testing

TDD makes life easier here. Most of the tests exist as fixtures in the `tests/fixtures` directory.

```sh
just test
```

You can optionally pass in the name of the folder the test is in to only run that test.

```sh
just test require_ident
```

Note that this is a substring match and it will match `require_ident` and `require_ident_complex`.
If you want to match a folder name exactly then add `__` at the end. ie `require_ident__`.

The canonical name for the tests are `_exec_tests__fixtures__<folder>__input_js`. 

## Development

This package mostly deals with transforming `require` and `module.exports/exports` to `import` and `export` respectively.
Each unique pattern has its own Visitor implementation and method of handling. These visorts are run in a particular order using the `chain!()` macro.

It is advisable to first add or modify testing fixtures to represent the feature being supported and then create or modify
a visitor to handle the pattern.

Some common utilities are provided such as `remove_empty!()` which adds some `visit_mut_*` functions that can remove invalid or empty statements.