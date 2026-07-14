#!/bin/bash

reps=10

rm -f out.* err.*
cargo build --release

for n in $(seq 1 $reps)


do
  echo ==== $n ====
  for size in $(seq 1 27)
  do
    vec_size=$(( 2 ** $size ))
    echo running size $vec_size
    srun -N 2 run_vec.sh --vec-len 10 -> out.$size 2> err.$size &
  done
  wait

  if [ $n -eq 1 ]
  then
    cat out.* | grep Clients | head -n 1 > results_vec_rrmi.csv
  fi
  cat $(ls out* | sort -V) | grep -A1 --no-group-separator  Clients | grep -v Clients  >> results_vec_rrmi.csv
  rm out.* err.*

done