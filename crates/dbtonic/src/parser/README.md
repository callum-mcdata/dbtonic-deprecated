## The Parser
This code is shamelessly stolen from dbt-extractor, a wonderful repo by  my colleague Nate May that is used in dbt-core. I was having issues getting it to work as a package import so I just copied over the two files I really wanted.

It has been edited to support the identification of two new types:
- Variables 
- Macros

This was done because both of those were throwing syntax errors. That's not  an issue in dbt-core because they fall back on dbt parsing but I am trying to do everything in rust so alternative solutions were needed!

The changes for `var` were easy - I just had to introduce it as a new enumeration of `Extraction` and then write in the behavior I wanted. Macros were a bit trickier - the `.` would throw out some gnarly errors. Luckily my colleague Devon Fulcher had a PR up against tree-sitter-jinja2 to support this behavior for something that he was working on. So once again I was able to take advantage of the work done by my peers! The best feeling.

The one sort of hacky workaround I did was just ignoring the ParseErrors returned when there is a jinja iterator (ie for, if, etc). tree-sitter-jinja2 throws ParseErrors on single curly jinja and I didn't have enough reasons to really need to parse that info out. Maybe I will in the future, but for now not so much.

**Outputs**:
```
VarT(
    "variable_name",
),

MacroT(
    "dbt_utils",
    "generate_surrogate_key",
),
```