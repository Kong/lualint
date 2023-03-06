#
# Usage:
#     sh ./scripts/test_single.sh <lua_file> [binary/cargo_run]
# 
#     binary: use the binary in target/debug/lualint
#     cargo_run: use cargo run -- run (default)
# 

SCRIPT_DIR=$(dirname "$0")
RULES_JSON=$SCRIPT_DIR/all_rules.jsonc
RULES_STR="$(cat $RULES_JSON)"
# echo "$RULES_STR"
RULES_STR=$(echo "$RULES_STR" | sed -e 's/\/\/.*//g') # strip comments
export LUALINT_LOG=trace

BINARY=$SCRIPT_DIR/../target/debug/lualint
CARGO_RUN="cargo run -- "

if [ $# -eq 2 ]; then
    if [ $2 = "binary" ]; then
        CMD="$BINARY run --rules '$RULES_STR' $1"
    elif [ $2 = "cargo_run" ]; then
        CMD="$CARGO_RUN run --rules '$RULES_STR' $1"
    else
        echo "Unknown option: $2"
        exit 1
    fi
else
    CMD="$CARGO_RUN run --rules '$RULES_STR' $1"
fi

# echo $CMD

eval $CMD

