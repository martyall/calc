use crate::ast::annotation::HasSourceLoc;
use crate::ast::declaration::Declaration;
use crate::ast::error::ASTError;
use crate::ast::expression::{Expr, Ident};
use crate::ast::typechecker::TypeContext;
use anyhow::{anyhow, Result};
use petgraph::{algo::toposort, graph::DiGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Program<A> {
    pub decls: Vec<Declaration<A>>,
    pub expr: Expr<A>,
}

impl<A: Clone> Program<A> {
    pub fn clear_annotations(self) -> Program<()> {
        Program {
            decls: self
                .decls
                .into_iter()
                .map(|decl| decl.clear_annotations())
                .collect(),
            expr: self.expr.clear_annotations(),
        }
    }

    pub fn public_variable_decls(&self) -> Vec<Declaration<A>> {
        self.decls
            .iter()
            .filter(|decl| match decl {
                Declaration::PublicVar { .. } => true,
                _ => false,
            })
            .cloned()
            .collect()
    }
}

impl<A: Clone + HasSourceLoc> Program<A> {
    pub fn typecheck(&self) -> Result<()> {
        let mut context = TypeContext {
            context: HashMap::new(),
        };
        for decl in &self.decls {
            decl.typecheck(&mut context)?;
        }
        self.expr.typecheck(&context)?;
        Ok(())
    }
}

impl<A: Clone + HasSourceLoc + PartialEq> Program<A> {
    // the smart constructor returns a program which has the property that
    // declarations only contain identifiers which are bound in previous declarations.
    // this means that if you are building up a context for evaluation in order, you
    // can be sure that all the variables you need to substitute will be bound in the context.
    pub fn new(decls: Vec<Declaration<A>>, expr: Expr<A>) -> Result<Self> {
        // check for duplicate bindings
        let mut decls_ident_set: HashSet<Ident> = HashSet::new();
        for decl in decls.clone() {
            let binder = decl.binder();
            if decls_ident_set.contains(&binder.var()) {
                return Err(anyhow!(ASTError::DuplicateIdentifier(
                    binder.ann().source_loc(),
                    binder.var().clone()
                )));
            }
            decls_ident_set.insert(binder.var().clone());
        }
        // sort the declarations so that all the dependencies of a declaration appear
        // before it in the list (i.e. topologically sorted).
        let sorted_decls = sort(decls)?;
        // check that all the variables used in the expression are bound in previous declarations
        // (i.e. verify the assertions above)
        let mut decl_ident_set: HashSet<Ident> = HashSet::new();
        for decl in sorted_decls.clone() {
            let vars = decl.get_dependencies();
            for (var, ann) in vars {
                if !decl_ident_set.contains(&var) {
                    return Err(anyhow!(ASTError::UnboundIdentifier(ann.source_loc(), var)));
                }
            }
            decl_ident_set.insert(decl.binder().var().clone());
        }

        // check that the final expression only uses variables bound in the declarations
        for (var, ann) in expr.variables() {
            if !decl_ident_set.contains(&var) {
                return Err(anyhow!(ASTError::UnboundIdentifier(ann.source_loc(), var)));
            }
        }

        Ok(Program {
            decls: sorted_decls,
            expr,
        })
    }
}

// build a dependency graph for the declarations where `y -> x` means that
// y appears as a variable in the expression bound to x.
fn dependency_graph<A: Clone + PartialEq + HasSourceLoc>(
    decls: &Vec<Declaration<A>>,
) -> Result<DiGraph<(Ident, A), ()>> {
    let mut graph = DiGraph::<(Ident, A), ()>::new();
    let mut ix_map = HashMap::new();
    for decl in decls {
        let binder = decl.binder();
        let ix = graph.add_node((binder.var().clone(), binder.ann().clone()));
        ix_map.insert(binder.var().clone(), ix);
    }
    for decl in decls {
        let binder = decl.binder();
        for dep in decl.get_dependencies() {
            match ix_map.get(&dep.0) {
                Some(ix) => {
                    graph.add_edge(ix.clone(), ix_map[&binder.var()], ());
                }
                None => {
                    return Err(anyhow!(ASTError::UnboundIdentifier(
                        dep.1.source_loc(),
                        dep.0.clone(),
                    )));
                }
            }
        }
    }
    Ok(graph)
}

// find the declaration for a given variable name
pub fn find_declaration<A: Clone>(
    ident: Ident,
    decls: Vec<Declaration<A>>,
) -> Option<Declaration<A>> {
    for decl in decls {
        match decl {
            Declaration::VarAssignment { binder, expr } => {
                if binder.var() == &ident {
                    return Some(Declaration::VarAssignment {
                        binder: binder.clone(),
                        expr: expr.clone(),
                    });
                }
            }
            Declaration::PublicVar { binder } => {
                if binder.var() == &ident {
                    return Some(Declaration::PublicVar {
                        binder: binder.clone(),
                    });
                }
            }
        }
    }
    None
}

// sort the declarations so that all the dependencies of a declaration appear
// before it in the list (i.e. topologically sorted).
fn sort<A: Clone + HasSourceLoc + PartialEq>(
    decls: Vec<Declaration<A>>,
) -> Result<Vec<Declaration<A>>> {
    let mut sorted = Vec::new();
    let graph = dependency_graph(&decls)?;
    let top_sorted = toposort(&graph, None);
    match top_sorted {
        Ok(nodes) => {
            for node in nodes {
                let (ident, ann) = graph.node_weight(node).unwrap();
                let decl = match find_declaration(ident.clone(), decls.clone()) {
                    Some(decl) => decl,
                    None => {
                        return Err(anyhow!(ASTError::UnboundIdentifier(
                            ann.source_loc(),
                            ident.clone()
                        )))
                    }
                };
                if decl.get_dependencies().is_empty() {
                    sorted.insert(0, decl)
                } else {
                    sorted.push(decl);
                }
            }
            Ok(sorted)
        }
        Err(cycle) => {
            let (c, ann) = graph.node_weight(cycle.node_id()).unwrap();
            Err(anyhow!(ASTError::CyclicDependency(
                ann.source_loc(),
                c.clone()
            )))
        }
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;
    use crate::ast::{declaration::Binder, error::ASTError, typechecker::Ty};

    #[test]
    fn duplicate_identifier_test() {
        let ident = Ident::new("x");
        let decls: Vec<Declaration<()>> = vec![
            Declaration::VarAssignment {
                binder: Binder::default(ident.clone(), None),
                expr: Expr::field_default(1),
            },
            Declaration::VarAssignment {
                binder: Binder::default(ident.clone(), None),
                expr: Expr::field_default(2),
            },
        ];
        match Program::new(decls, Expr::field_default(1)) {
            Err(err) => match err.downcast_ref() {
                Some(ASTError::DuplicateIdentifier(_, _)) => (),
                _ => panic!("Expected DuplicateIdentifier error"),
            },
            _ => panic!("Expected DuplicateIdentifier error"),
        };
    }

    #[test]
    fn sort_decl_test() {
        let decls: Vec<Declaration<()>> = vec![
            Declaration::PublicVar {
                binder: Binder::default(Ident::new("p"), Some(Ty::Field)),
            },
            Declaration::PublicVar {
                binder: Binder::default(Ident::new("q"), Some(Ty::Field)),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("x"), None),
                expr: Expr::variable_default(Ident::new("y")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("y"), None),
                expr: Expr::variable_default(Ident::new("z")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("z"), None),
                expr: Expr::variable_default(Ident::new("a")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("a"), None),
                expr: Expr::variable_default(Ident::new("b")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("b"), None),
                expr: Expr::field_default(1),
            },
        ];
        let sorted = sort(decls).unwrap();
        assert_eq!(
            sorted,
            vec![
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("p"), Some(Ty::Field)),
                },
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("q"), Some(Ty::Field)),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("b"), None),
                    expr: Expr::field_default(1),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("a"), None),
                    expr: Expr::variable_default(Ident::new("b")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("z"), None),
                    expr: Expr::variable_default(Ident::new("a")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("y"), None),
                    expr: Expr::variable_default(Ident::new("z")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("x"), None),
                    expr: Expr::variable_default(Ident::new("y")),
                },
            ]
        );
    }
}
