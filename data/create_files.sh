#!/bin/bash

for i in {1..40000000}; do echo "deposit, 1, $i, 1.0"; done > single_client.csv
for i in {1..40000000}; do j=$(($i % 65535)); echo "deposit, $j, $i, 1.0"; done > many_clients.csv

sed -i '' -e '1s;^;type,client,tx,amount\n;' single_client.csv
sed -i '' -e '1s;^;type,client,tx,amount\n;' many_clients.csv
