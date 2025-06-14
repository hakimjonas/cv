#!/bin/bash

# Kill any running blog_api_server processes
echo "Looking for blog_api_server processes..."

# Find processes by name
pids=$(pgrep -f blog_api_server)

if [ -z "$pids" ]; then
    echo "No blog_api_server processes found."
else
    echo "Found blog_api_server processes with PIDs: $pids"
    echo "Killing processes..."

    # Kill each process
    for pid in $pids; do
        kill -9 $pid
        echo "Killed process $pid"
    done

    echo "All blog_api_server processes killed."
fi
