START=00
END=40

# generate the files
for i in $(seq -f "%02g" $START $END); do
    ID=AP02000{$i}
    TOKEN=$(stool --method token --team_id ID)
    echo "team=${ID} token='${TOKEN}'" > $ID
    stool --method return --temporary-token $TOKEN
done