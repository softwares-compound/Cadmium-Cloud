#!/bin/bash

# Define the output file
output_file="project_code.txt"

# Initialize the output file
echo "Project Code Listing" > "$output_file"
echo "====================" >> "$output_file"
echo "" >> "$output_file"

# Find all files excluding the 'target' directory
find . -type f ! -path "./target/*" \( -name "*.rs" -o -name "*.toml" \) | while read -r file; do
    echo "Processing $file"
    echo "File: $file" >> "$output_file"
    echo "--------------------" >> "$output_file"
    cat "$file" >> "$output_file"
    echo "" >> "$output_file"
    echo "" >> "$output_file"
done

echo "Code listing has been saved to $output_file"
