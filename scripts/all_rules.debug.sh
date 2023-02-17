SCRIPT_DIR=$(dirname "$0")
RULES_JSON=$SCRIPT_DIR/_all_rules.jsonc
RULES_STR="$(cat $RULES_JSON)"
echo "$RULES_STR"
RULES_STR=$(echo "$RULES_STR" | sed -e 's/\/\/.*//g')
export LUALINT_LOG=trace
CMD="cargo run -- run --rules '$RULES_STR' $1"

echo $CMD

eval $CMD

