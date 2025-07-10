#!/bin/bash

# RepoRoller Test Repository Creator
# Creates GitHub test template repositories for RepoRoller integration tests

set -euo pipefail

# Default values
ORGANIZATION="${1:-pvandervelde}"
FORCE="${2:-false}"

# Template repositories to create
declare -a TEMPLATES=(
    "test-basic:Basic repository template for RepoRoller integration tests:tests/templates/test-basic"
    "test-variables:Variable substitution template for RepoRoller integration tests:tests/templates/test-variables"
    "test-filtering:File filtering template for RepoRoller integration tests:tests/templates/test-filtering"
    "test-invalid:Error handling template for RepoRoller integration tests:tests/templates/test-invalid"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

log_header() {
    echo -e "${MAGENTA}$1${NC}"
}

log_action() {
    echo -e "${CYAN}$1${NC}"
}

# Check if GitHub CLI is installed and authenticated
check_github_cli() {
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) is not installed. Please install it from https://cli.github.com/"
        exit 1
    fi
    log_info "GitHub CLI is available"

    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI is not authenticated. Please run 'gh auth login' first."
        exit 1
    fi
    log_info "GitHub CLI is authenticated"
}

# Validate template directory
validate_template_directory() {
    local path="$1"

    if [[ ! -d "$path" ]]; then
        log_error "Template directory not found: $path"
        return 1
    fi

    local file_count=$(find "$path" -type f | wc -l)
    if [[ $file_count -eq 0 ]]; then
        log_error "Template directory is empty: $path"
        return 1
    fi

    log_info "Template directory validated: $path ($file_count files)"
    return 0
}

# Check if repository exists
repository_exists() {
    local org="$1"
    local name="$2"

    gh repo view "$org/$name" &> /dev/null
}

# Remove repository
remove_repository() {
    local org="$1"
    local name="$2"

    log_warn "Removing existing repository: $org/$name"
    if gh repo delete "$org/$name" --yes; then
        log_info "Repository removed: $org/$name"
    else
        log_error "Failed to remove repository: $org/$name"
        return 1
    fi
}

# Create repository
create_repository() {
    local org="$1"
    local name="$2"
    local description="$3"
    local template_path="$4"

    log_action "Creating repository: $org/$name"

    # Create temporary directory for git operations
    local temp_dir=$(mktemp -d -t "repo-roller-$name-XXXXXX")

    # Cleanup function
    cleanup() {
        rm -rf "$temp_dir"
    }
    trap cleanup EXIT

    # Copy template files to temp directory
    cp -r "$template_path"/* "$temp_dir/"

    # Initialize git repository
    cd "$temp_dir"
    git init
    git add .
    git commit -m "Initial commit: $description"

    # Create GitHub repository
    if gh repo create "$org/$name" --public --description "$description" --template; then
        # Push to GitHub
        git remote add origin "https://github.com/$org/$name.git"
        git branch -M main
        git push -u origin main

        log_info "Repository created: $org/$name"
    else
        log_error "Failed to create repository: $org/$name"
        return 1
    fi
}

# Show usage
show_usage() {
    echo "Usage: $0 [ORGANIZATION] [FORCE]"
    echo ""
    echo "Arguments:"
    echo "  ORGANIZATION  GitHub organization (default: pvandervelde)"
    echo "  FORCE         Set to 'true' to force recreation (default: false)"
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "  $0 myorg"
    echo "  $0 myorg true"
}

# Main execution
main() {
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        show_usage
        exit 0
    fi

    log_header "RepoRoller Test Repository Creator"
    log_header "================================="

    # Validate prerequisites
    check_github_cli

    # Validate template directories
    local all_valid=true
    for template_info in "${TEMPLATES[@]}"; do
        IFS=':' read -r name description path <<< "$template_info"
        if ! validate_template_directory "$path"; then
            all_valid=false
        fi
    done

    if [[ "$all_valid" != "true" ]]; then
        log_error "One or more template directories are invalid. Aborting."
        exit 1
    fi

    # Process each template
    for template_info in "${TEMPLATES[@]}"; do
        IFS=':' read -r name description path <<< "$template_info"

        if repository_exists "$ORGANIZATION" "$name"; then
            if [[ "$FORCE" == "true" ]]; then
                remove_repository "$ORGANIZATION" "$name"
                sleep 2  # Give GitHub time to process deletion
            else
                log_warn "Repository already exists: $ORGANIZATION/$name (use 'true' as second argument to recreate)"
                continue
            fi
        fi

        create_repository "$ORGANIZATION" "$name" "$description" "$path"
    done

    echo ""
    log_info "Test repository creation completed!"
    log_action "The following repositories are now available:"
    for template_info in "${TEMPLATES[@]}"; do
        IFS=':' read -r name description path <<< "$template_info"
        echo -e "  - ${WHITE}https://github.com/$ORGANIZATION/$name${NC}"
    done
}

# Run main function
main "$@"
