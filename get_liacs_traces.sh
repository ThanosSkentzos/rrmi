scp client:/local/s4398831/rrmi/trace* . && scp server:/local/s4398831/rrmi/trace* .
sed -i 's/"tid":0/"tid":123/g' trace0065074.json

head -n -1 trace0065073.json | sed '$ s/$/,/' > traces_combined.json
tail -n +2 trace0065074.json >> traces_combined.json