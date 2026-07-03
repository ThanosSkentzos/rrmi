# tail -q -n 1 $(ls out*) | sort -n -t, > results.csv
cat logs_eth/out.* | grep Clients | head -n 1 > results_rust.csv
cat $(ls logs_eth/out* | sort -V) | grep -A1 --no-group-separator  Clients | grep -v Clients  >> results_rust.csv
