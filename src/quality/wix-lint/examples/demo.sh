#!/bin/bash
#
# wix-lint Demo Script
# ====================
# This script demonstrates the capabilities of wix-lint,
# a linter for WiX (Windows Installer XML) files.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINT_DIR="$(dirname "$SCRIPT_DIR")"
WIX_DATA_DIR="$(dirname "$(dirname "$LINT_DIR")")/core/wix-data"

# Build the linter if needed
LINTER="$LINT_DIR/target/release/wix-lint"
if [ ! -f "$LINTER" ]; then
    echo -e "${YELLOW}Building wix-lint...${NC}"
    (cd "$LINT_DIR" && cargo build --release --quiet)
fi

# Header
echo ""
echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║                    wix-lint Demo                           ║${NC}"
echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to run a demo
run_demo() {
    local title="$1"
    local description="$2"
    shift 2

    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${GREEN}▶ $title${NC}"
    echo -e "${YELLOW}$description${NC}"
    echo ""
    echo -e "${CYAN}Command:${NC} $*"
    echo -e "${BLUE}────────────────────────────────────────────────────────────${NC}"
    echo ""

    # Run the command
    "$@" || true

    echo ""
    read -p "Press Enter to continue..."
    echo ""
}

# Demo 1: Basic linting
run_demo "Demo 1: Basic Linting" \
    "Lint a WiX file with common issues" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" "$SCRIPT_DIR/sample-bad.wxs"

# Demo 2: Valid file
run_demo "Demo 2: Linting a Valid File" \
    "A well-structured WiX file should have minimal issues" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" "$SCRIPT_DIR/sample-good.wxs"

# Demo 3: JSON output
run_demo "Demo 3: JSON Output Format" \
    "Machine-readable output for CI/CD integration" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --format json "$SCRIPT_DIR/sample-bad.wxs"

# Demo 4: SARIF output
run_demo "Demo 4: SARIF Output Format" \
    "Standard format for GitHub Actions, Azure DevOps, etc." \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --format sarif "$SCRIPT_DIR/sample-bad.wxs"

# Demo 5: Filter by severity
run_demo "Demo 5: Filter by Severity" \
    "Show only errors (hide warnings and info)" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --severity error "$SCRIPT_DIR/sample-bad.wxs"

# Demo 6: Ignore specific rules
run_demo "Demo 6: Ignore Specific Rules" \
    "Suppress specific warnings you don't care about" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" \
    --ignore package-requires-upgradecode \
    --ignore package-requires-manufacturer \
    "$SCRIPT_DIR/sample-bad.wxs"

# Demo 7: Statistics
run_demo "Demo 7: Show Statistics" \
    "Get a summary of all issues found" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --statistics "$SCRIPT_DIR/sample-bad.wxs"

# Demo 8: Multiple files
run_demo "Demo 8: Lint Multiple Files" \
    "Lint all WiX files in a directory" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --statistics "$SCRIPT_DIR"/*.wxs

# Demo 9: Inline disables
run_demo "Demo 9: Inline Disable Comments" \
    "Suppress warnings with inline comments" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" "$SCRIPT_DIR/sample-disable.wxs"

# Demo 10: Quiet mode
run_demo "Demo 10: Quiet Mode (Count Only)" \
    "Just show the error count for scripts" \
    "$LINTER" --wix-data "$WIX_DATA_DIR" --count "$SCRIPT_DIR/sample-bad.wxs"

# Summary
echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║                    Demo Complete!                          ║${NC}"
echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}wix-lint Features Demonstrated:${NC}"
echo "  - Basic file linting with error reporting"
echo "  - Multiple output formats (text, JSON, SARIF)"
echo "  - Severity filtering (error, warning, info)"
echo "  - Rule ignoring and selection"
echo "  - Statistics and summaries"
echo "  - Multiple file support"
echo "  - Inline disable comments"
echo ""
echo -e "${YELLOW}For more options, run:${NC} wix-lint --help"
echo ""
