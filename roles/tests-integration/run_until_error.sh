#!/bin/bash

# Script to run tests until "Permission denied (os error 13)" error is found
# Change to the tests-integration directory
cd "$(dirname "$0")"

# Counter for number of attempts
attempt=1

echo "Starting test runs to reproduce 'Permission denied (os error 13)' error..."
echo "Press Ctrl+C to stop the script at any time."

while true; do
    echo "---------------------------------------------"
    echo "Attempt #$attempt"
    echo "---------------------------------------------"
    
    # Delete the template-provider directory if it exists
    if [ -d "template-provider" ]; then
        echo "Deleting template-provider directory..."
        rm -rf template-provider
    fi
    
    # Run the cargo test command and capture its output
    echo "Running cargo test..."

    # running an isolated test, we never witness the error
    # running all tests, we see the error
    
    # output=$(cargo t success_pool_template_provider_connection 2>&1)
    output=$(cargo t 2>&1)
    
    # Always show the output
    echo "Test output:"
    echo "$output"
    echo "---------------------------------------------"
    
    # Check if the error message is in the output
    if echo "$output" | grep -q "Permission denied (os error 13)"; then
        echo "---------------------------------------------"
        echo "ERROR FOUND on attempt #$attempt!"
        echo "---------------------------------------------"
        echo "Test output containing the error:"
        echo "$output" | grep -B 5 -A 5 "Permission denied (os error 13)"
        echo "---------------------------------------------"
        echo "Full test output has been saved to error_log.txt"
        echo "$output" > error_log.txt
        break
    else
        echo "Error not found on this attempt."
    fi
    
    # Increment the counter
    ((attempt++))
    
    # Optional: add a small delay between runs
    sleep 1
done 