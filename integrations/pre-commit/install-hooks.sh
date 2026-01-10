#!/bin/bash
# Install WixCraft Git hooks
# Usage: ./install-hooks.sh [git-repo-path]

set -e

REPO_PATH="${1:-.}"
HOOKS_DIR="$REPO_PATH/.git/hooks"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "Error: $HOOKS_DIR not found. Is this a git repository?"
    exit 1
fi

# Find WixCraft tools
find_tool() {
    local tool="$1"
    if command -v "$tool" &> /dev/null; then
        command -v "$tool"
    elif [ -f "./target/release/$tool" ]; then
        echo "./target/release/$tool"
    elif [ -f "$HOME/.wixcraft/bin/$tool" ]; then
        echo "$HOME/.wixcraft/bin/$tool"
    else
        echo ""
    fi
}

WIX_LINT=$(find_tool wix-lint)
WIX_FMT=$(find_tool wix-fmt)
WIX_SECURITY=$(find_tool wix-security)
WIX_UPGRADE=$(find_tool wix-upgrade)

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/bin/bash
# WixCraft pre-commit hook

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Find WiX files staged for commit
WXS_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(wxs|wxi)$' || true)

if [ -z "$WXS_FILES" ]; then
    exit 0
fi

echo "Running WixCraft checks on staged .wxs/.wxi files..."
echo ""

FAILED=0

# Run wix-lint if available
if command -v wix-lint &> /dev/null; then
    echo -e "${YELLOW}Running wix-lint...${NC}"
    for file in $WXS_FILES; do
        if ! wix-lint "$file" 2>/dev/null; then
            echo -e "${RED}Lint issues in: $file${NC}"
            FAILED=1
        fi
    done
else
    echo "wix-lint not found, skipping"
fi

# Run wix-fmt check if available
if command -v wix-fmt &> /dev/null; then
    echo -e "${YELLOW}Checking formatting...${NC}"
    for file in $WXS_FILES; do
        if ! wix-fmt --check "$file" 2>/dev/null; then
            echo -e "${RED}Format issues in: $file${NC}"
            echo "Run: wix-fmt --write $file"
            FAILED=1
        fi
    done
else
    echo "wix-fmt not found, skipping"
fi

# Run wix-security if available
if command -v wix-security &> /dev/null; then
    echo -e "${YELLOW}Running security scan...${NC}"
    for file in $WXS_FILES; do
        if ! wix-security scan --fail-on high "$file" 2>/dev/null; then
            echo -e "${RED}Security issues in: $file${NC}"
            FAILED=1
        fi
    done
else
    echo "wix-security not found, skipping"
fi

# Run wix-upgrade if available
if command -v wix-upgrade &> /dev/null; then
    echo -e "${YELLOW}Checking upgrade readiness...${NC}"
    for file in $WXS_FILES; do
        if ! wix-upgrade validate --fail-on error "$file" 2>/dev/null; then
            echo -e "${RED}Upgrade issues in: $file${NC}"
            FAILED=1
        fi
    done
else
    echo "wix-upgrade not found, skipping"
fi

echo ""

if [ $FAILED -eq 1 ]; then
    echo -e "${RED}Pre-commit checks failed. Fix issues before committing.${NC}"
    exit 1
else
    echo -e "${GREEN}All WixCraft checks passed.${NC}"
    exit 0
fi
HOOK

chmod +x "$HOOKS_DIR/pre-commit"

echo "WixCraft pre-commit hook installed to $HOOKS_DIR/pre-commit"
echo ""
echo "Available tools:"
[ -n "$WIX_LINT" ] && echo "  - wix-lint: $WIX_LINT" || echo "  - wix-lint: not found"
[ -n "$WIX_FMT" ] && echo "  - wix-fmt: $WIX_FMT" || echo "  - wix-fmt: not found"
[ -n "$WIX_SECURITY" ] && echo "  - wix-security: $WIX_SECURITY" || echo "  - wix-security: not found"
[ -n "$WIX_UPGRADE" ] && echo "  - wix-upgrade: $WIX_UPGRADE" || echo "  - wix-upgrade: not found"
