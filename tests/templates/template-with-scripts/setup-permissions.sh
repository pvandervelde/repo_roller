#!/bin/bash
# Setup executable permissions

cd "$(dirname "$0")"

# Make scripts executable
chmod +x script.sh
chmod +x script.py

# Verify permissions
echo "Permissions set:"
ls -l script.sh script.py regular-file.txt

echo "✓ Executable permissions configured"
