use super::error::ASTError;
use crate::ast::annotation::Span;
use crate::ast::declaration::Declaration;
use crate::ast::expression::{Expr, Ident};
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
}

impl<A: Clone> Program<A> {
    // the smart constructor returns a program which has the property that
    // declarations only contain identifiers which are bound in previous declarations.
    // this means that if you are building up a context for evaluation in order, you
    // can be sure that all the variables you need to substitute will be bound in the context.
    pub fn new(decls: Vec<Declaration<A>>, expr: Expr<A>) -> Result<Self> {
        let sorted_decls = sort(decls)?;
        let mut decl_ident_set: HashSet<Ident> = HashSet::new();
        for decl in sorted_decls.clone() {
            let ident = decl.get_identifier();
            // check that no variable is declared twice
            if decl_ident_set.contains(&ident) {
                return Err(anyhow!(ASTError::DuplicateIdentifier(ident)));
            }
            let vars = decl.get_dependencies();
            // check that all the variables used in the expression are bound in previous declarations
            for var in vars {
                if !decl_ident_set.contains(&var) {
                    return Err(anyhow!(ASTError::UnboundIdentifier(var)));
                }
            }
            decl_ident_set.insert(ident);
        }

        // check that the expression only uses variables bound in the declarations
        for var in expr.variables() {
            if !decl_ident_set.contains(&var) {
                return Err(anyhow!(ASTError::UnboundIdentifier(var)));
            }
        }

        Ok(Program {
            decls: sorted_decls,
            expr,
        })
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

// build a dependency graph for the declarations where `y -> x` means that
// y appears as a variable in the expression bound to x.
fn dependency_graph<A: Clone>(decls: &Vec<Declaration<A>>) -> DiGraph<Ident, ()> {
    let mut graph = DiGraph::<Ident, ()>::new();
    let mut ix_map = HashMap::new();
    for decl in decls {
        let var = decl.get_identifier();
        let ix = graph.add_node(var.clone());
        ix_map.insert(var, ix);
    }
    for decl in decls {
        let var = decl.get_identifier();
        for dep in decl.get_dependencies() {
            graph.add_edge(ix_map[&dep], ix_map[&var], ());
        }
    }
    graph
}

// find the declaration for a given variable name
pub fn find_declaration<A: Clone>(
    name: Ident,
    decls: Vec<Declaration<A>>,
) -> Option<Declaration<A>> {
    for decl in decls {
        match decl {
            Declaration::VarAssignment { binder, expr } => {
                if binder.var == name {
                    return Some(Declaration::VarAssignment {
                        binder: binder.clone(),
                        expr: expr.clone(),
                    });
                }
            }
            Declaration::PublicVar { binder } => {
                if binder.var == name {
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
fn sort<A: Clone>(decls: Vec<Declaration<A>>) -> Result<Vec<Declaration<A>>, ASTError> {
    let mut sorted = Vec::new();
    let graph = dependency_graph(&decls);
    let top_sorted = toposort(&graph, None);
    match top_sorted {
        Ok(nodes) => {
            for node in nodes {
                let name = graph.node_weight(node).unwrap();
                let decl = match find_declaration(name.clone(), decls.clone()) {
                    Some(decl) => decl,
                    None => return Err(ASTError::UnboundIdentifier(name.clone())),
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
            let c = graph.node_weight(cycle.node_id()).unwrap();
            Err(ASTError::CyclicDependency(c.clone()))
        }
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;
    use crate::ast::{declaration::Binder, error::ASTError};

    #[test]
    fn duplicate_identifier_test() {
        let ident = Ident::new("x");
        let decls: Vec<Declaration<Span>> = vec![
            Declaration::VarAssignment {
                binder: Binder::default(ident.clone()),
                expr: Expr::number_default(1),
            },
            Declaration::VarAssignment {
                binder: Binder::default(ident.clone()),
                expr: Expr::number_default(2),
            },
        ];
        match Program::new(decls, Expr::number_default(1)) {
            Err(err) => match err.downcast_ref() {
                Some(ASTError::DuplicateIdentifier(_)) => (),
                _ => panic!("Expected DuplicateIdentifier error"),
            },
            _ => panic!("Expected DuplicateIdentifier error"),
        };
    }

    #[test]
    fn sort_decl_test() {
        let decls: Vec<Declaration<Span>> = vec![
            Declaration::PublicVar {
                binder: Binder::default(Ident::new("p")),
            },
            Declaration::PublicVar {
                binder: Binder::default(Ident::new("q")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("x")),
                expr: Expr::variable_default(Ident::new("y")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("y")),
                expr: Expr::variable_default(Ident::new("z")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("z")),
                expr: Expr::variable_default(Ident::new("a")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("a")),
                expr: Expr::variable_default(Ident::new("b")),
            },
            Declaration::VarAssignment {
                binder: Binder::default(Ident::new("b")),
                expr: Expr::number_default(1),
            },
        ];
        let sorted = sort(decls).unwrap();
        assert_eq!(
            sorted,
            vec![
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("p")),
                },
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("q")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("b")),
                    expr: Expr::number_default(1),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("a")),
                    expr: Expr::variable_default(Ident::new("b")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("z")),
                    expr: Expr::variable_default(Ident::new("a")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("y")),
                    expr: Expr::variable_default(Ident::new("z")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("x")),
                    expr: Expr::variable_default(Ident::new("y")),
                },
            ]
        );
    }
}
