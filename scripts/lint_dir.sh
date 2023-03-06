if [ $# -ne 1 ]; then
    echo "Usage: $0 <SRC_DIR>"
    exit 1
fi
SRC_DIR=$1

# glob all *.lua file
for file in $(find $SRC_DIR -name "*.lua"); do
    echo "Testing $file"
    sh ./scripts/test_single.sh $file binary

    if [ $? -ne 0 ]; then
        echo "Failed to test $file"
        exit 1
    fi
done