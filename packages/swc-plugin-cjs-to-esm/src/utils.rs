use swc_core::{
    // common::{DUMMY_SP, util::take::Take}, 
    ecma::ast::*
};

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
    Checks if a given string can be used as an identifier
    Note that this is not robust but should be sufficient for 
  
 */
pub fn is_valid_identifier(s: &str) -> bool {
    // check that string does not start with a number
    if s.starts_with(|c: char| c.is_numeric()) {
        return false;
    }
    // Check that string does not contain non-alphanumeric characters
    // alphanumeric here includes unicode characters, _, and $
    if s.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '$') {
        return false;
    }
    return true;
}

/**
    Runs a closure function if the expression is a module.exports assignment.
    Note that exports = abc is not a valid default export.
*/
pub fn if_export_default<T, F:FnOnce() -> T>(node: &AssignExpr, f: F) -> Option<T> {
    // Should probably replace with let chains when stable
    if let PatOrExpr::Pat(pat) = &node.left {
        if let Pat::Expr(expr) = &**pat {
            if let Expr::Member(MemberExpr { obj, prop, .. }) = &**expr {
                if let Expr::Ident(Ident { sym, .. }) = &**obj {
                    if sym == "module" {
                        if let MemberProp::Ident(Ident { sym, .. }) = &*prop {
                            if sym == "exports" {
                                Some(f())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
                
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/**
    Runs a closure function if a given Expression is a call to `require`.
 */
pub fn if_require_call_expr<T, F:FnOnce(&CallExpr, Str) -> T>(expr: &Expr, f: F) -> Option<T> {
    // Should probably replace with let chains when stable
    if let Expr::Call(call_expr) = expr {
        if let Callee::Expr(callee_expr) = &call_expr.callee {
            if let Expr::Ident(Ident { sym, .. }) = &**callee_expr {
                if sym == "require" {
                    if let Some(arg) = call_expr.args.get(0) {
                        if let Expr::Lit(lit) = *arg.expr.to_owned() {
                            let src = match lit {
                                Lit::Str(s) => Some(s),
                                _ => panic!("Unexpected require argument"),
                            };
                            Some(f(call_expr, src.unwrap()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

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