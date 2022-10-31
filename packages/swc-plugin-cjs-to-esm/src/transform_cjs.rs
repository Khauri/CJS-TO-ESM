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
    Transforms any require statements followed by member accessors

    ```js
    const bar = require('foo').bar;
    ```

    to 

    ```js
    import * as bar$123 from 'foo';
    const bar = bar$123.bar;
    ```
 */
pub fn transform_require_expression_to_import() -> impl Fold {
    as_folder(TransformRequireComplexMemberVisitor { 
        imports: vec![], 
        cnt: 0, 
     })
}

/**
    Transforms simple destructured require statements into named imports

    ```js
    const {foo, bar: baz} = require('foo');
    ```

    to 

    ```js
    import {foo, bar as baz} from 'foo';
    ```
 */
pub fn transform_require_simple_destructure_to_import() -> impl Fold {
    as_folder(NoopVisitor)
}

/**
    Transforms complex destructured require statements such as

    ```js
    const {foo, bar = foo} = require('foo');
    ```

    to 

    ```js
    import * as bar$123 from 'foo';
    const {foo, bar = foo} = bar$123;
    ```
 */
pub fn transform_require_complex_destructure_to_import() -> impl Fold {
    as_folder(NoopVisitor)
}

/**
    Transforms a default cjs export to a named export

    ```js
    module.exports = foo;
    ```

    to

    ```js
    export {foo};
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

    And emits a warning that this file has as default export.
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
pub fn transform_module_exports_expression() -> impl Fold {
    as_folder(NoopVisitor)
}

/**
   Transforms top-level cjs `require` statements to esm `import`s.
   This chains together several visitors to handle different types of `require` syntaxes.
 */
pub fn transform_imports() -> impl Fold {
    chain!(
        transform_require_expr_stmt(),
        transform_require_ident_to_import(),
        transform_require_expression_to_import(),
        transform_require_simple_destructure_to_import(),
        transform_require_complex_destructure_to_import(),
    )
}

/**
    Transforms top-level cjs `module.exports` (and `exports.`) to esm `export`s.
 */
pub fn transform_exports() -> impl Fold {
    chain!(
        transform_module_exports_ident_to_named_export(),
        transform_module_exports_object(),
        transform_module_exports_named_expression(),
        transform_module_exports_expression(),
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