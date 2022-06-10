#!/bin/bash
set -e

function defer() {
  rm -f result.csv
  rm -f data/result.csv
}

trap defer EXIT

if [ ! -f Cargo.toml ]; then
  echo "run this script from the project's root directory"
  exit 1
fi

cargo run -- data/input_test.csv > data/result.csv

sed -i '' 1d data/result.csv
sort data/result.csv -o data/result.csv
sed -i '' -e '1s;^;client, available, held, total, locked\n;' data/result.csv

if cmp data/result.csv data/input_test.output.csv; then
  echo "test passed"
else
  echo "\n"
  diff data/result.csv data/input_test.output.csv
  exit 1
fi
