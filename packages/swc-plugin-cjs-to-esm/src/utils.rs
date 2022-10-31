// use swc_core::{common::{DUMMY_SP, util::take::Take}, ecma::ast::*};

/**
 * Determines if a given string uses single quotes
 */
// pub fn is_single_quoted(s: &str) -> bool {
//     s.starts_with('\'') && s.ends_with('\'')
// }

// /**
//  * Replaces the quotes of a string with the opposite quotes style.
//  * Warning: this does not account for escaped quotes.
//  */
// pub fn replace_quotes(s: &str) -> String {
//     if is_single_quoted(s) {
//         s.replace("'", "\"")
//     } else {
//         s.replace("\"", "'")
//     }
// }

/**
   Macro for removing empty statements in a visitor class
 */
#[macro_export]
macro_rules! remove_empty {
    () => {
        fn visit_mut_stmt(&mut self, s: &mut Stmt) {
            s.visit_mut_children_with(self);

            match s {
                // Remove declarator statements without any declorations
                Stmt::Decl(Decl::Var(var)) => {
                    if var.decls.is_empty() {
                        s.take();
                    }
                }
                Stmt::Expr(expr) => {
                  if let Expr::Invalid(..) = *expr.expr {
                    s.take();
                  }
                },
                _ => {}
            }
        }

        fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
            stmts.visit_mut_children_with(self);

            // We remove `Stmt::Empty` from the statement list.
            // This is optional, but it's required if you don't want extra `;` in output.
            stmts.retain(|s| {
                // We use `matches` macro as this match is trivial.
                !matches!(s, Stmt::Empty(..))
            });
        }

        fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
            stmts.visit_mut_children_with(self);

            // This is also required, because top-level statements are stored in `Vec<ModuleItem>`.
            stmts.retain(|s| {
                // We use `matches` macro as this match is trivial.
                !matches!(s, ModuleItem::Stmt(Stmt::Empty(..)))
            });
        }
    }
}