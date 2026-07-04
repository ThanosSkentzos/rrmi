#!/bin/bash

sequenceNumberCalls=100000
reps=10

rm -f out.* err.*
cargo build --release --features infiniband

for n in $(seq 1 $reps)


do
  echo ==== $n ====
  for size in $(seq 2 11)
  do
    echo running size $size
    srun -N $size run.sh $sequenceNumberCalls > out.$size 2> err.$size &
  done
  wait

  if [ $n -eq 1 ]
  then
    cat out.* | grep Clients | head -n 1 > results_rust_ib.csv
  fi
  cat $(ls out* | sort -V) | grep -A1 --no-group-separator  Clients | grep -v Clients  >> results_rust_ib.csv
  rm out.* err.*

done