# Welcome to dbtonic

dbtonic is a dbt linter designed from the ground up for use with dbt. 

## Why dbtonic
The term `dbtonic` is used sporadically in the dbt community to describe dbt code that satisfies our best practices. Some of these practices are defined in the dbt Labs LINK HERE while others are a bit more ephemeral. We figured that codifying these into a linter would help ensure that every dbt project can benefit from the collective experience of the community.

The inspiration for this project came from INSERT NAME's original post around the `ruff` python linter. He stated:
> Ruff is based on two core hypothesis:
> - Python tooling could be rewritten in more performant langauges
> - An integrated toolchain can tap into efficiencies that aren't available to a disparate set of tools

It got me thinking about what a similar implementation for dbt would look like. I decided to start from the `dbt_project_evaluator` dbt package that some of my amazing colleagues worked on and see what it would look like if we converted it to Rust. 

## Why use dbtonic

If you're finding this repo before I post about it publically, I'd recommend you turn around and cast this from your mind. I am not a professional programmer and this repo probably has bugs galore! 

If I've announced it publically then the above statement is still true but I've decided that I have enough confidence in its use for experimental use. The actual reason is easy - it's fast! dbt developers deserve great experiences that aren't constrained by the limitations of languages we're most familiar with. We should have access to tools that operate just like any other programming language. 

dbtonic's long term vision is to provide a dbt-first linting experience. **However**, I have a day job at dbt Labs that does not include this area so lets just say development is sporadic!

# The Future
This section contains all my notes on what the future could look like. In the short term I am using this as an alternative to Github issues because it is easier for me to keep track of. 

## Short Term
- TODO: We need to figure out how to recursively parse through the AST to quickly check things like sources, refs, relations.
- TODO: I need to add parsing for configs. This should go in `parse_statement` as a top level and not parse_prefix because configs will always be at the top of the model, never in the body.
  - I should probably figure out what enum DbtConfig should go in.
- TODO: I need to add parsing for jinja loops. This probably should be in parse_expr or parse_prefix. Should it return an expr? 

- I am creating a fork of dbtparser-rs. This seems easier than using treesitter with what I want to do.
    - I need to add Config, Var, & Source to Statements.
    - I need to add the Display behavior for all those Statements.
    - I might need to add logic for jinja with DoubleLBrance & DoubleRBrace
    - I might need to add logic for jinja with LJinjaIterator & RJinjaIterator 

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
- Figure out a way for the user to configure those rules. Can I use `ruff` as a baseline here?


## Longest Term
- Can I figure out a way to use OpenAI APIs to consistently document columns if a user hasn't? Would be optional
    - Has potential to get expensive. I'd want to batch the API calls. 

## Components
- Need a way to parse the yml and associate it with each node
    - maybe do valdation on shape if this is long-term some form of dbt project eval?
    - serde-yaml looks promising.
-  