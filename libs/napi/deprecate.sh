#!/bin/bash

declare -A packages=(
  ["@graphql-conductor/lib"]="0.0.10 0.0.9 0.0.8 0.0.4 0.0.3 0.0.2 0.0.1"
  ["@graphql-conductor/bin"]="1.0.5 1.0.4 1.0.0"
  ["@graphql-conductor/lib-darwin-arm64"]="0.0.10 0.0.8 0.0.7 0.0.6"
  ["@graphql-conductor/lib-darwin-x64"]="0.0.10 0.0.8 0.0.7 0.0.6"
  ["@graphql-conductor/lib-linux-x64-gnu"]="0.0.10 0.0.8 0.0.7 0.0.6"
)

deprecation_message="This version is deprecated as it was for debugging and development purposes."

for package_name in "${!packages[@]}"; do
  for version in ${packages[$package_name]}; do
    npm deprecate "$package_name@$version" "$deprecation_message"
  done
done
