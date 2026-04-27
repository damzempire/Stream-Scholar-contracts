#!/usr/bin/env bash
# check-doc-comments.sh
#
# Fails the CI pipeline if any public struct or function in the Soroban contracts
# is missing a /// doc-comment on the line immediately preceding it.
#
# Usage:
#   ./scripts/check-doc-comments.sh
#
# Exit codes:
#   0 – all public items are documented
#   1 – one or more public items are missing doc-comments

set -euo pipefail

CONTRACTS_DIR="contracts/scholar_contracts/src"
FAILED=0

check_file() {
    local file="$1"
    local prev_line=""
    local line_num=0

    while IFS= read -r line; do
        line_num=$((line_num + 1))

        # Match public functions and structs (not inside comments or strings)
        if echo "$line" | grep -qE '^\s+pub fn |^pub struct |^pub enum '; then
            # The previous non-empty line must be a doc-comment (///) or a derive/attribute
            if ! echo "$prev_line" | grep -qE '^\s*///|^\s*#\['; then
                echo "MISSING DOC-COMMENT: $file:$line_num"
                echo "  Line: $line"
                FAILED=1
            fi
        fi

        # Track previous non-empty line
        if [[ -n "${line// /}" ]]; then
            prev_line="$line"
        fi
    done < "$file"
}

echo "Checking doc-comments in $CONTRACTS_DIR..."

for rs_file in "$CONTRACTS_DIR"/*.rs; do
    # Skip test files
    if [[ "$rs_file" == *test* ]]; then
        continue
    fi
    check_file "$rs_file"
done

if [[ $FAILED -eq 1 ]]; then
    echo ""
    echo "ERROR: One or more public items are missing doc-comments."
    echo "Add a /// doc-comment above each public fn, struct, or enum."
    exit 1
else
    echo "All public items have doc-comments. ✓"
fi
