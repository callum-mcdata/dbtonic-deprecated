## Rules

- Unique / Not Null Or Combination Rule:
  - name: unique_not_null_or_combination_rule
  - description: Each model should contain either a single column with the unique and not_null tests OR the dbt_utils.unique_combinations test at the  model level.

- Yaml Defined Rule:
  - name: yaml_exists
  - description: The model must be defined in yaml somewhere in your project.