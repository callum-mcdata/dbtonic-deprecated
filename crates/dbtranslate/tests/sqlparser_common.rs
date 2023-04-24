// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(clippy::all)]
//! Test SQL syntax, which all sqlparser dialects must parse in the same way.
//!
//! Note that it does not mean all SQL here is valid in all the dialects, only
//! that 1) it's either standard or widely supported and 2) it can be parsed by
//! sqlparser regardless of the chosen dialect (i.e. it doesn't conflict with
//! dialect-specific parsing rules).

use matches::assert_matches;

use dbtranslate::ast::SelectItem::UnnamedExpr;
use dbtranslate::ast::TableFactor::Pivot;
use dbtranslate::ast::*;
use dbtranslate::dialect::{
    AnsiDialect, BigQueryDialect, GenericDialect,
    PostgreSqlDialect, RedshiftSqlDialect, SnowflakeDialect,
};
use dbtranslate::keywords::ALL_KEYWORDS;
use dbtranslate::parser::{Parser, ParserError};
use test_utils::{
    all_dialects, expr_from_projection, join, number, only, table, table_alias,
    TestedDialects, check_error
};

#[macro_use]
mod test_utils;

#[test]
fn test_insert_errors() {
    let error_message = "INSERT is not supported by dbt-sqlparser";
    let dialect = GenericDialect {};
    let sqls = vec![
        "INSERT customer VALUES (1, 2, 3)",
        "INSERT INTO customer VALUES (1, 2, 3)",
        "INSERT INTO customer VALUES (1, 2, 3), (1, 2, 3)",
        "INSERT INTO public.customer VALUES (1, 2, 3)",
        "INSERT INTO db.public.customer VALUES (1, 2, 3)",
        "INSERT INTO public.customer (id, name, active) VALUES (1, 2, 3)",
    ];

    for sql in sqls {
        check_error(sql, error_message, &dialect );
    }
}

#[test]
fn test_generic_create_errors() {
    let sql = "CREATE TABLE _my_$table (am00unt number)";
    let dialect = GenericDialect {};
    let error_message = "CREATE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_generic_update_errors() {
    let sql = "UPDATE t SET a = 1, b = 2, c = 3 WHERE d";
    let dialect: GenericDialect = GenericDialect {};
    let error_message = "UPDATE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_generic_set_errors() {
    let sql = "SET SOMETHING = '1'";
    let dialect: GenericDialect = GenericDialect {};
    let error_message = "SET is not supported by dbt-sqlparser outside of jinja";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_revoke_errors() {
    let sql = "REVOKE ALL PRIVILEGES ON users, auth FROM analyst CASCADE";
    let dialect = GenericDialect {};
    let error_message = "REVOKE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_merge_errors() {
    let sql = "MERGE INTO s.bar AS dest USING (SELECT * FROM s.foo)";
    let dialect = GenericDialect {};
    let error_message = "MERGE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_delete_errors() {
    let sql = "DELETE FROM table_name";
    let dialect = GenericDialect {};
    let error_message = "DELETE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_copy_errors() {
    let sql = "COPY users TO data.csv";
    let dialect = GenericDialect {};
    let error_message = "COPY is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_close_errors() {
    let sql = r#"CLOSE my_cursor"#;
    let dialect = GenericDialect {};
    let error_message = "CLOSE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_declare_errors() {
    let sql = "DECLARE \"SQL_CUR0x7fa44801bc00\" CURSOR WITH HOLD FOR SELECT 1";
    let dialect = GenericDialect {};
    let error_message = "DECLARE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_fetch_errors() {
    let sql = "SELECT foo FROM bar FETCH FIRST 2 ROWS ONLY";
    let dialect = GenericDialect {};
    let error_message = "Expected end of statement, found: FETCH";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_discard_errors() {
    let sql = "DISCARD ALL";
    let dialect = GenericDialect {};
    let error_message = "DISCARD is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_deallocate_errors() {
    let sql = "DEALLOCATE prepared_statement_name";
    let dialect = GenericDialect {};
    let error_message = "DEALLOCATE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_kill_errors() {
    let sql = "KILL connection_id";
    let dialect = GenericDialect {};
    let error_message = "KILL is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_execute_errors() {
    let sql = "EXECUTE my_procedure()";
    let dialect = GenericDialect {};
    let error_message = "EXECUTE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_use_errors() {
    let sql = "USE my_database";
    let dialect = GenericDialect {};
    let error_message = "USE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_cache_errors() {
    let sql = "CACHE my_table";
    let dialect = GenericDialect {};
    let error_message = "CACHE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_uncache_errors() {
    let sql = "UNCACHE my_table";
    let dialect = GenericDialect {};
    let error_message = "UNCACHE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_start_errors() {
    let sql = "START the_query_to_run";
    let dialect: GenericDialect = GenericDialect {};
    let error_message = "START is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_begin_errors() {
    let sql = "BEGIN my_query";
    let dialect = GenericDialect {};
    let error_message = "BEGIN is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_comment_errors() {
    let sql = "COMMENT on table is 'comment'";
    let dialect = GenericDialect {};
    let error_message = "COMMENT is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_prepare_errors() {
    let sql = "PREPARE my_query";
    let dialect = GenericDialect {};
    let error_message = "PREPARE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_savepoint_errors() {
    let sql = "SAVEPOINT my_query";
    let dialect = GenericDialect {};
    let error_message = "SAVEPOINT is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn parse_select_with_table_alias_as() {
    // AS is optional
    one_statement_parses_to(
        "SELECT a, b, c FROM lineitem l (A, B, C)",
        "SELECT a, b, c FROM lineitem AS l (A, B, C)",
    );
}

#[test]
fn parse_select_with_table_alias() {
    let select = verified_only_select("SELECT a, b, c FROM lineitem AS l (A, B, C)");
    assert_eq!(
        select.projection,
        vec![
            SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("a")),),
            SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("b")),),
            SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("c")),),
        ]
    );
    assert_eq!(
        select.from,
        vec![TableWithJoins {
            relation: TableFactor::Table {
                name: ObjectName(vec![Ident::new("lineitem")]),
                alias: Some(TableAlias {
                    name: Ident::new("l"),
                    columns: vec![Ident::new("A"), Ident::new("B"), Ident::new("C"),],
                }),
                args: None,
                with_hints: vec![],
            },
            joins: vec![],
        }]
    );
}

#[test]
fn parse_invalid_table_name() {
    let ast = all_dialects()
        .run_parser_method("db.public..customer", |parser| parser.parse_object_name());
    assert!(ast.is_err());
}

#[test]
fn parse_no_table_name() {
    let ast = all_dialects().run_parser_method("", |parser| parser.parse_object_name());
    assert!(ast.is_err());
}

#[test]
fn parse_top_level() {
    verified_stmt("SELECT 1");
    verified_stmt("(SELECT 1)");
    verified_stmt("((SELECT 1))");
    verified_stmt("VALUES (1)");
    verified_stmt("VALUES ROW(1, true, 'a'), ROW(2, false, 'b')");
}

#[test]
fn parse_simple_select() {
    let sql = "SELECT id, fname, lname FROM customer WHERE id = 1 LIMIT 5";
    let select = verified_only_select(sql);
    assert!(!select.distinct);
    assert_eq!(3, select.projection.len());
    let select = verified_query(sql);
    assert_eq!(Some(Expr::Value(number("5"))), select.limit);
}

#[test]
fn parse_limit_is_not_an_alias() {
    // In dialects supporting LIMIT it shouldn't be parsed as a table alias
    let ast = verified_query("SELECT id FROM customer LIMIT 1");
    assert_eq!(Some(Expr::Value(number("1"))), ast.limit);

    let ast = verified_query("SELECT 1 LIMIT 5");
    assert_eq!(Some(Expr::Value(number("5"))), ast.limit);
}

#[test]
fn parse_select_distinct() {
    let sql = "SELECT DISTINCT name FROM customer";
    let select = verified_only_select(sql);
    assert!(select.distinct);
    assert_eq!(
        &SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("name"))),
        only(&select.projection)
    );
}

#[test]
fn parse_select_distinct_two_fields() {
    let sql = "SELECT DISTINCT name, id FROM customer";
    let select = verified_only_select(sql);
    assert!(select.distinct);
    assert_eq!(
        &SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("name"))),
        &select.projection[0]
    );
    assert_eq!(
        &SelectItem::UnnamedExpr(Expr::Identifier(Ident::new("id"))),
        &select.projection[1]
    );
}

#[test]
fn parse_select_distinct_tuple() {
    let sql = "SELECT DISTINCT (name, id) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &vec![SelectItem::UnnamedExpr(Expr::Tuple(vec![
            Expr::Identifier(Ident::new("name")),
            Expr::Identifier(Ident::new("id")),
        ]))],
        &select.projection
    );
}

#[test]
fn parse_select_distinct_missing_paren() {
    let result = parse_sql_statements("SELECT DISTINCT (name, id FROM customer");
    assert_eq!(
        ParserError::ParserError("Expected ), found: FROM".to_string()),
        result.unwrap_err(),
    );
}

#[test]
fn parse_select_all() {
    one_statement_parses_to("SELECT ALL name FROM customer", "SELECT name FROM customer");
}

#[test]
fn parse_select_all_distinct() {
    let result = parse_sql_statements("SELECT ALL DISTINCT name FROM customer");
    assert_eq!(
        ParserError::ParserError("Cannot specify both ALL and DISTINCT".to_string()),
        result.unwrap_err(),
    );
}

#[test]
fn parse_select_into() {
    let sql = "SELECT * INTO table0 FROM table1";
    one_statement_parses_to(sql, "SELECT * INTO table0 FROM table1");
    let select = verified_only_select(sql);
    assert_eq!(
        &SelectInto {
            temporary: false,
            unlogged: false,
            table: false,
            name: ObjectName(vec![Ident::new("table0")]),
        },
        only(&select.into)
    );

    let sql = "SELECT * INTO TEMPORARY UNLOGGED TABLE table0 FROM table1";
    one_statement_parses_to(
        sql,
        "SELECT * INTO TEMPORARY UNLOGGED TABLE table0 FROM table1",
    );

    // Do not allow aliases here
    let sql = "SELECT * INTO table0 asdf FROM table1";
    let result = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: asdf".to_string()),
        result.unwrap_err()
    )
}

#[test]
fn parse_select_wildcard() {
    let sql = "SELECT * FROM foo";
    let select = verified_only_select(sql);
    assert_eq!(
        &SelectItem::Wildcard(WildcardAdditionalOptions::default()),
        only(&select.projection)
    );

    let sql = "SELECT foo.* FROM foo";
    let select = verified_only_select(sql);
    assert_eq!(
        &SelectItem::QualifiedWildcard(
            ObjectName(vec![Ident::new("foo")]),
            WildcardAdditionalOptions::default()
        ),
        only(&select.projection)
    );

    let sql = "SELECT myschema.mytable.* FROM myschema.mytable";
    let select = verified_only_select(sql);
    assert_eq!(
        &SelectItem::QualifiedWildcard(
            ObjectName(vec![Ident::new("myschema"), Ident::new("mytable"),]),
            WildcardAdditionalOptions::default(),
        ),
        only(&select.projection)
    );

    let sql = "SELECT * + * FROM foo;";
    let result = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: +".to_string()),
        result.unwrap_err(),
    );
}

#[test]
fn parse_count_wildcard() {
    verified_only_select("SELECT COUNT(*) FROM Order WHERE id = 10");

    verified_only_select(
        "SELECT COUNT(Employee.*) FROM Order JOIN Employee ON Order.employee = Employee.id",
    );
}

#[test]
fn parse_column_aliases() {
    let sql = "SELECT a.col + 1 AS newname FROM foo AS a";
    let select = verified_only_select(sql);
    if let SelectItem::ExprWithAlias {
        expr: Expr::BinaryOp {
            ref op, ref right, ..
        },
        ref alias,
    } = only(&select.projection)
    {
        assert_eq!(&BinaryOperator::Plus, op);
        assert_eq!(&Expr::Value(number("1")), right.as_ref());
        assert_eq!(&Ident::new("newname"), alias);
    } else {
        panic!("Expected ExprWithAlias")
    }

    // alias without AS is parsed correctly:
    one_statement_parses_to("SELECT a.col + 1 newname FROM foo AS a", sql);
}

#[test]
fn test_eof_after_as() {
    let res = parse_sql_statements("SELECT foo AS");
    assert_eq!(
        ParserError::ParserError("Expected an identifier after AS, found: EOF".to_string()),
        res.unwrap_err()
    );

    let res = parse_sql_statements("SELECT 1 FROM foo AS");
    assert_eq!(
        ParserError::ParserError("Expected an identifier after AS, found: EOF".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_select_count_wildcard() {
    let sql = "SELECT COUNT(*) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("COUNT")]),
            args: vec![FunctionArg::Unnamed(FunctionArgExpr::Wildcard)],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_select_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT +x) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("COUNT")]),
            args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::UnaryOp {
                op: UnaryOperator::Plus,
                expr: Box::new(Expr::Identifier(Ident::new("x"))),
            }))],
            over: None,
            distinct: true,
            special: false,
        }),
        expr_from_projection(only(&select.projection))
    );

    one_statement_parses_to(
        "SELECT COUNT(ALL +x) FROM customer",
        "SELECT COUNT(+x) FROM customer",
    );

    let sql = "SELECT COUNT(ALL DISTINCT + x) FROM customer";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Cannot specify both ALL and DISTINCT".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_not() {
    let sql = "SELECT id FROM customer WHERE NOT salary = ''";
    let _ast = verified_only_select(sql);
    //TODO: add assertions
}

#[test]
fn parse_invalid_infix_not() {
    let res = parse_sql_statements("SELECT c FROM t WHERE c NOT (");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: NOT".to_string()),
        res.unwrap_err(),
    );
}

#[test]
fn parse_collate() {
    let sql = "SELECT name COLLATE \"de_DE\" FROM customer";
    assert_matches!(
        only(&all_dialects().verified_only_select(sql).projection),
        SelectItem::UnnamedExpr(Expr::Collate { .. })
    );
}

#[test]
fn parse_collate_after_parens() {
    let sql = "SELECT (name) COLLATE \"de_DE\" FROM customer";
    assert_matches!(
        only(&all_dialects().verified_only_select(sql).projection),
        SelectItem::UnnamedExpr(Expr::Collate { .. })
    );
}

#[test]
fn parse_select_string_predicate() {
    let sql = "SELECT id, fname, lname FROM customer \
               WHERE salary <> 'Not Provided' AND salary <> ''";
    let _ast = verified_only_select(sql);
    //TODO: add assertions
}

#[test]
fn parse_projection_nested_type() {
    let sql = "SELECT customer.address.state FROM foo";
    let _ast = verified_only_select(sql);
    //TODO: add assertions
}

#[test]
fn parse_null_in_select() {
    let sql = "SELECT NULL";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Value(Value::Null),
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_exponent_in_select() -> Result<(), ParserError> {
    // all except Hive, as it allows numbers to start an identifier
    let dialects = TestedDialects {
        dialects: vec![
            Box::new(AnsiDialect {}),
            Box::new(BigQueryDialect {}),
            Box::new(GenericDialect {}),
            Box::new(PostgreSqlDialect {}),
            Box::new(RedshiftSqlDialect {}),
            Box::new(SnowflakeDialect {}),
        ],
    };
    let sql = "SELECT 10e-20, 1e3, 1e+3, 1e3a, 1e, 0.5e2";
    let mut select = dialects.parse_sql_statements(sql)?;

    let select = match select.pop().unwrap() {
        Statement::Query(inner) => *inner,
        _ => panic!("Expected Query"),
    };
    let select = match *select.body {
        SetExpr::Select(inner) => *inner,
        _ => panic!("Expected SetExpr::Select"),
    };

    assert_eq!(
        &vec![
            SelectItem::UnnamedExpr(Expr::Value(number("10e-20"))),
            SelectItem::UnnamedExpr(Expr::Value(number("1e3"))),
            SelectItem::UnnamedExpr(Expr::Value(number("1e+3"))),
            SelectItem::ExprWithAlias {
                expr: Expr::Value(number("1e3")),
                alias: Ident::new("a")
            },
            SelectItem::ExprWithAlias {
                expr: Expr::Value(number("1")),
                alias: Ident::new("e")
            },
            SelectItem::UnnamedExpr(Expr::Value(number("0.5e2"))),
        ],
        &select.projection
    );

    Ok(())
}

#[test]
fn parse_select_with_date_column_name() {
    let sql = "SELECT date";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Identifier(Ident {
            value: "date".into(),
            quote_style: None,
        }),
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_escaped_single_quote_string_predicate() {
    use self::BinaryOperator::*;
    let sql = "SELECT id, fname, lname FROM customer \
               WHERE salary <> 'Jim''s salary'";
    let ast = verified_only_select(sql);
    assert_eq!(
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("salary"))),
            op: NotEq,
            right: Box::new(Expr::Value(Value::SingleQuotedString(
                "Jim's salary".to_string()
            ))),
        }),
        ast.selection,
    );
}

#[test]
fn parse_number() {
    let expr = verified_expr("1.0");

    #[cfg(feature = "bigdecimal")]
    assert_eq!(
        expr,
        Expr::Value(Value::Number(bigdecimal::BigDecimal::from(1), false))
    );

    #[cfg(not(feature = "bigdecimal"))]
    assert_eq!(expr, Expr::Value(Value::Number("1.0".into(), false)));
}

#[test]
fn parse_compound_expr_1() {
    use self::BinaryOperator::*;
    use self::Expr::*;
    let sql = "a + b * c";
    assert_eq!(
        BinaryOp {
            left: Box::new(Identifier(Ident::new("a"))),
            op: Plus,
            right: Box::new(BinaryOp {
                left: Box::new(Identifier(Ident::new("b"))),
                op: Multiply,
                right: Box::new(Identifier(Ident::new("c"))),
            }),
        },
        verified_expr(sql)
    );
}

#[test]
fn parse_compound_expr_2() {
    use self::BinaryOperator::*;
    use self::Expr::*;
    let sql = "a * b + c";
    assert_eq!(
        BinaryOp {
            left: Box::new(BinaryOp {
                left: Box::new(Identifier(Ident::new("a"))),
                op: Multiply,
                right: Box::new(Identifier(Ident::new("b"))),
            }),
            op: Plus,
            right: Box::new(Identifier(Ident::new("c"))),
        },
        verified_expr(sql)
    );
}

#[test]
fn parse_unary_math() {
    use self::Expr::*;
    let sql = "-a + -b";
    assert_eq!(
        BinaryOp {
            left: Box::new(UnaryOp {
                op: UnaryOperator::Minus,
                expr: Box::new(Identifier(Ident::new("a"))),
            }),
            op: BinaryOperator::Plus,
            right: Box::new(UnaryOp {
                op: UnaryOperator::Minus,
                expr: Box::new(Identifier(Ident::new("b"))),
            }),
        },
        verified_expr(sql)
    );
}

#[test]
fn parse_is_null() {
    use self::Expr::*;
    let sql = "a IS NULL";
    assert_eq!(
        IsNull(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );
}

#[test]
fn parse_is_not_null() {
    use self::Expr::*;
    let sql = "a IS NOT NULL";
    assert_eq!(
        IsNotNull(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );
}

#[test]
fn parse_is_distinct_from() {
    use self::Expr::*;
    let sql = "a IS DISTINCT FROM b";
    assert_eq!(
        IsDistinctFrom(
            Box::new(Identifier(Ident::new("a"))),
            Box::new(Identifier(Ident::new("b"))),
        ),
        verified_expr(sql)
    );
}

#[test]
fn parse_is_not_distinct_from() {
    use self::Expr::*;
    let sql = "a IS NOT DISTINCT FROM b";
    assert_eq!(
        IsNotDistinctFrom(
            Box::new(Identifier(Ident::new("a"))),
            Box::new(Identifier(Ident::new("b"))),
        ),
        verified_expr(sql)
    );
}

#[test]
fn parse_not_precedence() {
    // NOT has higher precedence than OR/AND, so the following must parse as (NOT true) OR true
    let sql = "NOT true OR true";
    assert_matches!(
        verified_expr(sql),
        Expr::BinaryOp {
            op: BinaryOperator::Or,
            ..
        }
    );

    // But NOT has lower precedence than comparison operators, so the following parses as NOT (a IS NULL)
    let sql = "NOT a IS NULL";
    assert_matches!(
        verified_expr(sql),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            ..
        }
    );

    // NOT has lower precedence than BETWEEN, so the following parses as NOT (1 NOT BETWEEN 1 AND 2)
    let sql = "NOT 1 NOT BETWEEN 1 AND 2";
    assert_eq!(
        verified_expr(sql),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(Expr::Between {
                expr: Box::new(Expr::Value(number("1"))),
                low: Box::new(Expr::Value(number("1"))),
                high: Box::new(Expr::Value(number("2"))),
                negated: true,
            }),
        },
    );

    // NOT has lower precedence than LIKE, so the following parses as NOT ('a' NOT LIKE 'b')
    let sql = "NOT 'a' NOT LIKE 'b'";
    assert_eq!(
        verified_expr(sql),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(Expr::Like {
                expr: Box::new(Expr::Value(Value::SingleQuotedString("a".into()))),
                negated: true,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("b".into()))),
                escape_char: None,
            }),
        },
    );

    // NOT has lower precedence than IN, so the following parses as NOT (a NOT IN 'a')
    let sql = "NOT a NOT IN ('a')";
    assert_eq!(
        verified_expr(sql),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(Expr::InList {
                expr: Box::new(Expr::Identifier("a".into())),
                list: vec![Expr::Value(Value::SingleQuotedString("a".into()))],
                negated: true,
            }),
        },
    );
}

#[test]
fn parse_null_like() {
    let sql = "SELECT \
            column1 LIKE NULL AS col_null, \
            NULL LIKE column1 AS null_col \
        FROM customers";
    let select = verified_only_select(sql);
    assert_eq!(
        SelectItem::ExprWithAlias {
            expr: Expr::Like {
                expr: Box::new(Expr::Identifier(Ident::new("column1"))),
                negated: false,
                pattern: Box::new(Expr::Value(Value::Null)),
                escape_char: None,
            },
            alias: Ident {
                value: "col_null".to_owned(),
                quote_style: None,
            },
        },
        select.projection[0]
    );
    assert_eq!(
        SelectItem::ExprWithAlias {
            expr: Expr::Like {
                expr: Box::new(Expr::Value(Value::Null)),
                negated: false,
                pattern: Box::new(Expr::Identifier(Ident::new("column1"))),
                escape_char: None,
            },
            alias: Ident {
                value: "null_col".to_owned(),
                quote_style: None,
            },
        },
        select.projection[1]
    );
}

#[test]
fn parse_ilike() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}ILIKE '%a'",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::ILike {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: None,
            },
            select.selection.unwrap()
        );

        // Test with escape char
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}ILIKE '%a' ESCAPE '^'",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::ILike {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: Some('^'),
            },
            select.selection.unwrap()
        );

        // This statement tests that ILIKE and NOT ILIKE have the same precedence.
        // This was previously mishandled (#81).
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}ILIKE '%a' IS NULL",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::IsNull(Box::new(Expr::ILike {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: None,
            })),
            select.selection.unwrap()
        );
    }
    chk(false);
    chk(true);
}

#[test]
fn parse_in_list() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE segment {}IN ('HIGH', 'MED')",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::InList {
                expr: Box::new(Expr::Identifier(Ident::new("segment"))),
                list: vec![
                    Expr::Value(Value::SingleQuotedString("HIGH".to_string())),
                    Expr::Value(Value::SingleQuotedString("MED".to_string())),
                ],
                negated,
            },
            select.selection.unwrap()
        );
    }
    chk(false);
    chk(true);
}

#[test]
fn parse_in_subquery() {
    let sql = "SELECT * FROM customers WHERE segment IN (SELECT segm FROM bar)";
    let select = verified_only_select(sql);
    assert_eq!(
        Expr::InSubquery {
            expr: Box::new(Expr::Identifier(Ident::new("segment"))),
            subquery: Box::new(verified_query("SELECT segm FROM bar")),
            negated: false,
        },
        select.selection.unwrap()
    );
}

#[test]
fn parse_in_unnest() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE segment {}IN UNNEST(expr)",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::InUnnest {
                expr: Box::new(Expr::Identifier(Ident::new("segment"))),
                array_expr: Box::new(verified_expr("expr")),
                negated,
            },
            select.selection.unwrap()
        );
    }
    chk(false);
    chk(true);
}

#[test]
fn parse_in_error() {
    // <expr> IN <expr> is no valid
    let sql = "SELECT * FROM customers WHERE segment in segment";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected (, found: segment".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_string_agg() {
    let sql = "SELECT a || b";

    let select = verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("a"))),
            op: BinaryOperator::StringConcat,
            right: Box::new(Expr::Identifier(Ident::new("b"))),
        }),
        select.projection[0]
    );
}

/// selects all dialects but PostgreSQL
pub fn all_dialects_but_pg() -> TestedDialects {
    TestedDialects {
        dialects: all_dialects()
            .dialects
            .into_iter()
            .filter(|x| !x.is::<PostgreSqlDialect>())
            .collect(),
    }
}

#[test]
fn parse_bitwise_ops() {
    let bitwise_ops = &[
        ("^", BinaryOperator::BitwiseXor, all_dialects_but_pg()),
        ("|", BinaryOperator::BitwiseOr, all_dialects()),
        ("&", BinaryOperator::BitwiseAnd, all_dialects()),
    ];

    for (str_op, op, dialects) in bitwise_ops {
        let select = dialects.verified_only_select(&format!("SELECT a {} b", &str_op));
        assert_eq!(
            SelectItem::UnnamedExpr(Expr::BinaryOp {
                left: Box::new(Expr::Identifier(Ident::new("a"))),
                op: op.clone(),
                right: Box::new(Expr::Identifier(Ident::new("b"))),
            }),
            select.projection[0]
        );
    }
}

#[test]
fn parse_binary_any() {
    let select = verified_only_select("SELECT a = ANY(b)");
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("a"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::AnyOp(Box::new(Expr::Identifier(Ident::new("b"))))),
        }),
        select.projection[0]
    );
}

#[test]
fn parse_binary_all() {
    let select = verified_only_select("SELECT a = ALL(b)");
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("a"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::AllOp(Box::new(Expr::Identifier(Ident::new("b"))))),
        }),
        select.projection[0]
    );
}

#[test]
fn parse_logical_xor() {
    let sql = "SELECT true XOR true, false XOR false, true XOR false, false XOR true";
    let select = verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Value(Value::Boolean(true))),
            op: BinaryOperator::Xor,
            right: Box::new(Expr::Value(Value::Boolean(true))),
        }),
        select.projection[0]
    );
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Value(Value::Boolean(false))),
            op: BinaryOperator::Xor,
            right: Box::new(Expr::Value(Value::Boolean(false))),
        }),
        select.projection[1]
    );
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Value(Value::Boolean(true))),
            op: BinaryOperator::Xor,
            right: Box::new(Expr::Value(Value::Boolean(false))),
        }),
        select.projection[2]
    );
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::BinaryOp {
            left: Box::new(Expr::Value(Value::Boolean(false))),
            op: BinaryOperator::Xor,
            right: Box::new(Expr::Value(Value::Boolean(true))),
        }),
        select.projection[3]
    );
}

#[test]
fn parse_between() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE age {}BETWEEN 25 AND 32",
            if negated { "NOT " } else { "" }
        );
        let select = verified_only_select(sql);
        assert_eq!(
            Expr::Between {
                expr: Box::new(Expr::Identifier(Ident::new("age"))),
                low: Box::new(Expr::Value(number("25"))),
                high: Box::new(Expr::Value(number("32"))),
                negated,
            },
            select.selection.unwrap()
        );
    }
    chk(false);
    chk(true);
}

#[test]
fn parse_between_with_expr() {
    use self::BinaryOperator::*;
    let sql = "SELECT * FROM t WHERE 1 BETWEEN 1 + 2 AND 3 + 4 IS NULL";
    let select = verified_only_select(sql);
    assert_eq!(
        Expr::IsNull(Box::new(Expr::Between {
            expr: Box::new(Expr::Value(number("1"))),
            low: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(number("1"))),
                op: Plus,
                right: Box::new(Expr::Value(number("2"))),
            }),
            high: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(number("3"))),
                op: Plus,
                right: Box::new(Expr::Value(number("4"))),
            }),
            negated: false,
        })),
        select.selection.unwrap()
    );

    let sql = "SELECT * FROM t WHERE 1 = 1 AND 1 + x BETWEEN 1 AND 2";
    let select = verified_only_select(sql);
    assert_eq!(
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(number("1"))),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::Value(number("1"))),
            }),
            op: BinaryOperator::And,
            right: Box::new(Expr::Between {
                expr: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(number("1"))),
                    op: BinaryOperator::Plus,
                    right: Box::new(Expr::Identifier(Ident::new("x"))),
                }),
                low: Box::new(Expr::Value(number("1"))),
                high: Box::new(Expr::Value(number("2"))),
                negated: false,
            }),
        },
        select.selection.unwrap(),
    )
}

#[test]
fn parse_tuples() {
    let sql = "SELECT (1, 2), (1), ('foo', 3, baz)";
    let select = verified_only_select(sql);
    assert_eq!(
        vec![
            SelectItem::UnnamedExpr(Expr::Tuple(vec![
                Expr::Value(number("1")),
                Expr::Value(number("2")),
            ])),
            SelectItem::UnnamedExpr(Expr::Nested(Box::new(Expr::Value(number("1"))))),
            SelectItem::UnnamedExpr(Expr::Tuple(vec![
                Expr::Value(Value::SingleQuotedString("foo".into())),
                Expr::Value(number("3")),
                Expr::Identifier(Ident::new("baz")),
            ])),
        ],
        select.projection
    );
}

#[test]
fn parse_tuple_invalid() {
    let sql = "select (1";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected ), found: EOF".to_string()),
        res.unwrap_err()
    );

    let sql = "select (), 2";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected an expression:, found: )".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_select_order_by() {
    fn chk(sql: &str) {
        let select = verified_query(sql);
        assert_eq!(
            vec![
                OrderByExpr {
                    expr: Expr::Identifier(Ident::new("lname")),
                    asc: Some(true),
                    nulls_first: None,
                },
                OrderByExpr {
                    expr: Expr::Identifier(Ident::new("fname")),
                    asc: Some(false),
                    nulls_first: None,
                },
                OrderByExpr {
                    expr: Expr::Identifier(Ident::new("id")),
                    asc: None,
                    nulls_first: None,
                },
            ],
            select.order_by
        );
    }
    chk("SELECT id, fname, lname FROM customer WHERE id < 5 ORDER BY lname ASC, fname DESC, id");
    // make sure ORDER is not treated as an alias
    chk("SELECT id, fname, lname FROM customer ORDER BY lname ASC, fname DESC, id");
    chk("SELECT 1 AS lname, 2 AS fname, 3 AS id, 4 ORDER BY lname ASC, fname DESC, id");
}

#[test]
fn parse_select_order_by_limit() {
    let sql = "SELECT id, fname, lname FROM customer WHERE id < 5 \
               ORDER BY lname ASC, fname DESC LIMIT 2";
    let select = verified_query(sql);
    assert_eq!(
        vec![
            OrderByExpr {
                expr: Expr::Identifier(Ident::new("lname")),
                asc: Some(true),
                nulls_first: None,
            },
            OrderByExpr {
                expr: Expr::Identifier(Ident::new("fname")),
                asc: Some(false),
                nulls_first: None,
            },
        ],
        select.order_by
    );
    assert_eq!(Some(Expr::Value(number("2"))), select.limit);
}

#[test]
fn parse_select_order_by_nulls_order() {
    let sql = "SELECT id, fname, lname FROM customer WHERE id < 5 \
               ORDER BY lname ASC NULLS FIRST, fname DESC NULLS LAST LIMIT 2";
    let select = verified_query(sql);
    assert_eq!(
        vec![
            OrderByExpr {
                expr: Expr::Identifier(Ident::new("lname")),
                asc: Some(true),
                nulls_first: Some(true),
            },
            OrderByExpr {
                expr: Expr::Identifier(Ident::new("fname")),
                asc: Some(false),
                nulls_first: Some(false),
            },
        ],
        select.order_by
    );
    assert_eq!(Some(Expr::Value(number("2"))), select.limit);
}

#[test]
fn parse_select_group_by() {
    let sql = "SELECT id, fname, lname FROM customer GROUP BY lname, fname";
    let select = verified_only_select(sql);
    assert_eq!(
        vec![
            Expr::Identifier(Ident::new("lname")),
            Expr::Identifier(Ident::new("fname")),
        ],
        select.group_by
    );

    // Tuples can also be in the set
    one_statement_parses_to(
        "SELECT id, fname, lname FROM customer GROUP BY (lname, fname)",
        "SELECT id, fname, lname FROM customer GROUP BY (lname, fname)",
    );
}

#[test]
fn parse_select_group_by_grouping_sets() {
    let dialects = TestedDialects {
        dialects: vec![Box::new(GenericDialect {}), Box::new(PostgreSqlDialect {})],
    };
    let sql =
        "SELECT brand, size, sum(sales) FROM items_sold GROUP BY size, GROUPING SETS ((brand), (size), ())";
    let select = dialects.verified_only_select(sql);
    assert_eq!(
        vec![
            Expr::Identifier(Ident::new("size")),
            Expr::GroupingSets(vec![
                vec![Expr::Identifier(Ident::new("brand"))],
                vec![Expr::Identifier(Ident::new("size"))],
                vec![],
            ]),
        ],
        select.group_by
    );
}

#[test]
fn parse_select_group_by_rollup() {
    let dialects = TestedDialects {
        dialects: vec![Box::new(GenericDialect {}), Box::new(PostgreSqlDialect {})],
    };
    let sql = "SELECT brand, size, sum(sales) FROM items_sold GROUP BY size, ROLLUP (brand, size)";
    let select = dialects.verified_only_select(sql);
    assert_eq!(
        vec![
            Expr::Identifier(Ident::new("size")),
            Expr::Rollup(vec![
                vec![Expr::Identifier(Ident::new("brand"))],
                vec![Expr::Identifier(Ident::new("size"))],
            ]),
        ],
        select.group_by
    );
}

#[test]
fn parse_select_group_by_cube() {
    let dialects = TestedDialects {
        dialects: vec![Box::new(GenericDialect {}), Box::new(PostgreSqlDialect {})],
    };
    let sql = "SELECT brand, size, sum(sales) FROM items_sold GROUP BY size, CUBE (brand, size)";
    let select = dialects.verified_only_select(sql);
    assert_eq!(
        vec![
            Expr::Identifier(Ident::new("size")),
            Expr::Cube(vec![
                vec![Expr::Identifier(Ident::new("brand"))],
                vec![Expr::Identifier(Ident::new("size"))],
            ]),
        ],
        select.group_by
    );
}

#[test]
fn parse_select_having() {
    let sql = "SELECT foo FROM bar GROUP BY foo HAVING COUNT(*) > 1";
    let select = verified_only_select(sql);
    assert_eq!(
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Function(Function {
                name: ObjectName(vec![Ident::new("COUNT")]),
                args: vec![FunctionArg::Unnamed(FunctionArgExpr::Wildcard)],
                over: None,
                distinct: false,
                special: false,
            })),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Value(number("1"))),
        }),
        select.having
    );

    let sql = "SELECT 'foo' HAVING 1 = 1";
    let select = verified_only_select(sql);
    assert!(select.having.is_some());
}

#[cfg(feature = "bigdecimal")]
#[test]
fn parse_select_qualify() {
    let sql = "SELECT i, p, o FROM qt QUALIFY ROW_NUMBER() OVER (PARTITION BY p ORDER BY o) = 1";
    let select = verified_only_select(sql);
    assert_eq!(
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Function(Function {
                name: ObjectName(vec![Ident::new("ROW_NUMBER")]),
                args: vec![],
                over: Some(WindowSpec {
                    partition_by: vec![Expr::Identifier(Ident::new("p"))],
                    order_by: vec![OrderByExpr {
                        expr: Expr::Identifier(Ident::new("o")),
                        asc: None,
                        nulls_first: None,
                    }],
                    window_frame: None,
                }),
                distinct: false,
                special: false,
            })),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Value(number("1"))),
        }),
        select.qualify
    );

    let sql = "SELECT i, p, o, ROW_NUMBER() OVER (PARTITION BY p ORDER BY o) AS row_num FROM qt QUALIFY row_num = 1";
    let select = verified_only_select(sql);
    assert_eq!(
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("row_num"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Value(number("1"))),
        }),
        select.qualify
    );
}

#[test]
fn parse_limit_accepts_all() {
    one_statement_parses_to(
        "SELECT id, fname, lname FROM customer WHERE id = 1 LIMIT ALL",
        "SELECT id, fname, lname FROM customer WHERE id = 1",
    );
}

#[test]
fn parse_cast() {
    let sql = "SELECT CAST(id AS BIGINT) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::BigInt(None),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS TINYINT) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::TinyInt(None),
        },
        expr_from_projection(only(&select.projection))
    );

    one_statement_parses_to(
        "SELECT CAST(id AS MEDIUMINT) FROM customer",
        "SELECT CAST(id AS MEDIUMINT) FROM customer",
    );

    one_statement_parses_to(
        "SELECT CAST(id AS BIGINT) FROM customer",
        "SELECT CAST(id AS BIGINT) FROM customer",
    );

    verified_stmt("SELECT CAST(id AS NUMERIC) FROM customer");

    verified_stmt("SELECT CAST(id AS DEC) FROM customer");

    verified_stmt("SELECT CAST(id AS DECIMAL) FROM customer");

    let sql = "SELECT CAST(id AS NVARCHAR(50)) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Nvarchar(Some(50)),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS CLOB) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Clob(None),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS CLOB(50)) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Clob(Some(50)),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS BINARY(50)) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Binary(Some(50)),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS VARBINARY(50)) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Varbinary(Some(50)),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS BLOB) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Blob(None),
        },
        expr_from_projection(only(&select.projection))
    );

    let sql = "SELECT CAST(id AS BLOB(50)) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Cast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::Blob(Some(50)),
        },
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_try_cast() {
    let sql = "SELECT TRY_CAST(id AS BIGINT) FROM customer";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TryCast {
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
            data_type: DataType::BigInt(None),
        },
        expr_from_projection(only(&select.projection))
    );
    verified_stmt("SELECT TRY_CAST(id AS BIGINT) FROM customer");

    verified_stmt("SELECT TRY_CAST(id AS NUMERIC) FROM customer");

    verified_stmt("SELECT TRY_CAST(id AS DEC) FROM customer");

    verified_stmt("SELECT TRY_CAST(id AS DECIMAL) FROM customer");
}

#[test]
fn parse_extract() {
    let sql = "SELECT EXTRACT(YEAR FROM d)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Extract {
            field: DateTimeField::Year,
            expr: Box::new(Expr::Identifier(Ident::new("d"))),
        },
        expr_from_projection(only(&select.projection)),
    );

    one_statement_parses_to("SELECT EXTRACT(year from d)", "SELECT EXTRACT(YEAR FROM d)");

    verified_stmt("SELECT EXTRACT(MONTH FROM d)");
    verified_stmt("SELECT EXTRACT(WEEK FROM d)");
    verified_stmt("SELECT EXTRACT(DAY FROM d)");
    verified_stmt("SELECT EXTRACT(DATE FROM d)");
    verified_stmt("SELECT EXTRACT(HOUR FROM d)");
    verified_stmt("SELECT EXTRACT(MINUTE FROM d)");
    verified_stmt("SELECT EXTRACT(SECOND FROM d)");
    verified_stmt("SELECT EXTRACT(MILLISECOND FROM d)");
    verified_stmt("SELECT EXTRACT(MICROSECOND FROM d)");
    verified_stmt("SELECT EXTRACT(NANOSECOND FROM d)");
    verified_stmt("SELECT EXTRACT(CENTURY FROM d)");
    verified_stmt("SELECT EXTRACT(DECADE FROM d)");
    verified_stmt("SELECT EXTRACT(DOW FROM d)");
    verified_stmt("SELECT EXTRACT(DOY FROM d)");
    verified_stmt("SELECT EXTRACT(EPOCH FROM d)");
    verified_stmt("SELECT EXTRACT(ISODOW FROM d)");
    verified_stmt("SELECT EXTRACT(ISOYEAR FROM d)");
    verified_stmt("SELECT EXTRACT(JULIAN FROM d)");
    verified_stmt("SELECT EXTRACT(MICROSECOND FROM d)");
    verified_stmt("SELECT EXTRACT(MICROSECONDS FROM d)");
    verified_stmt("SELECT EXTRACT(MILLENIUM FROM d)");
    verified_stmt("SELECT EXTRACT(MILLENNIUM FROM d)");
    verified_stmt("SELECT EXTRACT(MILLISECOND FROM d)");
    verified_stmt("SELECT EXTRACT(MILLISECONDS FROM d)");
    verified_stmt("SELECT EXTRACT(QUARTER FROM d)");
    verified_stmt("SELECT EXTRACT(TIMEZONE FROM d)");
    verified_stmt("SELECT EXTRACT(TIMEZONE_HOUR FROM d)");
    verified_stmt("SELECT EXTRACT(TIMEZONE_MINUTE FROM d)");

    let res = parse_sql_statements("SELECT EXTRACT(JIFFY FROM d)");
    assert_eq!(
        ParserError::ParserError("Expected date/time field, found: JIFFY".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_ceil_number() {
    verified_stmt("SELECT CEIL(1.5)");
    verified_stmt("SELECT CEIL(float_column) FROM my_table");
}

#[test]
fn parse_floor_number() {
    verified_stmt("SELECT FLOOR(1.5)");
    verified_stmt("SELECT FLOOR(float_column) FROM my_table");
}

#[test]
fn parse_ceil_datetime() {
    let sql = "SELECT CEIL(d TO DAY)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Ceil {
            expr: Box::new(Expr::Identifier(Ident::new("d"))),
            field: DateTimeField::Day,
        },
        expr_from_projection(only(&select.projection)),
    );

    one_statement_parses_to("SELECT CEIL(d to day)", "SELECT CEIL(d TO DAY)");

    verified_stmt("SELECT CEIL(d TO HOUR) FROM df");
    verified_stmt("SELECT CEIL(d TO MINUTE) FROM df");
    verified_stmt("SELECT CEIL(d TO SECOND) FROM df");
    verified_stmt("SELECT CEIL(d TO MILLISECOND) FROM df");

    let res = parse_sql_statements("SELECT CEIL(d TO JIFFY) FROM df");
    assert_eq!(
        ParserError::ParserError("Expected date/time field, found: JIFFY".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_floor_datetime() {
    let sql = "SELECT FLOOR(d TO DAY)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Floor {
            expr: Box::new(Expr::Identifier(Ident::new("d"))),
            field: DateTimeField::Day,
        },
        expr_from_projection(only(&select.projection)),
    );

    one_statement_parses_to("SELECT FLOOR(d to day)", "SELECT FLOOR(d TO DAY)");

    verified_stmt("SELECT FLOOR(d TO HOUR) FROM df");
    verified_stmt("SELECT FLOOR(d TO MINUTE) FROM df");
    verified_stmt("SELECT FLOOR(d TO SECOND) FROM df");
    verified_stmt("SELECT FLOOR(d TO MILLISECOND) FROM df");

    let res = parse_sql_statements("SELECT FLOOR(d TO JIFFY) FROM df");
    assert_eq!(
        ParserError::ParserError("Expected date/time field, found: JIFFY".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_listagg() {
    let sql = "SELECT LISTAGG(DISTINCT dateid, ', ' ON OVERFLOW TRUNCATE '%' WITHOUT COUNT) \
               WITHIN GROUP (ORDER BY id, username)";
    let select = verified_only_select(sql);

    verified_stmt("SELECT LISTAGG(sellerid) WITHIN GROUP (ORDER BY dateid)");
    verified_stmt("SELECT LISTAGG(dateid)");
    verified_stmt("SELECT LISTAGG(DISTINCT dateid)");
    verified_stmt("SELECT LISTAGG(dateid ON OVERFLOW ERROR)");
    verified_stmt("SELECT LISTAGG(dateid ON OVERFLOW TRUNCATE N'...' WITH COUNT)");
    verified_stmt("SELECT LISTAGG(dateid ON OVERFLOW TRUNCATE X'deadbeef' WITH COUNT)");

    let expr = Box::new(Expr::Identifier(Ident::new("dateid")));
    let on_overflow = Some(ListAggOnOverflow::Truncate {
        filler: Some(Box::new(Expr::Value(Value::SingleQuotedString(
            "%".to_string(),
        )))),
        with_count: false,
    });
    let within_group = vec![
        OrderByExpr {
            expr: Expr::Identifier(Ident {
                value: "id".to_string(),
                quote_style: None,
            }),
            asc: None,
            nulls_first: None,
        },
        OrderByExpr {
            expr: Expr::Identifier(Ident {
                value: "username".to_string(),
                quote_style: None,
            }),
            asc: None,
            nulls_first: None,
        },
    ];
    assert_eq!(
        &Expr::ListAgg(ListAgg {
            distinct: true,
            expr,
            separator: Some(Box::new(Expr::Value(Value::SingleQuotedString(
                ", ".to_string()
            )))),
            on_overflow,
            within_group,
        }),
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_array_agg_func() {
    let supported_dialects = TestedDialects {
        dialects: vec![
            Box::new(GenericDialect {}),
            Box::new(PostgreSqlDialect {}),
            Box::new(AnsiDialect {}),
        ],
    };

    for sql in [
        "SELECT ARRAY_AGG(x ORDER BY x) AS a FROM T",
        "SELECT ARRAY_AGG(x ORDER BY x LIMIT 2) FROM tbl",
        "SELECT ARRAY_AGG(DISTINCT x ORDER BY x LIMIT 2) FROM tbl",
    ] {
        supported_dialects.verified_stmt(sql);
    }
}

#[test]
fn parse_alter_errors() {
    let sql = "ALTER TABLE tab ADD COLUMN foo TEXT;";
    let dialect = GenericDialect {};
    let error_message = "ALTER is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn parse_drop_errors() {
    let sql = "DROP TABLE some_table_name;";
    let dialect = GenericDialect {};
    let error_message = "DROP is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn parse_scalar_function_in_projection() {
    let names = vec!["sqrt", "foo"];

    for function_name in names {
        // like SELECT sqrt(id) FROM foo
        let sql = dbg!(format!("SELECT {function_name}(id) FROM foo"));
        let select = verified_only_select(&sql);
        assert_eq!(
            &Expr::Function(Function {
                name: ObjectName(vec![Ident::new(function_name)]),
                args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(
                    Expr::Identifier(Ident::new("id"))
                ))],
                over: None,
                distinct: false,
                special: false,
            }),
            expr_from_projection(only(&select.projection))
        );
    }
}

#[test]
fn parse_named_argument_function() {
    let sql = "SELECT FUN(a => '1', b => '2') FROM foo";
    let select = verified_only_select(sql);

    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("FUN")]),
            args: vec![
                FunctionArg::Named {
                    name: Ident::new("a"),
                    arg: FunctionArgExpr::Expr(Expr::Value(Value::SingleQuotedString(
                        "1".to_owned()
                    ))),
                },
                FunctionArg::Named {
                    name: Ident::new("b"),
                    arg: FunctionArgExpr::Expr(Expr::Value(Value::SingleQuotedString(
                        "2".to_owned()
                    ))),
                },
            ],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_window_functions() {
    let sql = "SELECT row_number() OVER (ORDER BY dt DESC), \
               sum(foo) OVER (PARTITION BY a, b ORDER BY c, d \
               ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW), \
               avg(bar) OVER (ORDER BY a \
               RANGE BETWEEN 1 PRECEDING AND 1 FOLLOWING), \
               sum(bar) OVER (ORDER BY a \
               RANGE BETWEEN INTERVAL '1' DAY PRECEDING AND INTERVAL '1 MONTH' FOLLOWING), \
               COUNT(*) OVER (ORDER BY a \
               RANGE BETWEEN INTERVAL '1 DAY' PRECEDING AND INTERVAL '1 DAY' FOLLOWING), \
               max(baz) OVER (ORDER BY a \
               ROWS UNBOUNDED PRECEDING), \
               sum(qux) OVER (ORDER BY a \
               GROUPS BETWEEN 1 PRECEDING AND 1 FOLLOWING) \
               FROM foo";
    let select = verified_only_select(sql);
    assert_eq!(7, select.projection.len());
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("row_number")]),
            args: vec![],
            over: Some(WindowSpec {
                partition_by: vec![],
                order_by: vec![OrderByExpr {
                    expr: Expr::Identifier(Ident::new("dt")),
                    asc: Some(false),
                    nulls_first: None,
                }],
                window_frame: None,
            }),
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );
}

#[test]
fn parse_aggregate_with_group_by() {
    let sql = "SELECT a, COUNT(1), MIN(b), MAX(b) FROM foo GROUP BY a";
    let _ast = verified_only_select(sql);
    //TODO: assertions
}

#[test]
fn parse_literal_decimal() {
    // These numbers were explicitly chosen to not roundtrip if represented as
    // f64s (i.e., as 64-bit binary floating point numbers).
    let sql = "SELECT 0.300000000000000004, 9007199254740993.0";
    let select = verified_only_select(sql);
    assert_eq!(2, select.projection.len());
    assert_eq!(
        &Expr::Value(number("0.300000000000000004")),
        expr_from_projection(&select.projection[0]),
    );
    assert_eq!(
        &Expr::Value(number("9007199254740993.0")),
        expr_from_projection(&select.projection[1]),
    )
}

#[test]
fn parse_literal_string() {
    let sql = "SELECT 'one', N'national string', X'deadBEEF'";
    let select = verified_only_select(sql);
    assert_eq!(3, select.projection.len());
    assert_eq!(
        &Expr::Value(Value::SingleQuotedString("one".to_string())),
        expr_from_projection(&select.projection[0])
    );
    assert_eq!(
        &Expr::Value(Value::NationalStringLiteral("national string".to_string())),
        expr_from_projection(&select.projection[1])
    );
    assert_eq!(
        &Expr::Value(Value::HexStringLiteral("deadBEEF".to_string())),
        expr_from_projection(&select.projection[2])
    );

    one_statement_parses_to("SELECT x'deadBEEF'", "SELECT X'deadBEEF'");
    one_statement_parses_to("SELECT n'national string'", "SELECT N'national string'");
}

#[test]
fn parse_literal_date() {
    let sql = "SELECT DATE '1999-01-01'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::Date,
            value: "1999-01-01".into(),
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_literal_time() {
    let sql = "SELECT TIME '01:23:34'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::Time(None, TimezoneInfo::None),
            value: "01:23:34".into(),
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_literal_datetime() {
    let sql = "SELECT DATETIME '1999-01-01 01:23:34.45'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::Datetime(None),
            value: "1999-01-01 01:23:34.45".into(),
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_literal_timestamp_without_time_zone() {
    let sql = "SELECT TIMESTAMP '1999-01-01 01:23:34'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::Timestamp(None, TimezoneInfo::None),
            value: "1999-01-01 01:23:34".into(),
        },
        expr_from_projection(only(&select.projection)),
    );

    one_statement_parses_to("SELECT TIMESTAMP '1999-01-01 01:23:34'", sql);
}

#[test]
fn parse_literal_timestamp_with_time_zone() {
    let sql = "SELECT TIMESTAMPTZ '1999-01-01 01:23:34Z'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::Timestamp(None, TimezoneInfo::Tz),
            value: "1999-01-01 01:23:34Z".into(),
        },
        expr_from_projection(only(&select.projection)),
    );

    one_statement_parses_to("SELECT TIMESTAMPTZ '1999-01-01 01:23:34Z'", sql);
}

#[test]
fn parse_interval() {
    let sql = "SELECT INTERVAL '1-1' YEAR TO MONTH";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from("1-1")))),
            leading_field: Some(DateTimeField::Year),
            leading_precision: None,
            last_field: Some(DateTimeField::Month),
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL '01:01.01' MINUTE (5) TO SECOND (5)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from(
                "01:01.01"
            )))),
            leading_field: Some(DateTimeField::Minute),
            leading_precision: Some(5),
            last_field: Some(DateTimeField::Second),
            fractional_seconds_precision: Some(5),
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL '1' SECOND (5, 4)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from("1")))),
            leading_field: Some(DateTimeField::Second),
            leading_precision: Some(5),
            last_field: None,
            fractional_seconds_precision: Some(4),
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL '10' HOUR";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from("10")))),
            leading_field: Some(DateTimeField::Hour),
            leading_precision: None,
            last_field: None,
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL 5 DAY";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(number("5"))),
            leading_field: Some(DateTimeField::Day),
            leading_precision: None,
            last_field: None,
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL 1 + 1 DAY";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(number("1"))),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Value(number("1"))),
            }),
            leading_field: Some(DateTimeField::Day),
            leading_precision: None,
            last_field: None,
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL '10' HOUR (1)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from("10")))),
            leading_field: Some(DateTimeField::Hour),
            leading_precision: Some(1),
            last_field: None,
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT INTERVAL '1 DAY'";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Interval {
            value: Box::new(Expr::Value(Value::SingleQuotedString(String::from(
                "1 DAY"
            )))),
            leading_field: None,
            leading_precision: None,
            last_field: None,
            fractional_seconds_precision: None,
        },
        expr_from_projection(only(&select.projection)),
    );

    let result = parse_sql_statements("SELECT INTERVAL '1' SECOND TO SECOND");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: SECOND".to_string()),
        result.unwrap_err(),
    );

    let result = parse_sql_statements("SELECT INTERVAL '10' HOUR (1) TO HOUR (2)");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: (".to_string()),
        result.unwrap_err(),
    );

    verified_only_select("SELECT INTERVAL '1' YEAR");
    verified_only_select("SELECT INTERVAL '1' MONTH");
    verified_only_select("SELECT INTERVAL '1' DAY");
    verified_only_select("SELECT INTERVAL '1' HOUR");
    verified_only_select("SELECT INTERVAL '1' MINUTE");
    verified_only_select("SELECT INTERVAL '1' SECOND");
    verified_only_select("SELECT INTERVAL '1' YEAR TO MONTH");
    verified_only_select("SELECT INTERVAL '1' DAY TO HOUR");
    verified_only_select("SELECT INTERVAL '1' DAY TO MINUTE");
    verified_only_select("SELECT INTERVAL '1' DAY TO SECOND");
    verified_only_select("SELECT INTERVAL '1' HOUR TO MINUTE");
    verified_only_select("SELECT INTERVAL '1' HOUR TO SECOND");
    verified_only_select("SELECT INTERVAL '1' MINUTE TO SECOND");
    verified_only_select("SELECT INTERVAL '1 YEAR'");
    verified_only_select("SELECT INTERVAL '1 YEAR' AS one_year");
    one_statement_parses_to(
        "SELECT INTERVAL '1 YEAR' one_year",
        "SELECT INTERVAL '1 YEAR' AS one_year",
    );
}

#[test]
fn parse_interval_and_or_xor() {
    let sql = "SELECT col FROM test \
        WHERE d3_date > d1_date + INTERVAL '5 days' \
        AND d2_date > d1_date + INTERVAL '3 days'";

    let actual_ast = Parser::parse_sql(&GenericDialect {}, sql).unwrap();

    let expected_ast = vec![Statement::Query(Box::new(Query {
        config: None,
        with: None,
        body: Box::new(SetExpr::Select(Box::new(Select {
            distinct: false,
            top: None,
            projection: vec![UnnamedExpr(Expr::Identifier(Ident {
                value: "col".to_string(),
                quote_style: None,
            }))],
            into: None,
            from: vec![TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName(vec![Ident {
                        value: "test".to_string(),
                        quote_style: None,
                    }]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                },
                joins: vec![],
            }],
            lateral_views: vec![],
            selection: Some(Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier(Ident {
                        value: "d3_date".to_string(),
                        quote_style: None,
                    })),
                    op: BinaryOperator::Gt,
                    right: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Identifier(Ident {
                            value: "d1_date".to_string(),
                            quote_style: None,
                        })),
                        op: BinaryOperator::Plus,
                        right: Box::new(Expr::Interval {
                            value: Box::new(Expr::Value(Value::SingleQuotedString(
                                "5 days".to_string(),
                            ))),
                            leading_field: None,
                            leading_precision: None,
                            last_field: None,
                            fractional_seconds_precision: None,
                        }),
                    }),
                }),
                op: BinaryOperator::And,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier(Ident {
                        value: "d2_date".to_string(),
                        quote_style: None,
                    })),
                    op: BinaryOperator::Gt,
                    right: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Identifier(Ident {
                            value: "d1_date".to_string(),
                            quote_style: None,
                        })),
                        op: BinaryOperator::Plus,
                        right: Box::new(Expr::Interval {
                            value: Box::new(Expr::Value(Value::SingleQuotedString(
                                "3 days".to_string(),
                            ))),
                            leading_field: None,
                            leading_precision: None,
                            last_field: None,
                            fractional_seconds_precision: None,
                        }),
                    }),
                }),
            }),
            group_by: vec![],
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            qualify: None,
        }))),
        order_by: vec![],
        limit: None,
        offset: None,
        jinja_variables: vec![],
    }))];

    assert_eq!(actual_ast, expected_ast);

    verified_stmt(
        "SELECT col FROM test \
        WHERE d3_date > d1_date + INTERVAL '5 days' \
        AND d2_date > d1_date + INTERVAL '3 days'",
    );

    verified_stmt(
        "SELECT col FROM test \
        WHERE d3_date > d1_date + INTERVAL '5 days' \
        OR d2_date > d1_date + INTERVAL '3 days'",
    );

    verified_stmt(
        "SELECT col FROM test \
        WHERE d3_date > d1_date + INTERVAL '5 days' \
        XOR d2_date > d1_date + INTERVAL '3 days'",
    );
}

#[test]
fn parse_at_timezone() {
    let zero = Expr::Value(number("0"));
    let sql = "SELECT FROM_UNIXTIME(0) AT TIME ZONE 'UTC-06:00' FROM t";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::AtTimeZone {
            timestamp: Box::new(Expr::Function(Function {
                name: ObjectName(vec![Ident {
                    value: "FROM_UNIXTIME".to_string(),
                    quote_style: None,
                }]),
                args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(zero.clone()))],
                over: None,
                distinct: false,
                special: false,
            })),
            time_zone: "UTC-06:00".to_string(),
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = r#"SELECT DATE_FORMAT(FROM_UNIXTIME(0) AT TIME ZONE 'UTC-06:00', '%Y-%m-%dT%H') AS "hour" FROM t"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &SelectItem::ExprWithAlias {
            expr: Expr::Function(Function {
                name: ObjectName(vec![Ident {
                    value: "DATE_FORMAT".to_string(),
                    quote_style: None,
                },],),
                args: vec![
                    FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::AtTimeZone {
                        timestamp: Box::new(Expr::Function(Function {
                            name: ObjectName(vec![Ident {
                                value: "FROM_UNIXTIME".to_string(),
                                quote_style: None,
                            },],),
                            args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(zero))],
                            over: None,
                            distinct: false,
                            special: false,
                        },)),
                        time_zone: "UTC-06:00".to_string(),
                    },),),
                    FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(
                        Value::SingleQuotedString("%Y-%m-%dT%H".to_string()),
                    ),),),
                ],
                over: None,
                distinct: false,
                special: false,
            },),
            alias: Ident {
                value: "hour".to_string(),
                quote_style: Some('"'),
            },
        },
        only(&select.projection),
    );
}

#[test]
fn parse_json_keyword() {
    let sql = r#"SELECT JSON '{
  "id": 10,
  "type": "fruit",
  "name": "apple",
  "on_menu": true,
  "recipes":
    {
      "salads":
      [
        { "id": 2001, "type": "Walnut Apple Salad" },
        { "id": 2002, "type": "Apple Spinach Salad" }
      ],
      "desserts":
      [
        { "id": 3001, "type": "Apple Pie" },
        { "id": 3002, "type": "Apple Scones" },
        { "id": 3003, "type": "Apple Crumble" }
      ]
    }
}'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::JSON,
            value: r#"{
  "id": 10,
  "type": "fruit",
  "name": "apple",
  "on_menu": true,
  "recipes":
    {
      "salads":
      [
        { "id": 2001, "type": "Walnut Apple Salad" },
        { "id": 2002, "type": "Apple Spinach Salad" }
      ],
      "desserts":
      [
        { "id": 3001, "type": "Apple Pie" },
        { "id": 3002, "type": "Apple Scones" },
        { "id": 3003, "type": "Apple Crumble" }
      ]
    }
}"#
            .into()
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_bignumeric_keyword() {
    let sql = r#"SELECT BIGNUMERIC '0'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"0"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '0'");

    let sql = r#"SELECT BIGNUMERIC '123456'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"123456"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '123456'");

    let sql = r#"SELECT BIGNUMERIC '-3.14'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"-3.14"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '-3.14'");

    let sql = r#"SELECT BIGNUMERIC '-0.54321'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"-0.54321"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '-0.54321'");

    let sql = r#"SELECT BIGNUMERIC '1.23456e05'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"1.23456e05"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '1.23456e05'");

    let sql = r#"SELECT BIGNUMERIC '-9.876e-3'"#;
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::TypedString {
            data_type: DataType::BigNumeric(ExactNumberInfo::None),
            value: r#"-9.876e-3"#.into()
        },
        expr_from_projection(only(&select.projection)),
    );
    verified_stmt("SELECT BIGNUMERIC '-9.876e-3'");
}

#[test]
fn parse_simple_math_expr_plus() {
    let sql = "SELECT a + b, 2 + a, 2.5 + a, a_f + b_f, 2 + a_f, 2.5 + a_f FROM c";
    verified_only_select(sql);
}

#[test]
fn parse_simple_math_expr_minus() {
    let sql = "SELECT a - b, 2 - a, 2.5 - a, a_f - b_f, 2 - a_f, 2.5 - a_f FROM c";
    verified_only_select(sql);
}

#[test]
fn parse_table_function() {
    let select = verified_only_select("SELECT * FROM TABLE(FUN('1')) AS a");

    match only(select.from).relation {
        TableFactor::TableFunction { expr, alias } => {
            let expected_expr = Expr::Function(Function {
                name: ObjectName(vec![Ident::new("FUN")]),
                args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(
                    Value::SingleQuotedString("1".to_owned()),
                )))],
                over: None,
                distinct: false,
                special: false,
            });
            assert_eq!(expr, expected_expr);
            assert_eq!(alias, table_alias("a"))
        }
        _ => panic!("Expecting TableFactor::TableFunction"),
    }

    let res = parse_sql_statements("SELECT * FROM TABLE '1' AS a");
    assert_eq!(
        ParserError::ParserError("Expected (, found: \'1\'".to_string()),
        res.unwrap_err()
    );

    let res = parse_sql_statements("SELECT * FROM TABLE (FUN(a) AS a");
    assert_eq!(
        ParserError::ParserError("Expected ), found: AS".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_unnest() {
    fn chk(
        alias: bool,
        with_offset: bool,
        with_offset_alias: bool,
        dialects: &TestedDialects,
        want: Vec<TableWithJoins>,
    ) {
        let sql = &format!(
            "SELECT * FROM UNNEST(expr){}{}{}",
            if alias { " AS numbers" } else { "" },
            if with_offset { " WITH OFFSET" } else { "" },
            if with_offset_alias {
                " AS with_offset_alias"
            } else {
                ""
            },
        );
        let select = dialects.verified_only_select(sql);
        assert_eq!(select.from, want);
    }
    let dialects = TestedDialects {
        dialects: vec![Box::new(BigQueryDialect {}), Box::new(GenericDialect {})],
    };
    // 1. both Alias and WITH OFFSET clauses.
    chk(
        true,
        true,
        false,
        &dialects,
        vec![TableWithJoins {
            relation: TableFactor::UNNEST {
                alias: Some(TableAlias {
                    name: Ident::new("numbers"),
                    columns: vec![],
                }),
                array_expr: Box::new(Expr::Identifier(Ident::new("expr"))),
                with_offset: true,
                with_offset_alias: None,
            },
            joins: vec![],
        }],
    );
    // 2. neither Alias nor WITH OFFSET clause.
    chk(
        false,
        false,
        false,
        &dialects,
        vec![TableWithJoins {
            relation: TableFactor::UNNEST {
                alias: None,
                array_expr: Box::new(Expr::Identifier(Ident::new("expr"))),
                with_offset: false,
                with_offset_alias: None,
            },
            joins: vec![],
        }],
    );
    // 3. Alias but no WITH OFFSET clause.
    chk(
        false,
        true,
        false,
        &dialects,
        vec![TableWithJoins {
            relation: TableFactor::UNNEST {
                alias: None,
                array_expr: Box::new(Expr::Identifier(Ident::new("expr"))),
                with_offset: true,
                with_offset_alias: None,
            },
            joins: vec![],
        }],
    );
    // 4. WITH OFFSET but no Alias.
    chk(
        true,
        false,
        false,
        &dialects,
        vec![TableWithJoins {
            relation: TableFactor::UNNEST {
                alias: Some(TableAlias {
                    name: Ident::new("numbers"),
                    columns: vec![],
                }),
                array_expr: Box::new(Expr::Identifier(Ident::new("expr"))),
                with_offset: false,
                with_offset_alias: None,
            },
            joins: vec![],
        }],
    );
}

#[test]
fn parse_parens() {
    use self::BinaryOperator::*;
    use self::Expr::*;
    let sql = "(a + b) - (c + d)";
    assert_eq!(
        BinaryOp {
            left: Box::new(Nested(Box::new(BinaryOp {
                left: Box::new(Identifier(Ident::new("a"))),
                op: Plus,
                right: Box::new(Identifier(Ident::new("b"))),
            }))),
            op: Minus,
            right: Box::new(Nested(Box::new(BinaryOp {
                left: Box::new(Identifier(Ident::new("c"))),
                op: Plus,
                right: Box::new(Identifier(Ident::new("d"))),
            }))),
        },
        verified_expr(sql)
    );
}

#[test]
fn parse_searched_case_expr() {
    let sql = "SELECT CASE WHEN bar IS NULL THEN 'null' WHEN bar = 0 THEN '=0' WHEN bar >= 0 THEN '>=0' ELSE '<0' END FROM foo";
    use self::BinaryOperator::*;
    use self::Expr::{BinaryOp, Case, Identifier, IsNull};
    let select = verified_only_select(sql);
    assert_eq!(
        &Case {
            operand: None,
            conditions: vec![
                IsNull(Box::new(Identifier(Ident::new("bar")))),
                BinaryOp {
                    left: Box::new(Identifier(Ident::new("bar"))),
                    op: Eq,
                    right: Box::new(Expr::Value(number("0"))),
                },
                BinaryOp {
                    left: Box::new(Identifier(Ident::new("bar"))),
                    op: GtEq,
                    right: Box::new(Expr::Value(number("0"))),
                },
            ],
            results: vec![
                Expr::Value(Value::SingleQuotedString("null".to_string())),
                Expr::Value(Value::SingleQuotedString("=0".to_string())),
                Expr::Value(Value::SingleQuotedString(">=0".to_string())),
            ],
            else_result: Some(Box::new(Expr::Value(Value::SingleQuotedString(
                "<0".to_string()
            )))),
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_simple_case_expr() {
    // ANSI calls a CASE expression with an operand "<simple case>"
    let sql = "SELECT CASE foo WHEN 1 THEN 'Y' ELSE 'N' END";
    let select = verified_only_select(sql);
    use self::Expr::{Case, Identifier};
    assert_eq!(
        &Case {
            operand: Some(Box::new(Identifier(Ident::new("foo")))),
            conditions: vec![Expr::Value(number("1"))],
            results: vec![Expr::Value(Value::SingleQuotedString("Y".to_string()))],
            else_result: Some(Box::new(Expr::Value(Value::SingleQuotedString(
                "N".to_string()
            )))),
        },
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_from_advanced() {
    let sql = "SELECT * FROM fn(1, 2) AS foo, schema.bar AS bar WITH (NOLOCK)";
    let _select = verified_only_select(sql);
}

#[test]
fn parse_nullary_table_valued_function() {
    let sql = "SELECT * FROM fn()";
    let _select = verified_only_select(sql);
}

#[test]
fn parse_implicit_join() {
    let sql = "SELECT * FROM t1, t2";
    let select = verified_only_select(sql);
    assert_eq!(
        vec![
            TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName(vec!["t1".into()]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                },
                joins: vec![],
            },
            TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName(vec!["t2".into()]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                },
                joins: vec![],
            },
        ],
        select.from,
    );

    let sql = "SELECT * FROM t1a NATURAL JOIN t1b, t2a NATURAL JOIN t2b";
    let select = verified_only_select(sql);
    assert_eq!(
        vec![
            TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName(vec!["t1a".into()]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                },
                joins: vec![Join {
                    relation: TableFactor::Table {
                        name: ObjectName(vec!["t1b".into()]),
                        alias: None,
                        args: None,
                        with_hints: vec![],
                    },
                    join_operator: JoinOperator::Inner(JoinConstraint::Natural),
                }],
            },
            TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName(vec!["t2a".into()]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                },
                joins: vec![Join {
                    relation: TableFactor::Table {
                        name: ObjectName(vec!["t2b".into()]),
                        alias: None,
                        args: None,
                        with_hints: vec![],
                    },
                    join_operator: JoinOperator::Inner(JoinConstraint::Natural),
                }],
            },
        ],
        select.from,
    );
}

#[test]
fn parse_cross_join() {
    let sql = "SELECT * FROM t1 CROSS JOIN t2";
    let select = verified_only_select(sql);
    assert_eq!(
        Join {
            relation: TableFactor::Table {
                name: ObjectName(vec![Ident::new("t2")]),
                alias: None,
                args: None,
                with_hints: vec![],
            },
            join_operator: JoinOperator::CrossJoin,
        },
        only(only(select.from).joins),
    );
}

#[test]
fn parse_joins_on() {
    fn join_with_constraint(
        relation: impl Into<String>,
        alias: Option<TableAlias>,
        f: impl Fn(JoinConstraint) -> JoinOperator,
    ) -> Join {
        Join {
            relation: TableFactor::Table {
                name: ObjectName(vec![Ident::new(relation.into())]),
                alias,
                args: None,
                with_hints: vec![],
            },
            join_operator: f(JoinConstraint::On(Expr::BinaryOp {
                left: Box::new(Expr::Identifier("c1".into())),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::Identifier("c2".into())),
            })),
        }
    }
    // Test parsing of aliases
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 JOIN t2 AS foo ON c1 = c2").from).joins,
        vec![join_with_constraint(
            "t2",
            table_alias("foo"),
            JoinOperator::Inner,
        )]
    );
    one_statement_parses_to(
        "SELECT * FROM t1 JOIN t2 foo ON c1 = c2",
        "SELECT * FROM t1 JOIN t2 AS foo ON c1 = c2",
    );
    // Test parsing of different join operators
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::Inner)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftOuter)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightOuter)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT SEMI JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftSemi)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT SEMI JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightSemi)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT ANTI JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftAnti)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT ANTI JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightAnti)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 FULL JOIN t2 ON c1 = c2").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::FullOuter)]
    );
}

#[test]
fn parse_joins_using() {
    fn join_with_constraint(
        relation: impl Into<String>,
        alias: Option<TableAlias>,
        f: impl Fn(JoinConstraint) -> JoinOperator,
    ) -> Join {
        Join {
            relation: TableFactor::Table {
                name: ObjectName(vec![Ident::new(relation.into())]),
                alias,
                args: None,
                with_hints: vec![],
            },
            join_operator: f(JoinConstraint::Using(vec!["c1".into()])),
        }
    }
    // Test parsing of aliases
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 JOIN t2 AS foo USING(c1)").from).joins,
        vec![join_with_constraint(
            "t2",
            table_alias("foo"),
            JoinOperator::Inner,
        )]
    );
    one_statement_parses_to(
        "SELECT * FROM t1 JOIN t2 foo USING(c1)",
        "SELECT * FROM t1 JOIN t2 AS foo USING(c1)",
    );
    // Test parsing of different join operators
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::Inner)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftOuter)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightOuter)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT SEMI JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftSemi)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT SEMI JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightSemi)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 LEFT ANTI JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::LeftAnti)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 RIGHT ANTI JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::RightAnti)]
    );
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 FULL JOIN t2 USING(c1)").from).joins,
        vec![join_with_constraint("t2", None, JoinOperator::FullOuter)]
    );
}

#[test]
fn parse_natural_join() {
    fn natural_join(f: impl Fn(JoinConstraint) -> JoinOperator, alias: Option<TableAlias>) -> Join {
        Join {
            relation: TableFactor::Table {
                name: ObjectName(vec![Ident::new("t2")]),
                alias,
                args: None,
                with_hints: vec![],
            },
            join_operator: f(JoinConstraint::Natural),
        }
    }

    // if not specified, inner join as default
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 NATURAL JOIN t2").from).joins,
        vec![natural_join(JoinOperator::Inner, None)]
    );
    // left join explicitly
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 NATURAL LEFT JOIN t2").from).joins,
        vec![natural_join(JoinOperator::LeftOuter, None)]
    );

    // right join explicitly
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 NATURAL RIGHT JOIN t2").from).joins,
        vec![natural_join(JoinOperator::RightOuter, None)]
    );

    // full join explicitly
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 NATURAL FULL JOIN t2").from).joins,
        vec![natural_join(JoinOperator::FullOuter, None)]
    );

    // natural join another table with alias
    assert_eq!(
        only(&verified_only_select("SELECT * FROM t1 NATURAL JOIN t2 AS t3").from).joins,
        vec![natural_join(JoinOperator::Inner, table_alias("t3"))]
    );

    let sql = "SELECT * FROM t1 natural";
    assert_eq!(
        ParserError::ParserError("Expected a join type after NATURAL, found: EOF".to_string()),
        parse_sql_statements(sql).unwrap_err(),
    );
}

#[test]
fn parse_complex_join() {
    let sql = "SELECT c1, c2 FROM t1, t4 JOIN t2 ON t2.c = t1.c LEFT JOIN t3 USING(q, c) WHERE t4.c = t1.c";
    verified_only_select(sql);
}

#[test]
fn parse_join_nesting() {
    let sql = "SELECT * FROM a NATURAL JOIN (b NATURAL JOIN (c NATURAL JOIN d NATURAL JOIN e)) \
               NATURAL JOIN (f NATURAL JOIN (g NATURAL JOIN h))";
    assert_eq!(
        only(&verified_only_select(sql).from).joins,
        vec![
            join(nest!(table("b"), nest!(table("c"), table("d"), table("e")))),
            join(nest!(table("f"), nest!(table("g"), table("h")))),
        ],
    );

    let sql = "SELECT * FROM (a NATURAL JOIN b) NATURAL JOIN c";
    let select = verified_only_select(sql);
    let from = only(select.from);
    assert_eq!(from.relation, nest!(table("a"), table("b")));
    assert_eq!(from.joins, vec![join(table("c"))]);

    let sql = "SELECT * FROM (((a NATURAL JOIN b)))";
    let select = verified_only_select(sql);
    let from = only(select.from);
    assert_eq!(from.relation, nest!(nest!(nest!(table("a"), table("b")))));
    assert_eq!(from.joins, vec![]);

    let sql = "SELECT * FROM a NATURAL JOIN (((b NATURAL JOIN c)))";
    let select = verified_only_select(sql);
    let from = only(select.from);
    assert_eq!(from.relation, table("a"));
    assert_eq!(
        from.joins,
        vec![join(nest!(nest!(nest!(table("b"), table("c")))))]
    );

    let sql = "SELECT * FROM (a NATURAL JOIN b) AS c";
    let select = verified_only_select(sql);
    let from = only(select.from);
    assert_eq!(
        from.relation,
        TableFactor::NestedJoin {
            table_with_joins: Box::new(TableWithJoins {
                relation: table("a"),
                joins: vec![join(table("b"))],
            }),
            alias: table_alias("c"),
        }
    );
    assert_eq!(from.joins, vec![]);
}

#[test]
fn parse_join_syntax_variants() {
    one_statement_parses_to(
        "SELECT c1 FROM t1 INNER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 JOIN t2 USING(c1)",
    );
    one_statement_parses_to(
        "SELECT c1 FROM t1 LEFT OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 LEFT JOIN t2 USING(c1)",
    );
    one_statement_parses_to(
        "SELECT c1 FROM t1 RIGHT OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 RIGHT JOIN t2 USING(c1)",
    );
    one_statement_parses_to(
        "SELECT c1 FROM t1 FULL OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 FULL JOIN t2 USING(c1)",
    );

    let res = parse_sql_statements("SELECT * FROM a OUTER JOIN b ON 1");
    assert_eq!(
        ParserError::ParserError("Expected APPLY, found: JOIN".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_ctes() {
    let cte_sqls = vec!["SELECT 1 AS foo", "SELECT 2 AS bar"];
    let with = &format!(
        "WITH a AS ({}), b AS ({}) SELECT foo + bar FROM a, b",
        cte_sqls[0], cte_sqls[1]
    );

    fn assert_ctes_in_select(expected: &[&str], sel: &Query) {
        for (i, exp) in expected.iter().enumerate() {
            let Cte { alias, query, .. } = &sel.with.as_ref().unwrap().cte_tables[i];
            assert_eq!(*exp, query.to_string());
            assert_eq!(
                if i == 0 {
                    Ident::new("a")
                } else {
                    Ident::new("b")
                },
                alias.name
            );
            assert!(alias.columns.is_empty());
        }
    }

    // Top-level CTE
    assert_ctes_in_select(&cte_sqls, &verified_query(with));
    // CTE in a subquery
    let sql = &format!("SELECT ({with})");
    let select = verified_only_select(sql);
    match expr_from_projection(only(&select.projection)) {
        Expr::Subquery(ref subquery) => {
            assert_ctes_in_select(&cte_sqls, subquery.as_ref());
        }
        _ => panic!("Expected subquery"),
    }
    // CTE in a derived table
    let sql = &format!("SELECT * FROM ({with})");
    let select = verified_only_select(sql);
    match only(select.from).relation {
        TableFactor::Derived { subquery, .. } => {
            assert_ctes_in_select(&cte_sqls, subquery.as_ref())
        }
        _ => panic!("Expected derived table"),
    }
    // CTE in a CTE...
    let sql = &format!("WITH outer_cte AS ({with}) SELECT * FROM outer_cte");
    let select = verified_query(sql);
    assert_ctes_in_select(&cte_sqls, &only(&select.with.unwrap().cte_tables).query);
}

#[test]
fn parse_cte_renamed_columns() {
    let sql = "WITH cte (col1, col2) AS (SELECT foo, bar FROM baz) SELECT * FROM cte";
    let query = all_dialects().verified_query(sql);
    assert_eq!(
        vec![Ident::new("col1"), Ident::new("col2")],
        query
            .with
            .unwrap()
            .cte_tables
            .first()
            .unwrap()
            .alias
            .columns
    );
}

#[test]
fn parse_recursive_cte() {
    let cte_query = "SELECT 1 UNION ALL SELECT val + 1 FROM nums WHERE val < 10".to_owned();
    let sql = &format!("WITH RECURSIVE nums (val) AS ({cte_query}) SELECT * FROM nums");

    let cte_query = verified_query(&cte_query);
    let query = verified_query(sql);

    let with = query.with.as_ref().unwrap();
    assert!(with.recursive);
    assert_eq!(with.cte_tables.len(), 1);
    let expected = Cte {
        alias: TableAlias {
            name: Ident {
                value: "nums".to_string(),
                quote_style: None,
            },
            columns: vec![Ident {
                value: "val".to_string(),
                quote_style: None,
            }],
        },
        query: Box::new(cte_query),
        from: None,
    };
    assert_eq!(with.cte_tables.first().unwrap(), &expected);
}

#[test]
fn parse_derived_tables() {
    let sql = "SELECT a.x, b.y FROM (SELECT x FROM foo) AS a CROSS JOIN (SELECT y FROM bar) AS b";
    let _ = verified_only_select(sql);
    //TODO: add assertions

    let sql = "SELECT a.x, b.y \
               FROM (SELECT x FROM foo) AS a (x) \
               CROSS JOIN (SELECT y FROM bar) AS b (y)";
    let _ = verified_only_select(sql);
    //TODO: add assertions

    let sql = "SELECT * FROM (((SELECT 1)))";
    let _ = verified_only_select(sql);
    // TODO: add assertions

    let sql = "SELECT * FROM t NATURAL JOIN (((SELECT 1)))";
    let _ = verified_only_select(sql);
    // TODO: add assertions

    let sql = "SELECT * FROM (((SELECT 1) UNION (SELECT 2)) AS t1 NATURAL JOIN t2)";
    let select = verified_only_select(sql);
    let from = only(select.from);
    assert_eq!(
        from.relation,
        TableFactor::NestedJoin {
            table_with_joins: Box::new(TableWithJoins {
                relation: TableFactor::Derived {
                    lateral: false,
                    subquery: Box::new(verified_query("(SELECT 1) UNION (SELECT 2)")),
                    alias: Some(TableAlias {
                        name: "t1".into(),
                        columns: vec![],
                    }),
                },
                joins: vec![Join {
                    relation: TableFactor::Table {
                        name: ObjectName(vec!["t2".into()]),
                        alias: None,
                        args: None,
                        with_hints: vec![],
                    },
                    join_operator: JoinOperator::Inner(JoinConstraint::Natural),
                }],
            }),
            alias: None,
        }
    );
}

#[test]
fn parse_union_except_intersect() {
    // TODO: add assertions
    verified_stmt("SELECT 1 UNION SELECT 2");
    verified_stmt("SELECT 1 UNION ALL SELECT 2");
    verified_stmt("SELECT 1 UNION DISTINCT SELECT 1");
    verified_stmt("SELECT 1 EXCEPT SELECT 2");
    verified_stmt("SELECT 1 EXCEPT ALL SELECT 2");
    verified_stmt("SELECT 1 EXCEPT DISTINCT SELECT 1");
    verified_stmt("SELECT 1 INTERSECT SELECT 2");
    verified_stmt("SELECT 1 INTERSECT ALL SELECT 2");
    verified_stmt("SELECT 1 INTERSECT DISTINCT SELECT 1");
    verified_stmt("SELECT 1 UNION SELECT 2 UNION SELECT 3");
    verified_stmt("SELECT 1 EXCEPT SELECT 2 UNION SELECT 3"); // Union[Except[1,2], 3]
    verified_stmt("SELECT 1 INTERSECT (SELECT 2 EXCEPT SELECT 3)");
    verified_stmt("WITH cte AS (SELECT 1 AS foo) (SELECT foo FROM cte ORDER BY 1 LIMIT 1)");
    verified_stmt("SELECT 1 UNION (SELECT 2 ORDER BY 1 LIMIT 1)");
    verified_stmt("SELECT 1 UNION SELECT 2 INTERSECT SELECT 3"); // Union[1, Intersect[2,3]]
    verified_stmt("SELECT foo FROM tab UNION SELECT bar FROM TAB");
    verified_stmt("(SELECT * FROM new EXCEPT SELECT * FROM old) UNION ALL (SELECT * FROM old EXCEPT SELECT * FROM new) ORDER BY 1");
    verified_stmt("(SELECT * FROM new EXCEPT DISTINCT SELECT * FROM old) UNION DISTINCT (SELECT * FROM old EXCEPT DISTINCT SELECT * FROM new) ORDER BY 1");
}

#[test]
fn parse_values() {
    verified_stmt("SELECT * FROM (VALUES (1), (2), (3))");
    verified_stmt("SELECT * FROM (VALUES (1), (2), (3)), (VALUES (1, 2, 3))");
    verified_stmt("SELECT * FROM (VALUES (1)) UNION VALUES (1)");
    verified_stmt("SELECT * FROM (VALUES ROW(1, true, 'a'), ROW(2, false, 'b')) AS t (a, b, c)");
}

#[test]
fn parse_multiple_statements() {
    fn test_with(sql1: &str, sql2_kw: &str, sql2_rest: &str) {
        // Check that a string consisting of two statements delimited by a semicolon
        // parses the same as both statements individually:
        let res = parse_sql_statements(&(sql1.to_owned() + ";" + sql2_kw + sql2_rest));
        assert_eq!(
            vec![
                one_statement_parses_to(sql1, ""),
                one_statement_parses_to(&(sql2_kw.to_owned() + sql2_rest), ""),
            ],
            res.unwrap()
        );
        // Check that extra semicolon at the end is stripped by normalization:
        one_statement_parses_to(&(sql1.to_owned() + ";"), sql1);
        // Check that forgetting the semicolon results in an error:
        let res = parse_sql_statements(&(sql1.to_owned() + " " + sql2_kw + sql2_rest));
        assert_eq!(
            ParserError::ParserError("Expected end of statement, found: ".to_string() + sql2_kw),
            res.unwrap_err()
        );
    }
    test_with("SELECT foo", "SELECT", " bar");
    // ensure that SELECT/WITH is not parsed as a table or column alias if ';'
    // separating the statements is omitted:
    test_with("SELECT foo FROM baz", "SELECT", " bar");
    test_with("SELECT foo", "WITH", " cte AS (SELECT 1 AS s) SELECT bar");
    test_with(
        "SELECT foo FROM baz",
        "WITH",
        " cte AS (SELECT 1 AS s) SELECT bar",
    );
    // Make sure that empty statements do not cause an error:
    let res = parse_sql_statements(";;");
    assert_eq!(0, res.unwrap().len());
}

#[test]
fn parse_scalar_subqueries() {
    let sql = "(SELECT 1) + (SELECT 2)";
    assert_matches!(
        verified_expr(sql),
        Expr::BinaryOp {
            op: BinaryOperator::Plus,
            ..
        }
    );
}

#[test]
fn parse_substring() {
    one_statement_parses_to("SELECT SUBSTRING('1')", "SELECT SUBSTRING('1')");

    one_statement_parses_to(
        "SELECT SUBSTRING('1' FROM 1)",
        "SELECT SUBSTRING('1' FROM 1)",
    );

    one_statement_parses_to(
        "SELECT SUBSTRING('1' FROM 1 FOR 3)",
        "SELECT SUBSTRING('1' FROM 1 FOR 3)",
    );

    one_statement_parses_to("SELECT SUBSTRING('1' FOR 3)", "SELECT SUBSTRING('1' FOR 3)");
}

#[test]
fn parse_overlay() {
    one_statement_parses_to(
        "SELECT OVERLAY('abccccde' PLACING 'abc' FROM 3)",
        "SELECT OVERLAY('abccccde' PLACING 'abc' FROM 3)",
    );
    one_statement_parses_to(
        "SELECT OVERLAY('abccccde' PLACING 'abc' FROM 3 FOR 12)",
        "SELECT OVERLAY('abccccde' PLACING 'abc' FROM 3 FOR 12)",
    );
    assert_eq!(
        ParserError::ParserError("Expected PLACING, found: FROM".to_owned()),
        parse_sql_statements("SELECT OVERLAY('abccccde' FROM 3)").unwrap_err(),
    );

    let sql = "SELECT OVERLAY('abcdef' PLACING name FROM 3 FOR id + 1) FROM CUSTOMERS";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Overlay {
            expr: Box::new(Expr::Value(Value::SingleQuotedString("abcdef".to_string()))),
            overlay_what: Box::new(Expr::Identifier(Ident::new("name"))),
            overlay_from: Box::new(Expr::Value(number("3"))),
            overlay_for: Some(Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Identifier(Ident::new("id"))),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Value(number("1"))),
            })),
        },
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_trim() {
    one_statement_parses_to(
        "SELECT TRIM(BOTH 'xyz' FROM 'xyzfooxyz')",
        "SELECT TRIM(BOTH 'xyz' FROM 'xyzfooxyz')",
    );

    one_statement_parses_to(
        "SELECT TRIM(LEADING 'xyz' FROM 'xyzfooxyz')",
        "SELECT TRIM(LEADING 'xyz' FROM 'xyzfooxyz')",
    );

    one_statement_parses_to(
        "SELECT TRIM(TRAILING 'xyz' FROM 'xyzfooxyz')",
        "SELECT TRIM(TRAILING 'xyz' FROM 'xyzfooxyz')",
    );

    one_statement_parses_to(
        "SELECT TRIM('xyz' FROM 'xyzfooxyz')",
        "SELECT TRIM('xyz' FROM 'xyzfooxyz')",
    );
    one_statement_parses_to("SELECT TRIM('   foo   ')", "SELECT TRIM('   foo   ')");
    one_statement_parses_to(
        "SELECT TRIM(LEADING '   foo   ')",
        "SELECT TRIM(LEADING '   foo   ')",
    );

    assert_eq!(
        ParserError::ParserError("Expected ), found: 'xyz'".to_owned()),
        parse_sql_statements("SELECT TRIM(FOO 'xyz' FROM 'xyzfooxyz')").unwrap_err()
    );
}

#[test]
fn parse_exists_subquery() {
    let expected_inner = verified_query("SELECT 1");
    let sql = "SELECT * FROM t WHERE EXISTS (SELECT 1)";
    let select = verified_only_select(sql);
    assert_eq!(
        Expr::Exists {
            negated: false,
            subquery: Box::new(expected_inner.clone()),
        },
        select.selection.unwrap(),
    );

    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1)";
    let select = verified_only_select(sql);
    assert_eq!(
        Expr::Exists {
            negated: true,
            subquery: Box::new(expected_inner),
        },
        select.selection.unwrap(),
    );

    verified_stmt("SELECT * FROM t WHERE EXISTS (WITH u AS (SELECT 1) SELECT * FROM u)");
    verified_stmt("SELECT EXISTS (SELECT 1)");

    let res = parse_sql_statements("SELECT EXISTS (");
    assert_eq!(
        ParserError::ParserError(
            "Expected SELECT, VALUES, or a subquery in the query body, found: EOF".to_string()
        ),
        res.unwrap_err(),
    );

    let res = parse_sql_statements("SELECT EXISTS (NULL)");
    assert_eq!(
        ParserError::ParserError(
            "Expected SELECT, VALUES, or a subquery in the query body, found: NULL".to_string()
        ),
        res.unwrap_err(),
    );
}


#[test]
fn parse_invalid_subquery_without_parens() {
    let res = parse_sql_statements("SELECT SELECT 1 FROM bar WHERE 1=1 FROM baz");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: 1".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_offset() {
    let expect = Some(Offset {
        value: Expr::Value(number("2")),
        rows: OffsetRows::Rows,
    });
    let ast = verified_query("SELECT foo FROM bar OFFSET 2 ROWS");
    assert_eq!(ast.offset, expect);
    let ast = verified_query("SELECT foo FROM bar WHERE foo = 4 OFFSET 2 ROWS");
    assert_eq!(ast.offset, expect);
    let ast = verified_query("SELECT foo FROM bar ORDER BY baz OFFSET 2 ROWS");
    assert_eq!(ast.offset, expect);
    let ast = verified_query("SELECT foo FROM bar WHERE foo = 4 ORDER BY baz OFFSET 2 ROWS");
    assert_eq!(ast.offset, expect);
    let ast = verified_query("SELECT foo FROM (SELECT * FROM bar OFFSET 2 ROWS) OFFSET 2 ROWS");
    assert_eq!(ast.offset, expect);
    match *ast.body {
        SetExpr::Select(s) => match only(s.from).relation {
            TableFactor::Derived { subquery, .. } => {
                assert_eq!(subquery.offset, expect);
            }
            _ => panic!("Test broke"),
        },
        _ => panic!("Test broke"),
    }
    let ast = verified_query("SELECT 'foo' OFFSET 0 ROWS");
    assert_eq!(
        ast.offset,
        Some(Offset {
            value: Expr::Value(number("0")),
            rows: OffsetRows::Rows,
        })
    );
    let ast = verified_query("SELECT 'foo' OFFSET 1 ROW");
    assert_eq!(
        ast.offset,
        Some(Offset {
            value: Expr::Value(number("1")),
            rows: OffsetRows::Row,
        })
    );
    let ast = verified_query("SELECT 'foo' OFFSET 1");
    assert_eq!(
        ast.offset,
        Some(Offset {
            value: Expr::Value(number("1")),
            rows: OffsetRows::None,
        })
    );
}

#[test]
fn lateral_derived() {
    fn chk(lateral_in: bool) {
        let lateral_str = if lateral_in { "LATERAL " } else { "" };
        let sql = format!(
            "SELECT * FROM customer LEFT JOIN {lateral_str}\
             (SELECT * FROM order WHERE order.customer = customer.id LIMIT 3) AS order ON true"
        );
        let select = verified_only_select(&sql);
        let from = only(select.from);
        assert_eq!(from.joins.len(), 1);
        let join = &from.joins[0];
        assert_eq!(
            join.join_operator,
            JoinOperator::LeftOuter(JoinConstraint::On(Expr::Value(Value::Boolean(true))))
        );
        if let TableFactor::Derived {
            lateral,
            ref subquery,
            alias: Some(ref alias),
        } = join.relation
        {
            assert_eq!(lateral_in, lateral);
            assert_eq!(Ident::new("order"), alias.name);
            assert_eq!(
                subquery.to_string(),
                "SELECT * FROM order WHERE order.customer = customer.id LIMIT 3"
            );
        } else {
            unreachable!()
        }
    }
    chk(false);
    chk(true);

    let sql = "SELECT * FROM customer LEFT JOIN LATERAL generate_series(1, customer.id)";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError(
            "Expected subquery after LATERAL, found: generate_series".to_string()
        ),
        res.unwrap_err()
    );

    let sql = "SELECT * FROM a LEFT JOIN LATERAL (b CROSS JOIN c)";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError(
            "Expected SELECT, VALUES, or a subquery in the query body, found: b".to_string()
        ),
        res.unwrap_err()
    );
}

#[test]
#[should_panic(expected = "Parse results with GenericDialect are different from PostgreSqlDialect")]
fn ensure_multiple_dialects_are_tested() {
    // The SQL here must be parsed differently by different dialects.
    // At the time of writing, `@foo` is accepted as a valid identifier
    // by the Generic and the MSSQL dialect, but not by Postgres and ANSI.
    let _ = parse_sql_statements("SELECT @foo");
}

#[test]
fn test_placeholder() {
    let sql = "SELECT * FROM student WHERE id = ?";
    let ast = verified_only_select(sql);
    assert_eq!(
        ast.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("id"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Value(Value::Placeholder("?".into()))),
        })
    );

    let dialects = TestedDialects {
        dialects: vec![
            Box::new(GenericDialect {}),
            Box::new(PostgreSqlDialect {}),
            Box::new(AnsiDialect {}),
            Box::new(BigQueryDialect {}),
            Box::new(SnowflakeDialect {}),
        ],
    };
    let sql = "SELECT * FROM student WHERE id = $Id1";
    let ast = dialects.verified_only_select(sql);
    assert_eq!(
        ast.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("id"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Value(Value::Placeholder("$Id1".into()))),
        })
    );

    let sql = "SELECT * FROM student LIMIT $1 OFFSET $2";
    let ast = dialects.verified_query(sql);
    assert_eq!(
        ast.limit,
        Some(Expr::Value(Value::Placeholder("$1".into())))
    );
    assert_eq!(
        ast.offset,
        Some(Offset {
            value: Expr::Value(Value::Placeholder("$2".into())),
            rows: OffsetRows::None,
        }),
    );

    let sql = "SELECT $fromage_français, :x, ?123";
    let ast = dialects.verified_only_select(sql);
    assert_eq!(
        ast.projection,
        vec![
            UnnamedExpr(Expr::Value(Value::Placeholder("$fromage_français".into()))),
            UnnamedExpr(Expr::Value(Value::Placeholder(":x".into()))),
            UnnamedExpr(Expr::Value(Value::Placeholder("?123".into()))),
        ]
    );
}

#[test]
fn all_keywords_sorted() {
    // assert!(ALL_KEYWORDS.is_sorted())
    let mut copy = Vec::from(ALL_KEYWORDS);
    copy.sort_unstable();
    assert_eq!(copy, ALL_KEYWORDS)
}

fn parse_sql_statements(sql: &str) -> Result<Vec<Statement>, ParserError> {
    all_dialects().parse_sql_statements(sql)
}

fn one_statement_parses_to(sql: &str, canonical: &str) -> Statement {
    all_dialects().one_statement_parses_to(sql, canonical)
}

fn verified_stmt(query: &str) -> Statement {
    all_dialects().verified_stmt(query)
}

fn verified_query(query: &str) -> Query {
    all_dialects().verified_query(query)
}

fn verified_only_select(query: &str) -> Select {
    all_dialects().verified_only_select(query)
}

fn verified_expr(query: &str) -> Expr {
    all_dialects().verified_expr(query)
}

#[test]
fn parse_offset_and_limit() {
    let sql = "SELECT foo FROM bar LIMIT 2 OFFSET 2";
    let expect = Some(Offset {
        value: Expr::Value(number("2")),
        rows: OffsetRows::None,
    });
    let ast = verified_query(sql);
    assert_eq!(ast.offset, expect);
    assert_eq!(ast.limit, Some(Expr::Value(number("2"))));

    // different order is OK
    one_statement_parses_to("SELECT foo FROM bar OFFSET 2 LIMIT 2", sql);

    // expressions are allowed
    let sql = "SELECT foo FROM bar LIMIT 1 + 2 OFFSET 3 * 4";
    let ast = verified_query(sql);
    assert_eq!(
        ast.limit,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Value(number("1"))),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Value(number("2"))),
        }),
    );
    assert_eq!(
        ast.offset,
        Some(Offset {
            value: Expr::BinaryOp {
                left: Box::new(Expr::Value(number("3"))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expr::Value(number("4"))),
            },
            rows: OffsetRows::None,
        }),
    );

    // Can't repeat OFFSET / LIMIT
    let res = parse_sql_statements("SELECT foo FROM bar OFFSET 2 OFFSET 2");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: OFFSET".to_string()),
        res.unwrap_err()
    );

    let res = parse_sql_statements("SELECT foo FROM bar LIMIT 2 LIMIT 2");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: LIMIT".to_string()),
        res.unwrap_err()
    );

    let res = parse_sql_statements("SELECT foo FROM bar OFFSET 2 LIMIT 2 OFFSET 2");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: OFFSET".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_time_functions() {
    let sql = "SELECT CURRENT_TIMESTAMP()";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("CURRENT_TIMESTAMP")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );

    // Validating Parenthesis
    one_statement_parses_to("SELECT CURRENT_TIMESTAMP", sql);

    let sql = "SELECT CURRENT_TIME()";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("CURRENT_TIME")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );

    // Validating Parenthesis
    one_statement_parses_to("SELECT CURRENT_TIME", sql);

    let sql = "SELECT CURRENT_DATE()";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("CURRENT_DATE")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );

    // Validating Parenthesis
    one_statement_parses_to("SELECT CURRENT_DATE", sql);

    let sql = "SELECT LOCALTIME()";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("LOCALTIME")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );

    // Validating Parenthesis
    one_statement_parses_to("SELECT LOCALTIME", sql);

    let sql = "SELECT LOCALTIMESTAMP()";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("LOCALTIMESTAMP")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[0])
    );

    // Validating Parenthesis
    one_statement_parses_to("SELECT LOCALTIMESTAMP", sql);
}

#[test]
fn parse_position() {
    let sql = "SELECT POSITION('@' IN field)";
    let select = verified_only_select(sql);
    assert_eq!(
        &Expr::Position {
            expr: Box::new(Expr::Value(Value::SingleQuotedString("@".to_string()))),
            r#in: Box::new(Expr::Identifier(Ident::new("field"))),
        },
        expr_from_projection(only(&select.projection))
    );
}

#[test]
fn parse_position_negative() {
    let sql = "SELECT POSITION(foo) from bar";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Position function must include IN keyword".to_string()),
        res.unwrap_err()
    );

    let sql = "SELECT POSITION(foo IN) from bar";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError("Expected an expression:, found: )".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_is_boolean() {
    use self::Expr::*;

    let sql = "a IS TRUE";
    assert_eq!(
        IsTrue(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    let sql = "a IS NOT TRUE";
    assert_eq!(
        IsNotTrue(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    let sql = "a IS FALSE";
    assert_eq!(
        IsFalse(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    let sql = "a IS NOT FALSE";
    assert_eq!(
        IsNotFalse(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    let sql = "a IS UNKNOWN";
    assert_eq!(
        IsUnknown(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    let sql = "a IS NOT UNKNOWN";
    assert_eq!(
        IsNotUnknown(Box::new(Identifier(Ident::new("a")))),
        verified_expr(sql)
    );

    verified_stmt("SELECT f FROM foo WHERE field IS TRUE");
    verified_stmt("SELECT f FROM foo WHERE field IS NOT TRUE");

    verified_stmt("SELECT f FROM foo WHERE field IS FALSE");
    verified_stmt("SELECT f FROM foo WHERE field IS NOT FALSE");

    verified_stmt("SELECT f FROM foo WHERE field IS UNKNOWN");
    verified_stmt("SELECT f FROM foo WHERE field IS NOT UNKNOWN");

    let sql = "SELECT f from foo where field is 0";
    let res = parse_sql_statements(sql);
    assert_eq!(
        ParserError::ParserError(
            "Expected [NOT] NULL or TRUE|FALSE or [NOT] DISTINCT FROM after IS, found: 0"
                .to_string()
        ),
        res.unwrap_err()
    );
}

#[test]
fn parse_deeply_nested_parens_hits_recursion_limits() {
    let sql = "(".repeat(1000);
    let res = parse_sql_statements(&sql);
    assert_eq!(ParserError::RecursionLimitExceeded, res.unwrap_err());
}

#[test]
fn parse_deeply_nested_expr_hits_recursion_limits() {
    let dialect = GenericDialect {};

    let where_clause = make_where_clause(100);
    let sql = format!("SELECT id, user_id FROM test WHERE {where_clause}");

    let res = Parser::new(&dialect)
        .try_with_sql(&sql)
        .expect("tokenize to work")
        .parse_statements();

    assert_eq!(res, Err(ParserError::RecursionLimitExceeded));
}

#[test]
fn parse_deeply_nested_subquery_expr_hits_recursion_limits() {
    let dialect = GenericDialect {};

    let where_clause = make_where_clause(100);
    let sql = format!("SELECT id, user_id where id IN (select id from t WHERE {where_clause})");

    let res = Parser::new(&dialect)
        .try_with_sql(&sql)
        .expect("tokenize to work")
        .parse_statements();

    assert_eq!(res, Err(ParserError::RecursionLimitExceeded));
}

#[test]
fn parse_with_recursion_limit() {
    let dialect = GenericDialect {};

    let where_clause = make_where_clause(20);
    let sql = format!("SELECT id, user_id FROM test WHERE {where_clause}");

    // Expect the statement to parse with default limit
    let res = Parser::new(&dialect)
        .try_with_sql(&sql)
        .expect("tokenize to work")
        .parse_statements();

    assert!(matches!(res, Ok(_)), "{res:?}");

    // limit recursion to something smaller, expect parsing to fail
    let res = Parser::new(&dialect)
        .try_with_sql(&sql)
        .expect("tokenize to work")
        .with_recursion_limit(20)
        .parse_statements();

    assert_eq!(res, Err(ParserError::RecursionLimitExceeded));

    // limit recursion to 50, expect it to succeed
    let res = Parser::new(&dialect)
        .try_with_sql(&sql)
        .expect("tokenize to work")
        .with_recursion_limit(50)
        .parse_statements();

    assert!(matches!(res, Ok(_)), "{res:?}");
}

#[test]
fn parse_pivot_table() {
    let sql = concat!(
        "SELECT * FROM monthly_sales AS a ",
        "PIVOT(SUM(a.amount) FOR a.MONTH IN ('JAN', 'FEB', 'MAR', 'APR')) AS p (c, d) ",
        "ORDER BY EMPID"
    );

    assert_eq!(
        verified_only_select(sql).from[0].relation,
        Pivot {
            name: ObjectName(vec![Ident::new("monthly_sales")]),
            table_alias: Some(TableAlias {
                name: Ident::new("a"),
                columns: vec![]
            }),
            aggregate_function: Expr::Function(Function {
                name: ObjectName(vec![Ident::new("SUM")]),
                args: (vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(
                    Expr::CompoundIdentifier(vec![Ident::new("a"), Ident::new("amount"),])
                ))]),
                over: None,
                distinct: false,
                special: false,
            }),
            value_column: vec![Ident::new("a"), Ident::new("MONTH")],
            pivot_values: vec![
                Value::SingleQuotedString("JAN".to_string()),
                Value::SingleQuotedString("FEB".to_string()),
                Value::SingleQuotedString("MAR".to_string()),
                Value::SingleQuotedString("APR".to_string()),
            ],
            pivot_alias: Some(TableAlias {
                name: Ident {
                    value: "p".to_string(),
                    quote_style: None
                },
                columns: vec![Ident::new("c"), Ident::new("d")],
            }),
        }
    );
    assert_eq!(verified_stmt(sql).to_string(), sql);

    let sql_without_table_alias = concat!(
        "SELECT * FROM monthly_sales ",
        "PIVOT(SUM(a.amount) FOR a.MONTH IN ('JAN', 'FEB', 'MAR', 'APR')) AS p (c, d) ",
        "ORDER BY EMPID"
    );
    assert_matches!(
        verified_only_select(sql_without_table_alias).from[0].relation,
        Pivot {
            table_alias: None, // parsing should succeed with empty alias
            ..
        }
    );
    assert_eq!(
        verified_stmt(sql_without_table_alias).to_string(),
        sql_without_table_alias
    );
}

/// Makes a predicate that looks like ((user_id = $id) OR user_id = $2...)
fn make_where_clause(num: usize) -> String {
    use std::fmt::Write;
    let mut output = "(".repeat(num - 1);

    for i in 0..num {
        if i > 0 {
            write!(&mut output, " OR ").unwrap();
        }
        write!(&mut output, "user_id = {i}").unwrap();
        if i < num - 1 {
            write!(&mut output, ")").unwrap();
        }
    }
    output
}

#[test]
fn parse_non_latin_identifiers() {
    let supported_dialects = TestedDialects {
        dialects: vec![
            Box::new(GenericDialect {}),
            Box::new(PostgreSqlDialect {}),
            Box::new(RedshiftSqlDialect {}),
        ],
    };

    supported_dialects.verified_stmt("SELECT a.説明 FROM test.public.inter01 AS a");
    supported_dialects.verified_stmt("SELECT a.説明 FROM inter01 AS a, inter01_transactions AS b WHERE a.説明 = b.取引 GROUP BY a.説明");
    supported_dialects.verified_stmt("SELECT 説明, hühnervögel, garçon, Москва, 東京 FROM inter01");
    assert!(supported_dialects
        .parse_sql_statements("SELECT 💝 FROM table1")
        .is_err());
}

#[test]
fn parse_jinja_ref() {
    let sql = "SELECT 1 FROM {{ ref('model') }}";

    let stmt = verified_only_select(sql);
    assert_eq!(
        stmt.from[0].relation,
        TableFactor::DbtRef{
            model_name: Ident::with_quote('\'', "model"),
            alias: None
        }
    );
    assert_eq!(verified_stmt(sql).to_string(), sql);
}

#[test]
fn parse_jinja_ref_as() {
    let sql = "SELECT 1 FROM {{ ref('model') }} AS my_model";

    let stmt = verified_only_select(sql);
    assert_eq!(
        stmt.from[0].relation,
        TableFactor::DbtRef{
            model_name: Ident::with_quote('\'', "model"),
            alias: Some(TableAlias { name: Ident { value: "my_model".to_string(), quote_style: None }, columns: vec![] })
        }
    );
    assert_eq!(verified_stmt(sql).to_string(), sql);
}

#[test]
fn parse_jinja_source() {
    let sql = "SELECT 1 FROM {{ source('source_name', 'table_name') }}";

    let stmt = verified_only_select(sql);
    assert_eq!(
        stmt.from[0].relation,
        TableFactor::DbtSource{
            source_name: Ident::with_quote('\'', "source_name"),
            table_name: Ident::with_quote('\'', "table_name"),
            alias: None
        }
    );
    assert_eq!(verified_stmt(sql).to_string(), sql);
}

#[test]
fn parse_jinja_source_as() {
    let sql = "SELECT 1 FROM {{ source('source_name', 'table_name') }} AS my_source";

    let stmt = verified_only_select(sql);
    assert_eq!(
        stmt.from[0].relation,
        TableFactor::DbtSource{
            source_name: Ident::with_quote('\'', "source_name"),
            table_name: Ident::with_quote('\'', "table_name"),
            alias: Some(TableAlias { name: Ident { value: "my_source".to_string(), quote_style: None }, columns: vec![] })
        }
    );
    assert_eq!(verified_stmt(sql).to_string(), sql);
}