#!/bin/bash

reps=10

rm -f out.* err.*
cargo build --release --features infiniband
out=results_vec_rrmi_ib.csv

for size in $(seq 1 27)

do
  echo running size 2^$size
  for n in $(seq 1 $reps)
  do
    vec_size=$(( 2 ** $size ))
    srun -N 2 -J $"vec_2^$size" run_vec.sh --vec-len $vec_size> out.$size.$n 2> err.$size.$n &
  done
  wait

  if [ $size -eq 1 ]
  then
    cat out.* | grep Clients | head -n 1 > $out 
  fi
  cat $(ls out* | sort -V) | grep -A1 --no-group-separator  Clients | grep -v Clients  >> $out
  rm out.* err.*

done