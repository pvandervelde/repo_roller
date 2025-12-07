#!/bin/bash
# Setup symbolic links for testing

cd "$(dirname "$0")"

# Create symlink to file
ln -sf real-file.txt link-to-file.txt

# Create directory and symlink to it
mkdir -p actual-dir
echo "Content in actual directory" > actual-dir/content.txt
ln -sf actual-dir link-to-dir

echo "✓ Symlinks created successfully"
