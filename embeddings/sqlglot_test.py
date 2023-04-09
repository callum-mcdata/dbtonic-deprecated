from sqlglot import diff, parse_one

sql = "SELECT a + 1 AS z from {{ ref('model') }}"
print(repr(parse_one(sql)))
