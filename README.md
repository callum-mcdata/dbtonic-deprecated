### Near Term:
- Find a way for it to recognize and ignore single quoted jinja loops

### Long Term
- Determine a consistent format for the rules engine to operate over
    - Quigley used the model for MF. Should I create somethign similar? A vector
    - where each node contains {AST, raw_sql, & yaml}

### Components
- Need a way to parse the yml and associate it with each node
    - maybe do valdation on shape if this is long-term some form of dbt project eval?
-  