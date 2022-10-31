use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};
use swc_core::common::{DUMMY_SP, util::take::Take};

use crate::{remove_empty, utils::if_require_call_expr};

pub struct NoopVisitor;

impl VisitMut for NoopVisitor {}

pub struct TransformModuleExportsNamedExprVisitor {
    pub exports: Vec<ExportDecl>,
}

impl VisitMut for TransformModuleExportsNamedExprVisitor {
    remove_empty!();

    fn visit_mut_expr_stmt(&mut self, e: &mut ExprStmt) {
        e.visit_mut_children_with(self);
        // If left side is invalid then remove
        if let Expr::Assign(a) = &*e.expr {
            match &a.left {
                PatOrExpr::Pat(pat) => {
                    if let Pat::Invalid(..) = **pat {
                        e.expr.take();
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_mut_assign_expr(&mut self, e: &mut AssignExpr) {
        e.visit_mut_children_with(self);
        if e.right.is_object() {
            return;
        }
        // check if module.exports.foo = bar or exports.foo = bar;
        if let PatOrExpr::Pat(pat) = &e.left {
            if let Pat::Expr(expr) = &**pat {
                if let Expr::Member(mem_expr) = &**expr {
                    // Get the identifier from last member expression
                    let ident = mem_expr
                        .prop
                        .as_ident()
                        .unwrap();
                    let mut is_match = false;
                    match &*mem_expr.obj {
                        Expr::Ident(ident) => {
                            if ident.sym == *"exports" {
                                is_match = true;
                            }
                        },
                        Expr::Member(mem_expr_2) => {
                            if let Expr::Ident(obj) = &*mem_expr_2.obj {
                                if let MemberProp::Ident(prop) = &mem_expr_2.prop {
                                    if obj.sym == *"module" && prop.sym == *"exports" {
                                        is_match = true;
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                    if !is_match {
                        return;
                    }
                    // Add ExportDecl
                    self.exports.push(ExportDecl {
                        span: DUMMY_SP,
                        decl: Decl::Var(Box::new(VarDecl {
                            span: DUMMY_SP,
                            kind: VarDeclKind::Const,
                            declare: false,
                            decls: vec![VarDeclarator {
                                span: DUMMY_SP,
                                name: Pat::Ident(ident.to_owned().into()),
                                init: Some(e.right.clone()),
                                definite: false,
                            }],
                        })),
                    });
                    e.take();
                }
            }
        }
    }

    fn visit_mut_module(&mut self, m: &mut Module) {
        m.visit_mut_children_with(self);
        for decl in &self.exports {
            m.body.push(
                ModuleItem::ModuleDecl(
                    ModuleDecl::ExportDecl(decl.to_owned())
                ),
            );
        }
    }
}

pub struct TransformModuleExportsIdentVisitor {
    pub exports: Vec<NamedExport>,
}

impl VisitMut for TransformModuleExportsIdentVisitor {
    remove_empty!();

    fn visit_mut_expr_stmt(&mut self, e: &mut ExprStmt) {
        e.visit_mut_children_with(self);
        // If left side is invalid then remove
        if let Expr::Assign(a) = &*e.expr {
            match &a.left {
                PatOrExpr::Pat(pat) => {
                    if let Pat::Invalid(..) = **pat {
                        e.expr.take();
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_mut_assign_expr(&mut self, e: &mut AssignExpr) {
        e.visit_mut_children_with(self);
        // check if module.exports.foo = bar or exports.foo = bar;
        if let Expr::Ident(rhs) = &*e.right {
            if let PatOrExpr::Pat(pat) = &e.left {
                if let Pat::Expr(expr) = &**pat {
                    if let Expr::Member(mem_expr) = &**expr {
                        // Get the identifier from last member expression
                        let exported = match &mem_expr.prop {
                            MemberProp::Ident(m) => {
                                // Check that rhs.sym is the same as m.sym
                                if rhs.sym == m.sym {
                                    None
                                } else {
                                    Some(ModuleExportName::Ident(m.clone()))
                                }
                            }
                            _ => None
                        };
                        let mut is_match = false;
                        match &*mem_expr.obj {
                            Expr::Ident(ident) => {
                                if ident.sym == *"exports" {
                                    is_match = true;
                                }
                            },
                            Expr::Member(mem_expr_2) => {
                                if let Expr::Ident(obj) = &*mem_expr_2.obj {
                                    if let MemberProp::Ident(prop) = &mem_expr_2.prop {
                                        if obj.sym == *"module" && prop.sym == *"exports" {
                                            is_match = true;
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        if !is_match {
                            return;
                        }
                        self.exports.push(NamedExport {
                            span: DUMMY_SP,
                            src: None,
                            specifiers: vec![
                                ExportSpecifier::Named(ExportNamedSpecifier {
                                    span: DUMMY_SP,
                                    orig: ModuleExportName::Ident(Ident::new(
                                        rhs.sym.to_owned(),
                                        DUMMY_SP,
                                    )),
                                    exported,
                                    is_type_only: false,
                                }),
                            ],
                            type_only: false,
                            asserts: None,
                        });
                        e.take();
                    }
                }
            }
        }
    }

    fn visit_mut_module(&mut self, m: &mut Module) {
        m.visit_mut_children_with(self);
        for decl in &self.exports {
            m.body.push(
                ModuleItem::ModuleDecl(
                    ModuleDecl::ExportNamed(decl.to_owned())
                ),
            );
        }
    }
}

pub struct TransformRequireIdentVisitor {
    pub imports: Vec<ModuleDecl>,
}

impl VisitMut for TransformRequireIdentVisitor {
    remove_empty!();

    // Kinda messy. Could use a refactor?
    fn visit_mut_var_decl(&mut self, d: &mut VarDecl) {
        d.visit_mut_children_with(self);
        // Remove any declarations that match the pattern `const foo = require('foo')`
        d.decls.retain_mut(|decl| {
            if let Pat::Ident(name) = &decl.name {
                if_require_call_expr(
                    &decl.init.as_ref().unwrap(),
                    |_expr, src| {
                        let import = ModuleDecl::Import(ImportDecl {
                            span: DUMMY_SP,
                            specifiers: vec![ImportSpecifier::Namespace(
                                ImportStarAsSpecifier { 
                                    span: DUMMY_SP, 
                                    local: Ident::new(name.sym.to_owned(), DUMMY_SP)
                                }
                            )],
                            src: Box::new(src.to_owned()),
                            type_only: false,
                            asserts: None,
                        });
                        self.imports.push(import);
                        false
                    }
                ).unwrap_or(true)
            } else {
                true
            }
        });
    }

    fn visit_mut_module(&mut self, m: &mut Module) {
        m.visit_mut_children_with(self);
        for decl in &self.imports {
            m.body.insert(
                0,
                // Append `import * as foo from 'foo';`
                ModuleItem::ModuleDecl(decl.to_owned()),
            );
        }
    }
}

pub struct TransformRequireStatementVistor {
    // maintian a list of raw require statements
    pub imports: Vec<Str>
}

impl VisitMut for TransformRequireStatementVistor {
    remove_empty!();
    
    fn visit_mut_module(&mut self, m: &mut Module) {
        m.visit_mut_children_with(self);
        // Add `import 'test';`
        for str in &self.imports {
            m.body.insert(
                0,
                ModuleItem::ModuleDecl(
                    ModuleDecl::Import(
                        ImportDecl {
                            span: DUMMY_SP,
                            specifiers: vec![],
                            src: Box::new(str.to_owned()),
                            type_only: false,
                            asserts: None,
                        }
                    )
                ),
            );
        }
    }

    fn visit_mut_expr_stmt(&mut self, s: &mut ExprStmt) {
        s.visit_mut_children_with(self);
        // print!("{:?}", s);
        if_require_call_expr(
            &s.expr.to_owned(), 
            |_expr, src| {
                // Add to imports vector and mark for deletion 
                self.imports.push(src.to_owned());
                s.expr.take();
            }
        );
    }
}

//  Future TODO. TransformRequireComplexMemberVisitor will work for this for now
pub struct TransformRequireSingleMemberVisitor {

}

// const foo = require('foo').bar; -> import { bar as foo } from 'foo';
impl VisitMut for TransformRequireSingleMemberVisitor {
}

pub struct TransformRequireComplexMemberVisitor {
    pub imports: Vec<ModuleDecl>,
    pub cnt: usize, // used to keep track of unnamed imports
}

// const foo = require('foo').bar.baz; -> import * as foo$ from 'foo'; const foo = foo$.bar.baz;
impl VisitMut for TransformRequireComplexMemberVisitor {
    // basic creates a new import for the module and replace require with the new import
    // Left hand side is only used to determine the name. If no name then it will be named _mod
    remove_empty!();

    fn visit_mut_module(&mut self, m: &mut Module) {
        m.visit_mut_children_with(self);
        for decl in &self.imports {
            m.body.insert(
                0,
                ModuleItem::ModuleDecl(decl.to_owned()),
            );
        }
    }

    fn visit_mut_member_expr(&mut self, e: &mut MemberExpr) {
        e.visit_mut_children_with(self);
        // println!("Here: {:?}", e);
        if_require_call_expr(
            &e.obj.to_owned(),
            |_expr, src| {
                self.cnt += 1;
                let import_ident = Ident::new(format!("_mod${}", self.cnt).into(), DUMMY_SP);
                // TODO: Not sure how to get the name of the variable. Might need to add in more visitors
                e.obj = Box::new(Expr::Ident(import_ident.to_owned()));
                // import * as foo from 'foo';
                let import = ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers: vec![ImportSpecifier::Namespace(
                        ImportStarAsSpecifier { 
                            span: DUMMY_SP, 
                            local: import_ident
                        }
                    )],
                    src: Box::new(src.to_owned()),
                    type_only: false,
                    asserts: None,
                });
                self.imports.push(import);
            }
        );
    }
}