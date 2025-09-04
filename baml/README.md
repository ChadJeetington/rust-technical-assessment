Starter BAML Code

https://docs.boundaryml.com/guide/installation-language/rest-api-other-languages

# Starter code for BAML
npx @boundaryml/baml init \
  --client-type rest/openapi --openapi-client-type rust

# Generate baml_client code from baml_source
npx @boundaryml/baml dev --preview
or
npx @boundaryml/baml dev

# Generating baml_client code 
 npx @boundaryml/baml generate