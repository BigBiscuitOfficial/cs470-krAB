#!/bin/bash

# Check if a directory was provided
# if [ -z "$1" ]; then
#     echo "Usage: $0 <directory_path>"
#     exit 1
# fi

TARGET_DIR=./output/scaling_results/mpi/
OUTPUT_FILE="scaling_results.csv"

# Move into the target directory
cd "$TARGET_DIR" || { echo "Directory not found"; exit 1; }

# Check if there are any CSV files to process
# ls *.csv >/dev/null 2>&1
# if [ $? -ne 0 ]; then
#     echo "No CSV files found in $TARGET_DIR"
#     exit 1
# fi

echo "Combining files into $OUTPUT_FILE..."

# 1. Take the header from the first CSV file found
first_file=$(ls *.csv | head -n 1)
head -n 1 "$first_file" > "$OUTPUT_FILE"

# 2. Append data from all CSVs (skipping headers) 
# We exclude the output file itself from the globbing
for file in *.csv; do
    if [ "$file" != "$OUTPUT_FILE" ]; then
        tail -n +2 "$file" >> "$OUTPUT_FILE"
    fi
done

echo "Combination complete. Deleting source files..."

# 3. Delete the original CSV files (excluding our new combined file)
find . -maxdepth 1 -name "*.csv" ! -name "$OUTPUT_FILE" -type f -delete

echo "Done! All source files removed."
