## Configuring dbtonic
Right now, the configuration for `dbtonic` is done through a file in your project called `dbtonic.toml`. If you're unfamiliar with toml, it's very similar to yaml but the keys are contained in [] and values following.

Here is an example of the file

``` dbtonic.toml
[rules]
unique_not_null_or_combination_rule = false
model_yaml_exists = false
```