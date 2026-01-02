#!/bin/bash
set -e

SCHEMA_URL="https://api.linear.app/graphql"

# Introspection query
QUERY='{"query":"query IntrospectionQuery { __schema { queryType { name } mutationType { name } subscriptionType { name } types { ...FullType } directives { name description locations args { ...InputValue } } } } fragment FullType on __Type { kind name description fields(includeDeprecated: true) { name description args { ...InputValue } type { ...TypeRef } isDeprecated deprecationReason } inputFields { ...InputValue } interfaces { ...TypeRef } enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason } possibleTypes { ...TypeRef } } fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue } fragment TypeRef on __Type { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name } } } } } } } }"}'

echo "Fetching Linear GraphQL schema..."
curl -s -X POST "$SCHEMA_URL" \
  -H "Content-Type: application/json" \
  -d "$QUERY" | jq -r '.data' > schema.json

echo "Schema saved to schema.json"
echo "Note: cynic uses JSON introspection format"
