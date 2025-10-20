# git-rs

A minimal Git implementation written in Rust from scratch. Implements core Git commands with full compatibility to standard Git repositories.

## Features

- **`init`** - Initialize a new Git repository
- **`hash-object`** - Hash files and store as Git objects
- **`cat-file`** - Display Git object contents
- **`ls-tree`** - List tree object contents
- **`write-tree`** - Create tree objects from working directory
- **`commit-tree`** - Create commit objects
- **`commit`** - High-level commit with automatic tree creation

## Usage

### Initialize Repository
```bash
# Create a new Git repository in current directory
cargo run -- init
```

### Hash Objects
```bash
# Hash a file (displays hash without storing)
cargo run -- hash-object README.md

# Hash and store a file in Git database
cargo run -- hash-object -w README.md
# Returns: e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
```

### Inspect Objects
```bash
# Display contents of any Git object (blob, tree, or commit)
cargo run -- cat-file -p e69de29bb2d1d6434b8b29ae775ad8c2e48c5391

# Example output for a blob:
# Hello, world!

# Example output for a commit:
# tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904
# author John Doe <john@example.com> 1698765432 +0000
# committer John Doe <john@example.com> 1698765432 +0000
# 
# Initial commit
```

### Work with Trees
```bash
# List all files in a tree (shows modes, types, hashes, and names)
cargo run -- ls-tree <tree-hash>

# Show only filenames
cargo run -- ls-tree --name-only <tree-hash>

# Create tree from current working directory
cargo run -- write-tree
# Returns: 4b825dc642cb6eb9a060e54bf8d69288fbee4904
```

### Create Commits
```bash
# Low-level: create commit with specific tree and parent
cargo run -- commit-tree -m "Initial commit" <tree-hash>
cargo run -- commit-tree -m "Second commit" -p <parent-hash> <tree-hash>

# High-level: create commit automatically (recommended)
cargo run -- commit -m "Add new feature"
# Automatically creates tree from working directory and manages HEAD
```

### Example Workflow
```bash
# 1. Initialize repository
cargo run -- init

# 2. Add some files to your working directory
echo "Hello, Git!" > hello.txt

# 3. Create a commit
cargo run -- commit -m "Initial commit with hello.txt"

# 4. Inspect the commit
cargo run -- cat-file -p HEAD  # If HEAD exists, or use the commit hash
```

## Implementation

Built with Rust using Git's exact object format specification:
- SHA-1 hashing for content addressing
- Zlib compression for object storage
- Proper `.git` directory structure
- Full compatibility with standard Git

## Building

```bash
# Development
cargo build
cargo run -- <command>

# Release
cargo build --release
```
