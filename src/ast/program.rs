use crate::ast::declaration::Declaration;
use crate::ast::expression::{Expr, Ident};
use petgraph::{algo::toposort, graph::DiGraph};
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Program {
    pub decls: Vec<Declaration>,
    pub expr: Expr,
}

impl Program {
    pub fn new(decls: Vec<Declaration>, expr: Expr) -> Self {
        let mut ident_set: HashSet<Ident> = HashSet::new();
        for decl in &decls {
            let ident = decl.get_identifier();
            if ident_set.contains(&ident) {
                panic!("Duplicate identifier {:?}", ident);
            }
            ident_set.insert(ident);
        }
        let sorted_decls = sort(decls);
        Program {
            decls: sorted_decls,
            expr,
        }
    }
}

// find the declaration for a given variable name
fn find_decl(name: Ident, decls: Vec<Declaration>) -> Option<Declaration> {
    for decl in decls {
        match decl {
            Declaration::VarAssignment(n, expr) => {
                if n == name {
                    return Some(Declaration::VarAssignment(n.clone(), expr.clone()));
                }
            }
            Declaration::PublicVar(n) => {
                if n == name {
                    return Some(Declaration::PublicVar(n.clone()));
                }
            }
        }
    }
    None
}

// build a dependency graph for the declarations where `y -> x` means that
// y appears as a variable in the expression bound to x.
fn dependency_graph(decls: &Vec<Declaration>) -> DiGraph<Ident, ()> {
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

// sort the declarations so that all the dependencies of a declaration appear
// before it in the list.
fn sort(decls: Vec<Declaration>) -> Vec<Declaration> {
    let mut sorted = Vec::new();
    let graph = dependency_graph(&decls);
    let top_sorted = toposort(&graph, None);
    match top_sorted {
        Ok(nodes) => {
            for node in nodes {
                let name = graph.node_weight(node).unwrap();
                let decl = match find_decl(name.clone(), decls.clone()) {
                    Some(decl) => decl,
                    None => panic!("Declaration for {:?} not found", name),
                };
                if decl.get_dependencies().is_empty() {
                    sorted.insert(0, decl)
                } else {
                    sorted.push(decl);
                }
            }
            sorted
        }
        Err(cycle) => panic!("Cycle detected: {:?}", cycle),
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate identifier Ident(\"x\")")]
    fn duplicate_identifier_test() {
        let decls: Vec<Declaration> = vec![
            Declaration::VarAssignment(Ident::new("x"), Expr::Number(1)),
            Declaration::VarAssignment(Ident::new("x"), Expr::Number(2)),
        ];
        Program::new(decls, Expr::Number(1));
    }

    #[test]
    fn sort_decl_test() {
        let decls: Vec<Declaration> = vec![
            Declaration::PublicVar(Ident::new("p")),
            Declaration::PublicVar(Ident::new("q")),
            Declaration::VarAssignment(Ident::new("x"), Expr::Variable(Ident::new("y"))),
            Declaration::VarAssignment(Ident::new("y"), Expr::Variable(Ident::new("z"))),
            Declaration::VarAssignment(Ident::new("z"), Expr::Variable(Ident::new("a"))),
            Declaration::VarAssignment(Ident::new("a"), Expr::Variable(Ident::new("b"))),
            Declaration::VarAssignment(Ident::new("b"), Expr::Number(1)),
        ];
        let sorted = sort(decls);
        assert_eq!(
            sorted,
            vec![
                Declaration::PublicVar(Ident::new("p")),
                Declaration::PublicVar(Ident::new("q")),
                Declaration::VarAssignment(Ident::new("b"), Expr::Number(1)),
                Declaration::VarAssignment(Ident::new("a"), Expr::Variable(Ident::new("b"))),
                Declaration::VarAssignment(Ident::new("z"), Expr::Variable(Ident::new("a"))),
                Declaration::VarAssignment(Ident::new("y"), Expr::Variable(Ident::new("z"))),
                Declaration::VarAssignment(Ident::new("x"), Expr::Variable(Ident::new("y"))),
            ]
        );
    }
}
