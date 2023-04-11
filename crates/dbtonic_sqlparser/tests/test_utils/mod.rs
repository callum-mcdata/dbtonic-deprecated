

// Re-export everything from `src/test_utils.rs`.
pub use sqlparser::test_utils::*;

// For the test-only macros we take a different approach of keeping them here
// rather than in the library crate.
//
// This is because we don't need any of them to be shared between the
// integration tests (i.e. `tests/*`) and the unit tests (i.e. `src/*`),
// but also because Rust doesn't scope macros to a particular module
// (and while we export internal helpers as sqlparser::test_utils::<...>,
// expecting our users to abstain from relying on them, exporting internal
// macros at the top level, like `sqlparser::nest` was deemed too confusing).

#[macro_export]
macro_rules! nest {
    ($base:expr $(, $join:expr)*) => {
        TableFactor::NestedJoin { table_with_joins: Box::new(TableWithJoins {
            relation: $base,
            joins: vec![$(join($join)),*]
        }), alias: None}
    };
}
