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
//! Test SQL syntax specific to PostgreSQL. The parser based on the
//! generic dialect is also tested (on the inputs it can handle).

#[macro_use]
mod test_utils;
use test_utils::*;

use dbtranslate::ast::*;
use dbtranslate::dialect::{GenericDialect, PostgreSqlDialect};

#[test]
fn test_postgres_create_table_errors() {
    let sql = "CREATE TABLE _my_$table (am00unt number)";
    let dialect = PostgreSqlDialect {};
    let error_message = "CREATE is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn test_postgres_drop_schema_errors() {
    let sql = "DROP SCHEMA IF EXISTS schema_name";
    let dialect = PostgreSqlDialect {};
    let error_message = "DROP is not supported by dbt-sqlparser";
    check_error(sql, error_message, &dialect);
}

#[test]
fn parse_pg_binary_ops() {
    let binary_ops = &[
        // Sharp char and Caret cannot be used with Generic Dialect, it conflicts with identifiers
        ("#", BinaryOperator::PGBitwiseXor, pg()),
        ("^", BinaryOperator::PGExp, pg()),
        (">>", BinaryOperator::PGBitwiseShiftRight, pg_and_generic()),
        ("<<", BinaryOperator::PGBitwiseShiftLeft, pg_and_generic()),
    ];

    for (str_op, op, dialects) in binary_ops {
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
fn parse_pg_unary_ops() {
    let pg_unary_ops = &[
        ("~", UnaryOperator::PGBitwiseNot),
        ("|/", UnaryOperator::PGSquareRoot),
        ("||/", UnaryOperator::PGCubeRoot),
        ("!!", UnaryOperator::PGPrefixFactorial),
        ("@", UnaryOperator::PGAbs),
    ];

    for (str_op, op) in pg_unary_ops {
        let select = pg().verified_only_select(&format!("SELECT {}a", &str_op));
        assert_eq!(
            SelectItem::UnnamedExpr(Expr::UnaryOp {
                op: *op,
                expr: Box::new(Expr::Identifier(Ident::new("a"))),
            }),
            select.projection[0]
        );
    }
}

#[test]
fn parse_pg_postfix_factorial() {
    let postfix_factorial = &[("!", UnaryOperator::PGPostfixFactorial)];

    for (str_op, op) in postfix_factorial {
        let select = pg().verified_only_select(&format!("SELECT a{}", &str_op));
        assert_eq!(
            SelectItem::UnnamedExpr(Expr::UnaryOp {
                op: *op,
                expr: Box::new(Expr::Identifier(Ident::new("a"))),
            }),
            select.projection[0]
        );
    }
}

#[test]
fn parse_pg_regex_match_ops() {
    let pg_regex_match_ops = &[
        ("~", BinaryOperator::PGRegexMatch),
        ("~*", BinaryOperator::PGRegexIMatch),
        ("!~", BinaryOperator::PGRegexNotMatch),
        ("!~*", BinaryOperator::PGRegexNotIMatch),
    ];

    for (str_op, op) in pg_regex_match_ops {
        let select = pg().verified_only_select(&format!("SELECT 'abc' {} '^a'", &str_op));
        assert_eq!(
            SelectItem::UnnamedExpr(Expr::BinaryOp {
                left: Box::new(Expr::Value(Value::SingleQuotedString("abc".into()))),
                op: op.clone(),
                right: Box::new(Expr::Value(Value::SingleQuotedString("^a".into()))),
            }),
            select.projection[0]
        );
    }
}

#[test]
fn parse_array_index_expr() {
    #[cfg(feature = "bigdecimal")]
    let num: Vec<Expr> = (0..=10)
        .map(|s| Expr::Value(Value::Number(bigdecimal::BigDecimal::from(s), false)))
        .collect();
    #[cfg(not(feature = "bigdecimal"))]
    let num: Vec<Expr> = (0..=10)
        .map(|s| Expr::Value(Value::Number(s.to_string(), false)))
        .collect();

    let sql = "SELECT foo[0] FROM foos";
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::ArrayIndex {
            obj: Box::new(Expr::Identifier(Ident::new("foo"))),
            indexes: vec![num[0].clone()],
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT foo[0][0] FROM foos";
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::ArrayIndex {
            obj: Box::new(Expr::Identifier(Ident::new("foo"))),
            indexes: vec![num[0].clone(), num[0].clone()],
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = r#"SELECT bar[0]["baz"]["fooz"] FROM foos"#;
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::ArrayIndex {
            obj: Box::new(Expr::Identifier(Ident::new("bar"))),
            indexes: vec![
                num[0].clone(),
                Expr::Identifier(Ident {
                    value: "baz".to_string(),
                    quote_style: Some('"')
                }),
                Expr::Identifier(Ident {
                    value: "fooz".to_string(),
                    quote_style: Some('"')
                })
            ],
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT (CAST(ARRAY[ARRAY[2, 3]] AS INT[][]))[1][2]";
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::ArrayIndex {
            obj: Box::new(Expr::Nested(Box::new(Expr::Cast {
                expr: Box::new(Expr::Array(Array {
                    elem: vec![Expr::Array(Array {
                        elem: vec![num[2].clone(), num[3].clone(),],
                        named: true,
                    })],
                    named: true,
                })),
                data_type: DataType::Array(Some(Box::new(DataType::Array(Some(Box::new(
                    DataType::Int(None)
                ))))))
            }))),
            indexes: vec![num[1].clone(), num[2].clone()],
        },
        expr_from_projection(only(&select.projection)),
    );

    let sql = "SELECT ARRAY[]";
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::Array(dbtranslate::ast::Array {
            elem: vec![],
            named: true
        }),
        expr_from_projection(only(&select.projection)),
    );
}

#[test]
fn parse_array_subquery_expr() {
    let sql = "SELECT ARRAY(SELECT 1 UNION SELECT 2)";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        &Expr::ArraySubquery(Box::new(Query {
            config: None,
            with: None,
            body: Box::new(SetExpr::SetOperation {
                op: SetOperator::Union,
                set_quantifier: SetQuantifier::None,
                left: Box::new(SetExpr::Select(Box::new(Select {
                    distinct: false,
                    top: None,
                    projection: vec![SelectItem::UnnamedExpr(Expr::Value(Value::Number(
                        #[cfg(not(feature = "bigdecimal"))]
                        "1".to_string(),
                        #[cfg(feature = "bigdecimal")]
                        bigdecimal::BigDecimal::from(1),
                        false,
                    )))],
                    into: None,
                    from: vec![],
                    lateral_views: vec![],
                    selection: None,
                    group_by: vec![],
                    cluster_by: vec![],
                    distribute_by: vec![],
                    sort_by: vec![],
                    having: None,
                    qualify: None,
                }))),
                right: Box::new(SetExpr::Select(Box::new(Select {
                    distinct: false,
                    top: None,
                    projection: vec![SelectItem::UnnamedExpr(Expr::Value(Value::Number(
                        #[cfg(not(feature = "bigdecimal"))]
                        "2".to_string(),
                        #[cfg(feature = "bigdecimal")]
                        bigdecimal::BigDecimal::from(2),
                        false,
                    )))],
                    into: None,
                    from: vec![],
                    lateral_views: vec![],
                    selection: None,
                    group_by: vec![],
                    cluster_by: vec![],
                    distribute_by: vec![],
                    sort_by: vec![],
                    having: None,
                    qualify: None,
                }))),
            }),
            order_by: vec![],
            limit: None,
            offset: None,
            jinja_variables: vec![],
        })),
        expr_from_projection(only(&select.projection)),
    );
}


#[test]
fn test_json() {
    let sql = "SELECT params ->> 'name' FROM events";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("params"))),
            operator: JsonOperator::LongArrow,
            right: Box::new(Expr::Value(Value::SingleQuotedString("name".to_string()))),
        }),
        select.projection[0]
    );

    let sql = "SELECT params -> 'name' FROM events";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("params"))),
            operator: JsonOperator::Arrow,
            right: Box::new(Expr::Value(Value::SingleQuotedString("name".to_string()))),
        }),
        select.projection[0]
    );

    let sql = "SELECT info -> 'items' ->> 'product' FROM orders";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("info"))),
            operator: JsonOperator::Arrow,
            right: Box::new(Expr::JsonAccess {
                left: Box::new(Expr::Value(Value::SingleQuotedString("items".to_string()))),
                operator: JsonOperator::LongArrow,
                right: Box::new(Expr::Value(Value::SingleQuotedString(
                    "product".to_string()
                )))
            }),
        }),
        select.projection[0]
    );

    let sql = "SELECT info #> '{a,b,c}' FROM orders";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("info"))),
            operator: JsonOperator::HashArrow,
            right: Box::new(Expr::Value(Value::SingleQuotedString(
                "{a,b,c}".to_string()
            ))),
        }),
        select.projection[0]
    );

    let sql = "SELECT info #>> '{a,b,c}' FROM orders";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("info"))),
            operator: JsonOperator::HashLongArrow,
            right: Box::new(Expr::Value(Value::SingleQuotedString(
                "{a,b,c}".to_string()
            ))),
        }),
        select.projection[0]
    );

    let sql = "SELECT info FROM orders WHERE info @> '{\"a\": 1}'";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::new("info"))),
            operator: JsonOperator::AtArrow,
            right: Box::new(Expr::Value(Value::SingleQuotedString(
                "{\"a\": 1}".to_string()
            ))),
        },
        select.selection.unwrap(),
    );

    let sql = "SELECT info FROM orders WHERE '{\"a\": 1}' <@ info";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        Expr::JsonAccess {
            left: Box::new(Expr::Value(Value::SingleQuotedString(
                "{\"a\": 1}".to_string()
            ))),
            operator: JsonOperator::ArrowAt,
            right: Box::new(Expr::Identifier(Ident::new("info"))),
        },
        select.selection.unwrap(),
    );

    let sql = "SELECT info #- ARRAY['a', 'b'] FROM orders";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::from("info"))),
            operator: JsonOperator::HashMinus,
            right: Box::new(Expr::Array(Array {
                elem: vec![
                    Expr::Value(Value::SingleQuotedString("a".to_string())),
                    Expr::Value(Value::SingleQuotedString("b".to_string())),
                ],
                named: true,
            })),
        }),
        select.projection[0],
    );

    let sql = "SELECT info FROM orders WHERE info @? '$.a'";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::from("info"))),
            operator: JsonOperator::AtQuestion,
            right: Box::new(Expr::Value(Value::SingleQuotedString("$.a".to_string())),),
        },
        select.selection.unwrap(),
    );

    let sql = "SELECT info FROM orders WHERE info @@ '$.a'";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        Expr::JsonAccess {
            left: Box::new(Expr::Identifier(Ident::from("info"))),
            operator: JsonOperator::AtAt,
            right: Box::new(Expr::Value(Value::SingleQuotedString("$.a".to_string())),),
        },
        select.selection.unwrap(),
    );
}

#[test]
fn test_composite_value() {
    let sql = "SELECT (on_hand.item).name FROM on_hand WHERE (on_hand.item).price > 9";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::CompositeAccess {
            key: Ident::new("name"),
            expr: Box::new(Expr::Nested(Box::new(Expr::CompoundIdentifier(vec![
                Ident::new("on_hand"),
                Ident::new("item")
            ]))))
        }),
        select.projection[0]
    );

    #[cfg(feature = "bigdecimal")]
    let num: Expr = Expr::Value(Value::Number(bigdecimal::BigDecimal::from(9), false));
    #[cfg(not(feature = "bigdecimal"))]
    let num: Expr = Expr::Value(Value::Number("9".to_string(), false));
    assert_eq!(
        select.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::CompositeAccess {
                key: Ident::new("price"),
                expr: Box::new(Expr::Nested(Box::new(Expr::CompoundIdentifier(vec![
                    Ident::new("on_hand"),
                    Ident::new("item")
                ]))))
            }),
            op: BinaryOperator::Gt,
            right: Box::new(num)
        })
    );

    let sql = "SELECT (information_schema._pg_expandarray(ARRAY['i', 'i'])).n";
    let select = pg().verified_only_select(sql);
    assert_eq!(
        SelectItem::UnnamedExpr(Expr::CompositeAccess {
            key: Ident::new("n"),
            expr: Box::new(Expr::Nested(Box::new(Expr::Function(Function {
                name: ObjectName(vec![
                    Ident::new("information_schema"),
                    Ident::new("_pg_expandarray")
                ]),
                args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Array(
                    Array {
                        elem: vec![
                            Expr::Value(Value::SingleQuotedString("i".to_string())),
                            Expr::Value(Value::SingleQuotedString("i".to_string())),
                        ],
                        named: true
                    }
                )))],
                over: None,
                distinct: false,
                special: false
            }))))
        }),
        select.projection[0]
    );
}

#[test]
fn parse_quoted_identifier() {
    pg_and_generic().verified_stmt(r#"SELECT "quoted "" ident""#);
}

#[test]
fn parse_quoted_identifier_2() {
    pg_and_generic().verified_stmt(r#"SELECT """quoted ident""""#);
}

fn pg() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(PostgreSqlDialect {})],
    }
}

fn pg_and_generic() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(PostgreSqlDialect {}), Box::new(GenericDialect {})],
    }
}

#[test]
fn parse_escaped_literal_string() {
    let sql =
        r#"SELECT E's1 \n s1', E's2 \\n s2', E's3 \\\n s3', E's4 \\\\n s4', E'\'', E'foo \\'"#;
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(6, select.projection.len());
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("s1 \n s1".to_string())),
        expr_from_projection(&select.projection[0])
    );
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("s2 \\n s2".to_string())),
        expr_from_projection(&select.projection[1])
    );
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("s3 \\\n s3".to_string())),
        expr_from_projection(&select.projection[2])
    );
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("s4 \\\\n s4".to_string())),
        expr_from_projection(&select.projection[3])
    );
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("'".to_string())),
        expr_from_projection(&select.projection[4])
    );
    assert_eq!(
        &Expr::Value(Value::EscapedStringLiteral("foo \\".to_string())),
        expr_from_projection(&select.projection[5])
    );

    let sql = r#"SELECT E'\'"#;
    assert_eq!(
        pg_and_generic()
            .parse_sql_statements(sql)
            .unwrap_err()
            .to_string(),
        "sql parser error: Unterminated encoded string literal at Line: 1, Column 8"
    );
}

#[test]
fn parse_current_functions() {
    let sql = "SELECT CURRENT_CATALOG, CURRENT_USER, SESSION_USER, USER";
    let select = pg_and_generic().verified_only_select(sql);
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("CURRENT_CATALOG")]),
            args: vec![],
            over: None,
            distinct: false,
            special: true,
        }),
        expr_from_projection(&select.projection[0])
    );
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("CURRENT_USER")]),
            args: vec![],
            over: None,
            distinct: false,
            special: true,
        }),
        expr_from_projection(&select.projection[1])
    );
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("SESSION_USER")]),
            args: vec![],
            over: None,
            distinct: false,
            special: true,
        }),
        expr_from_projection(&select.projection[2])
    );
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::new("USER")]),
            args: vec![],
            over: None,
            distinct: false,
            special: true,
        }),
        expr_from_projection(&select.projection[3])
    );
}

#[test]
fn parse_custom_operator() {
    // operator with a database and schema
    let sql = r#"SELECT * FROM events WHERE relname OPERATOR(database.pg_catalog.~) '^(table)$'"#;
    let select = pg().verified_only_select(sql);
    assert_eq!(
        select.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident {
                value: "relname".into(),
                quote_style: None,
            })),
            op: BinaryOperator::PGCustomBinaryOperator(vec![
                "database".into(),
                "pg_catalog".into(),
                "~".into()
            ]),
            right: Box::new(Expr::Value(Value::SingleQuotedString("^(table)$".into())))
        })
    );

    // operator with a schema
    let sql = r#"SELECT * FROM events WHERE relname OPERATOR(pg_catalog.~) '^(table)$'"#;
    let select = pg().verified_only_select(sql);
    assert_eq!(
        select.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident {
                value: "relname".into(),
                quote_style: None,
            })),
            op: BinaryOperator::PGCustomBinaryOperator(vec!["pg_catalog".into(), "~".into()]),
            right: Box::new(Expr::Value(Value::SingleQuotedString("^(table)$".into())))
        })
    );

    // custom operator without a schema
    let sql = r#"SELECT * FROM events WHERE relname OPERATOR(~) '^(table)$'"#;
    let select = pg().verified_only_select(sql);
    assert_eq!(
        select.selection,
        Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident {
                value: "relname".into(),
                quote_style: None,
            })),
            op: BinaryOperator::PGCustomBinaryOperator(vec!["~".into()]),
            right: Box::new(Expr::Value(Value::SingleQuotedString("^(table)$".into())))
        })
    );
}

#[test]
fn parse_delimited_identifiers() {
    // check that quoted identifiers in any position remain quoted after serialization
    let select = pg().verified_only_select(
        r#"SELECT "alias"."bar baz", "myfun"(), "simple id" AS "column alias" FROM "a table" AS "alias""#,
    );
    // check FROM
    match only(select.from).relation {
        TableFactor::Table {
            name,
            alias,
            args,
            with_hints,
        } => {
            assert_eq!(vec![Ident::with_quote('"', "a table")], name.0);
            assert_eq!(Ident::with_quote('"', "alias"), alias.unwrap().name);
            assert!(args.is_none());
            assert!(with_hints.is_empty());
        }
        _ => panic!("Expecting TableFactor::Table"),
    }
    // check SELECT
    assert_eq!(3, select.projection.len());
    assert_eq!(
        &Expr::CompoundIdentifier(vec![
            Ident::with_quote('"', "alias"),
            Ident::with_quote('"', "bar baz"),
        ]),
        expr_from_projection(&select.projection[0]),
    );
    assert_eq!(
        &Expr::Function(Function {
            name: ObjectName(vec![Ident::with_quote('"', "myfun")]),
            args: vec![],
            over: None,
            distinct: false,
            special: false,
        }),
        expr_from_projection(&select.projection[1]),
    );
    match &select.projection[2] {
        SelectItem::ExprWithAlias { expr, alias } => {
            assert_eq!(&Expr::Identifier(Ident::with_quote('"', "simple id")), expr);
            assert_eq!(&Ident::with_quote('"', "column alias"), alias);
        }
        _ => panic!("Expected ExprWithAlias"),
    }

}

#[test]
fn parse_like() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}LIKE '%a'",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::Like {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: None,
            },
            select.selection.unwrap()
        );

        // Test with escape char
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}LIKE '%a' ESCAPE '\\'",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::Like {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: Some('\\'),
            },
            select.selection.unwrap()
        );

        // This statement tests that LIKE and NOT LIKE have the same precedence.
        // This was previously mishandled (#81).
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}LIKE '%a' IS NULL",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::IsNull(Box::new(Expr::Like {
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
fn parse_similar_to() {
    fn chk(negated: bool) {
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}SIMILAR TO '%a'",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::SimilarTo {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: None,
            },
            select.selection.unwrap()
        );

        // Test with escape char
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}SIMILAR TO '%a' ESCAPE '\\'",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::SimilarTo {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: Some('\\'),
            },
            select.selection.unwrap()
        );

        // This statement tests that SIMILAR TO and NOT SIMILAR TO have the same precedence.
        let sql = &format!(
            "SELECT * FROM customers WHERE name {}SIMILAR TO '%a' ESCAPE '\\' IS NULL",
            if negated { "NOT " } else { "" }
        );
        let select = pg().verified_only_select(sql);
        assert_eq!(
            Expr::IsNull(Box::new(Expr::SimilarTo {
                expr: Box::new(Expr::Identifier(Ident::new("name"))),
                negated,
                pattern: Box::new(Expr::Value(Value::SingleQuotedString("%a".to_string()))),
                escape_char: Some('\\'),
            })),
            select.selection.unwrap()
        );
    }
    chk(false);
    chk(true);
}


#[test]
fn parse_dollar_quoted_string() {
    let sql = "SELECT $$hello$$, $tag_name$world$tag_name$, $$Foo$Bar$$, $$Foo$Bar$$col_name, $$$$, $tag_name$$tag_name$";

    let stmts = pg().parse_sql_statements(sql).unwrap();
    let Statement::Query(query) = stmts.get(0).unwrap();

    let projection = if let SetExpr::Select(select) = &*query.body {
        &select.projection
    } else {
        panic!("Expected a Select in the query body");
    };

    assert_eq!(
        &Expr::Value(Value::DollarQuotedString(DollarQuotedString {
            tag: None,
            value: "hello".into()
        })),
        expr_from_projection(&projection[0])
    );

    assert_eq!(
        &Expr::Value(Value::DollarQuotedString(DollarQuotedString {
            tag: Some("tag_name".into()),
            value: "world".into()
        })),
        expr_from_projection(&projection[1])
    );

    assert_eq!(
        &Expr::Value(Value::DollarQuotedString(DollarQuotedString {
            tag: None,
            value: "Foo$Bar".into()
        })),
        expr_from_projection(&projection[2])
    );

    assert_eq!(
        projection[3],
        SelectItem::ExprWithAlias {
            expr: Expr::Value(Value::DollarQuotedString(DollarQuotedString {
                tag: None,
                value: "Foo$Bar".into(),
            })),
            alias: Ident {
                value: "col_name".into(),
                quote_style: None,
            },
        }
    );

    assert_eq!(
        expr_from_projection(&projection[4]),
        &Expr::Value(Value::DollarQuotedString(DollarQuotedString {
            tag: None,
            value: "".into()
        })),
    );

    assert_eq!(
        expr_from_projection(&projection[5]),
        &Expr::Value(Value::DollarQuotedString(DollarQuotedString {
            tag: Some("tag_name".into()),
            value: "".into()
        })),
    );
}

#[test]
fn parse_incorrect_dollar_quoted_string() {
    let sql = "SELECT $x$hello$$";
    assert!(pg().parse_sql_statements(sql).is_err());

    let sql = "SELECT $hello$$";
    assert!(pg().parse_sql_statements(sql).is_err());

    let sql = "SELECT $$$";
    assert!(pg().parse_sql_statements(sql).is_err());
}
