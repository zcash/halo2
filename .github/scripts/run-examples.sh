#!/bin/sh

# Get the list of examples from "examples" dir & Cargo.toml
EXAMPLES_WITH_FEATURES=$(awk '/^\[\[example\]\]/ { getline; name=$3; name=substr(name, 2, length(name)-2); getline; if ($1 == "required-features") { features=$NF; gsub(/["\[\]]/, "", features); print name "#" features } }' ./halo2_proofs/Cargo.toml)
EXAMPLES_WITHOUT_FEATURES=$(ls ./halo2_proofs/examples/*.rs | xargs -n1 basename -s .rs)

# Remove examples with features listed in Cargo.toml from examples without features
EXAMPLES_WITHOUT_FEATURES=$(echo "$EXAMPLES_WITHOUT_FEATURES" | grep -vFx "$(echo "$EXAMPLES_WITH_FEATURES" | cut -d '#' -f 1)")

# Combine examples with and without features
EXAMPLES=$(echo "$EXAMPLES_WITH_FEATURES $EXAMPLES_WITHOUT_FEATURES" | tr ' ' '\n' | sort -u | tr '\n' ' ')

# Run the examples
for example in $EXAMPLES; do
    if [ "$(echo "$example" | grep '#')" ]; then
        name=$(echo $example | cut -d '#' -f 1)
        features=$(echo $example | cut -d '#' -f 2)
        cargo run --package halo2_proofs --example $name --features $features
    else
        cargo run --package halo2_proofs --example $example
    fi
done
