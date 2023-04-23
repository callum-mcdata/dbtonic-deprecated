
# The Roadmap
This section contains all my notes on what the future could look like. In the short term I am using this as an alternative to Github issues because it is easier for me to keep track of. 

## TODO List
- DONE! I used the multiline comment functionality in the tokenizer. __Figure out how to make `{#` be recognized as a comment.__
- Done! It is called `get-ast`. __Create an AST endpoint for returning an AST__
- Done! It is called `get-tokens`. __Create a Token list endpoint that returns all the tokens__
- Add parsing for jinja loops
  - I need to add some sort of AST construct a la DbtConfig for jinja. 
  - From my notes mucking about, one example of this is an Expr that we'd want to support for column-type macros.
- Add parsing for Vars
- Switch sqlparser-rs from local link to Github repo under my account called dbt-sqlparser.

## Short Term
- TODO: I need to figure out how to recursively parse through the AST to quickly check things like sources, refs, relations. Sqlparser-rs has something called `visitor` functionality that would seem to accomplish this but I am fuzzy on how it works. I'd need to add this for dbt elements.
- Change ParserError to include an the code that is causing the issue. Given parser probably working on tokens can I make it take a step back to the sql?

## Medium Term
- Determine a consistent format for the rules engine to operate over. 
    - The format is the ModelNode. It should have:
        - model_name CHECK
        - raw sql CHECK
        - ast CHECK
        - token list CHECK
        - yml struct CHECK
        - maybe depends on or depended on?
- Implement all the rules from dbt_project_evaluator.
- Done! The file is called `dbtonic.toml`. __Figure out a way for the user to configure those rules. Can I use `ruff` as a baseline here?__
- Add a way for people to pip install dbtonic and use that way

## Long Term
- Figure out how model versioning completely blows up my parsing logic :sweat:

## Longest Term
- Can I figure out a way to use OpenAI APIs to consistently document columns if a user hasn't? Would be optional
    - Has potential to get expensive. I'd want to batch the API calls. 