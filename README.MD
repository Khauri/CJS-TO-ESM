# CJS2ESM

A package and swc plugin that converts cjs to esm. This plugin can work as a good starting point for migrating old codebases.

It is recommended to run a linter on your codebase afterwards to detect issues and places where manual fixes are necessary.
The recommended eslint rules to add to your linter are
- [import/named](https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/named.md) - To ensure that any named imports actually exist
- [import/no-default-export](https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-default-export.md) - To catch places where default exports had to be used because of ambiguous or invalid esm patterns. Refactoring is advised but in some cases you will probably add an eslint-ignore next-line and move on. Be sure to update any imports that are expecting a named export to the default export in this case.
- [import/no-commonjs](https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-commonjs.md) - For detecting any missed `require` or `module.export` calls. This mostly applies to non-static requires or requires within functions. Audit these usages carefully and if they can be converted to  

Status: ⚠️ 
- ✅ - Completed
- 🚧 - In Progress
- ❌ - Not started

| Transformation | Status |
|---|:---:|
| Simple Imports | ✅ |
| Member Expression Imports | ✅ |
| Named Module Exports to Named Export | ✅ |
| Impurely Destructured Imports | 🚧 |
| Purely Destructured Imports | 🚧 |
| Default Object Export to Named Export | ❌ |
| Complex Export Objects | ❌ |

## Exports

cjs2esm will attempt to turn all exports into named exports. Creating named exports is better for tree shaking.
Using default exports everywhere somewhat defeats the purpose of transitioning to esm.

Note: All export support using `module.exports` and `exports` unless otherwise stated 

### Default Object Export to Named Export
If `module.exports = ` is used and an object is encountered, that object will be transformed on a best-effort basis to named exports.

```js
module.exports = {
  export1,
  export2: export3
};
```

becomes

```js
export {export1, export3 as export2};
```

### Named Module Export to Named Export
Any named module.exports that export a variable reference

```js
module.exports.export1 = export1;
module.exports.export2 = export3;
```

becomes

```js
export {export1};
export {export3 as export2};
```

If an export is not an object 
```js
module.exports.a = () => {};
module.exports.b = 123;
module.exports.c = [];
``` 

Then it will become
```js
export const a = () => {};
export const b = 123;
export const c = [];
```

### Complex Export Objects

If an object in the export is assigned to anything other than a variable _or_ is a reserved keyword.

```js
module.exports = {
  isTest: process.env.NODE_ENV === 'test',
  import: true,
}
```

Then it will be pulled out into another property.

```js
const import$abc = true;
const isTest$abc = process.env.NODE_ENV === 'test';
export {isTest$abc as isTest, import$abc as import};
```

If the object property is not a valid js identifier

```js
module.exports = {
  ['abcd-efg']: true
};
```

### Object spreading and invalid identifiers

Exporting objects that have been spread as well as invalid identifiers are not allowed in esm. 
Unfortunately the only thing that can be done here is to preserve them as the `default`, but this will most likely cause your code to break when importing the object. If any single value behaves this way then the entire object will be preserved as a default export.
Valid properties will still be extracted as named exports.

```js
const value = {a: 1, b: 2, c: 3};
module.exports = {
  abc: true, // valid
  ['abcd-efg']: true, // invalid because abcd-efg is not a valid identifier
  ...value // invalid because spread syntax
};
```

becomes

```js
export const abc = true;
export default {abc, ['abcd-efg']: true, ...value};
```

And you will need to modify manually modify your imports to use the value like so:

```js
import myModule, {abc} from 'my-module';

console.log(abc === myModule.abc); // true
console.log(myModule); // {abc: true, 'abcd-efg': true, a: 1, b: 2, c: 3};
```

## Imports

All imports are assumed to be named imports.

### Simple imports
```js
const myModule = require('my-module');
```

by default becomes

```js
import * as myModule from 'my-module';
```

### Member Expression Imports
If the require statement has member expressions
```js
const myModule = require('my-module').init();
```
Then it will be transformed like so
```js
import * as myModule$abcd from 'my-module';
const myModule = myModule.init();
```

### Purely Destructured Imports
Require statements that are purely destructured (including with aliases)
```js
const {export1, export2: export3} = require('my-module');
```
are transformed into named exports
```js
import {export1, export2 as export3} from 'my-module';
```

### Impurely Destructured Imports
If the require statement performs any non-pure destructuring or does anything remotely complex such as
```js
const {export1, export2 = 'abc'} = require('my-module');
```

Then it will be transformed like so
```js
import * as _myModule$abcd from 'my-module';
const {export1, export2 = 'abc'} = _myModule$abcd;
```
(A hash is appended at the end of the module name to avoid naming conflicts. A _ is prepended to signify the name was auto-generated)

Otherwise, the module will be transformed like this
```js
import {export1, export2} from 'my-module';
// or if using simple aliasing const {export1: ex1, export2} = require('my-module');
import {export1 as ex1, export2} from 'my-module';
```

### Dynamic require
Any require not at the top level will be left alone.
This plugin can optionally prepend 

```js
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
```

to your files, but this only works for node modules. For browser environments you will need to switch to using `import()`.
This package cannot handle this automatically because `import()` is an asynchronous function while `require()` is not, and so
you will likely need to modify your functions.

Although, if your environment supports `import.meta.glob` then it may be possible to convert some requires to eager glob imports.
ie
```js
const module = import.meta.glob('./my-module', {eager: true})['./my-module'];
```
But as of right now these imports only support relative modules and not npm installed ones.

## Development

Prerequisites:
- [just](https://github.com/casey/just) - For automating testing and build scripts. Install with `cargo install just`.
- node & npm - For development of the CLI, publishing, etc

```sh
cargo test
```

If you would like logs from the tests then it's recommended to run the tests in serial because otherwise the logs
get mixed up.

```sh
cargo test -- --nocapture --test-threads 1
```