#!/bin/bash

sequenceNumberCalls=100000

rm -f out.* err.*

for size in $(seq 2 10)
do
  echo running size $size
  srun -N $size run.sh $sequenceNumberCalls > out.$size 2> err.$size
done
