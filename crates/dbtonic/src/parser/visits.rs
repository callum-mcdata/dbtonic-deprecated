// use sqlparser::parser::Parser;
// use sqlparser::dialect::GenericDialect;
// use sqlparser::ast::{ObjectName, Expr};
// use sqlparser::ast::visitor::Visitor;
// use sqlparser::ast::query::TableFactor;
// use core::ops::ControlFlow;
// use sqlparser::ast::Ident;

// // A structure that records statements and relations
//  #[derive(Default)]
// struct DbtRefVisitor {
//     dbt_refs: Vec<Ident>,
// }

// // Visit relations and exprs before children are visited (depth first walk)
// // Note you can also visit statements and visit exprs after children have been visitoed
// impl Visitor for DbtRefVisitor {
//     type Break = ();

//     fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<Self::Break> {
//         if let Some(TableFactor::DbtRef { model_name, .. }) = relation.0.get(0) {
//             self.dbt_refs.push(model_name.clone());
//         }
//         ControlFlow::Continue(())
//     }
// }


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use sqlparser::dialect::GenericDialect;
//     use sqlparser::parser::Parser;
//     use sqlparser::tokenizer::Tokenizer;

//     #[test]
//     fn test_dbt_refs() {
//     let sql = "SELECT a FROM foo where x IN (SELECT y FROM bar)";
//     let statements = Parser::parse_sql(&GenericDialect{}, sql).unwrap();

//     // Drive the visitor through the AST
//     let mut visitor = DbtRefVisitor::default();
//     for statement in &statements {
//         statement.visit(&mut visitor);
//     }

//     dbg!(&visitor.visited);
//     // The visitor has visited statements and expressions in pre-traversal order
//     let expected : Vec<_> = [
//         "PRE: EXPR: a",
//         "PRE: RELATION: foo",
//         "PRE: EXPR: x IN (SELECT y FROM bar)",
//         "PRE: EXPR: x",
//         "PRE: EXPR: y",
//         "PRE: RELATION: bar",
//     ]
//     .into_iter().map(|s| s.to_string()).collect();

//     assert_eq!(visitor.visited, expected);
//     }
// }