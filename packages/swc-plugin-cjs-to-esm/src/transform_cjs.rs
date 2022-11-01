use swc_core::common::chain;
use swc_core::ecma::visit::{Fold, as_folder};

use crate::visitors::*;

/**
    Transforms require expression statements:

    ```js
    require('foo');
    ```

    to 

    ```js
    import 'foo';
    ```
 */
pub fn transform_require_expr_stmt() -> impl Fold {
    // Why does this require initialization? Do I need a default?
    as_folder(TransformRequireStatementVistor { imports: vec![] })
}

/**
    Transforms require statements with simple identifier:

    ```js
    const foo = require('foo');
    ```

    to 

    ```js
    import * as foo from 'foo';
    ```
 */
pub fn transform_require_ident_to_import() -> impl Fold {
    as_folder(TransformRequireIdentVisitor { imports: vec![] })
}

/**
    Transforms any pure destructured require statements

    ```js
    const { foo, bar: baz } = require('foo');
    ```
    to

    ```js
    import { foo, bar as baz } from 'foo';
    ```
 */
pub fn transform_require_pure_destructure_to_named_imports() -> impl Fold {
    as_folder(TransformPureDestructuredRequireVisitor::new())
}

/**
    Transforms any expression not caught by the other rules

    ```js
    const bar = require('foo').bar;
    const {a, b, c = b} = require('baz');
    ```

    to 

    ```js
    import * as mod$1 from 'foo';
    import * as mod$2 from 'baz';
    const bar = mod$1.bar;
    const {a, b, c = b} = mod$2;
    ```
 */
pub fn transform_require_expression_to_import() -> impl Fold {
    as_folder(TransformRequireFallback::new())
}

/**
    Transforms a default cjs export to a named export and a default export

    ```js
    module.exports = foo;
    ```

    to

    ```js
    export {foo};
    export default foo;
    ```
 */
pub fn transform_module_exports_ident_to_named_export() -> impl Fold {
    as_folder(TransformModuleExportsIdentVisitor { exports: vec![] })
}

/**
    Transforms

    ```js
    module.exports = {foo: bar, baz};
    ```

    to

    ```js
    export {foo as bar, baz};
    ```
 */
pub fn transform_module_exports_object() -> impl Fold {
    as_folder(NoopVisitor)
}

/**
    Transforms

    ```js
    module.exports.foo = 123;
    ```

    to

    ```js
    export const foo = 123;
    ```
 */
pub fn transform_module_exports_named_expression() -> impl Fold {
    as_folder(TransformModuleExportsNamedExprVisitor { exports: vec![] })
}

/**
    Transforms

    ```js
    module.exports = 123;
    ```

    to

    ```js
    export default 123;
    ```

    And emits a warning that this file has as default export.
 */
pub fn transform_module_default_export() -> impl Fold {
    as_folder(TransformModuleDefaultExport::new())
}

/**
   Transforms top-level cjs `require` statements to esm `import`s.
   This chains together several visitors to handle different types of `require` syntaxes.
 */
pub fn transform_imports() -> impl Fold {
    chain!(
        // TODO: Handle transformation of `require('foo').bar();` to `import * as _mod$a1 from 'foo'; foo$123.bar();`
        transform_require_expr_stmt(),
        transform_require_ident_to_import(),
        transform_require_pure_destructure_to_named_imports(),
        // TODO: Handle special case of const a = require('...').default
        // This is a fallback statement and should probably remain last. Handles all other unusual cases.
        transform_require_expression_to_import(),
    )
}

/**
    Transforms top-level cjs `module.exports` (and `exports.`) to esm `export`s.
 */
pub fn transform_exports() -> impl Fold {
    chain!(
        transform_module_exports_ident_to_named_export(),
        transform_module_exports_named_expression(),
        transform_module_exports_object(),
        transform_module_default_export(),
    )
}

/**
    Transforms cjs require/module.exports to esm imports/exports.
 */
pub fn cjs_to_esm() -> impl Fold {
    chain!(
        transform_imports(),
        transform_exports(),
    )
}