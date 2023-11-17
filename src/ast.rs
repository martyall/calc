use petgraph::{algo::toposort, graph::DiGraph};
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Pow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum UOpcode {
    Neg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    UnaryOp(UOpcode, Box<Expr>),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}

impl Expr {
    pub fn inline(self, context: &mut HashMap<String, Expr>) -> Self {
        match self {
            Expr::Number(n) => Expr::Number(n),
            Expr::UnaryOp(op, expr) => {
                let expr = expr.inline(context);
                Expr::UnaryOp(op, Box::new(expr))
            }
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.inline(context);
                let rhs = rhs.inline(context);
                Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
            }
            Expr::Variable(name) => {
                let maybe_existing = context.get(&name).cloned();
                let new_expr = if let Some(existing) = maybe_existing {
                    existing.inline(context)
                } else {
                    return Expr::Variable(name.clone());
                };
                context.insert(name.clone(), new_expr.clone());
                new_expr
            }
        }
    }
    pub fn variables(&self) -> Vec<String> {
        match self {
            Expr::Number(_) => vec![],
            Expr::UnaryOp(_, expr) => expr.variables(),
            Expr::BinOp(lhs, _, rhs) => {
                let mut deps = lhs.variables();
                deps.append(&mut rhs.variables());
                deps
            }
            Expr::Variable(name) => vec![name.clone()],
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Declaration {
    VarAssignment(String, Expr),
    PublicVar(String),
}

impl Declaration {
    // get the variable name for this declaration
    pub fn get_identifier(&self) -> String {
        match self {
            Declaration::VarAssignment(name, _) => name.clone(),
            Declaration::PublicVar(name) => name.clone(),
        }
    }

    // get all the free variables in the expression bound in this declaration
    // (none for public variables)
    pub fn get_dependencies(&self) -> Vec<String> {
        match self {
            Declaration::VarAssignment(_, expr) => {
                let mut vars = expr.variables();
                vars.dedup();
                vars
            }
            Declaration::PublicVar(_) => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Program {
    pub decls: Vec<Declaration>,
    pub expr: Expr,
}

impl Program {
    pub fn new(decls: Vec<Declaration>, expr: Expr) -> Self {
        let mut ident_set: HashSet<String> = HashSet::new();
        for decl in &decls {
            let ident = decl.get_identifier();
            if ident_set.contains(&ident) {
                panic!("Duplicate identifier {}", ident);
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
fn find_decl(name: String, decls: Vec<Declaration>) -> Option<Declaration> {
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
fn dependency_graph(decls: &Vec<Declaration>) -> DiGraph<String, ()> {
    let mut graph = DiGraph::<String, ()>::new();
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
                    None => panic!("Declaration for {} not found", name),
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
    #[should_panic(expected = "Duplicate identifier x")]
    fn duplicate_identifier_test() {
        let decls: Vec<Declaration> = vec![
            Declaration::VarAssignment("x".to_string(), Expr::Number(1)),
            Declaration::VarAssignment("x".to_string(), Expr::Number(2)),
        ];
        Program::new(decls, Expr::Number(1));
    }

    #[test]
    fn decl_test() {
        let decls: Vec<Declaration> = vec![
            Declaration::PublicVar("p".to_string()),
            Declaration::PublicVar("q".to_string()),
            Declaration::VarAssignment("x".to_string(), Expr::Variable("y".to_string())),
            Declaration::VarAssignment("y".to_string(), Expr::Variable("z".to_string())),
            Declaration::VarAssignment("z".to_string(), Expr::Variable("a".to_string())),
            Declaration::VarAssignment("a".to_string(), Expr::Variable("b".to_string())),
            Declaration::VarAssignment("b".to_string(), Expr::Number(1)),
        ];
        let sorted = sort(decls);
        assert_eq!(
            sorted,
            vec![
                Declaration::PublicVar("p".to_string()),
                Declaration::PublicVar("q".to_string()),
                Declaration::VarAssignment("b".to_string(), Expr::Number(1)),
                Declaration::VarAssignment("a".to_string(), Expr::Variable("b".to_string())),
                Declaration::VarAssignment("z".to_string(), Expr::Variable("a".to_string())),
                Declaration::VarAssignment("y".to_string(), Expr::Variable("z".to_string())),
                Declaration::VarAssignment("x".to_string(), Expr::Variable("y".to_string())),
            ]
        );
    }
}
