#!/bin/bash

# Phase 4: Validate
# Runs tests, type checking, linting
# Exits non-zero if any validation fails

set -e

PROJECT_ROOT="${1:-.}"
VALIDATION_LOG="${PROJECT_ROOT}/.validation-log.json"

echo "📋 Validation Starting..."

TESTS_PASSED=0
TYPECHECK_PASSED=0
LINT_PASSED=0
BUILD_PASSED=0

# Detect project type
HAS_PACKAGE_JSON=$([ -f "${PROJECT_ROOT}/package.json" ] && echo 1 || echo 0)
HAS_PYPROJECT=$([ -f "${PROJECT_ROOT}/pyproject.toml" ] && echo 1 || echo 0)
HAS_PYTEST=$([ -f "${PROJECT_ROOT}/pytest.ini" ] && echo 1 || echo 0)

ERRORS=()

# Test: TS/JS
if [ $HAS_PACKAGE_JSON -eq 1 ]; then
  echo "🧪 Running npm tests..."
  if cd "$PROJECT_ROOT" && npm run test:ci 2>&1 | tee -a "$VALIDATION_LOG"; then
    TESTS_PASSED=1
    echo "✅ Tests passed"
  else
    TESTS_PASSED=0
    ERRORS+=("npm tests failed")
  fi
  
  # TypeCheck
  echo "🔍 Running TypeScript check..."
  if npx tsc --noEmit 2>&1 | tee -a "$VALIDATION_LOG"; then
    TYPECHECK_PASSED=1
    echo "✅ TypeScript OK"
  else
    TYPECHECK_PASSED=0
    ERRORS+=("TypeScript errors")
  fi
  
  # Lint
  echo "🧹 Running ESLint..."
  if npx eslint . --ext .ts,.tsx,.js,.jsx 2>&1 | tee -a "$VALIDATION_LOG"; then
    LINT_PASSED=1
    echo "✅ Linting OK"
  else
    LINT_PASSED=0
    ERRORS+=("Linting errors")
  fi
  
  # Build
  echo "🏗️ Building..."
  if npm run build 2>&1 | tee -a "$VALIDATION_LOG"; then
    BUILD_PASSED=1
    echo "✅ Build OK"
  else
    BUILD_PASSED=0
    ERRORS+=("Build failed")
  fi
fi

# Test: Python
if [ $HAS_PYPROJECT -eq 1 ] || [ $HAS_PYTEST -eq 1 ]; then
  echo "🧪 Running pytest..."
  if cd "$PROJECT_ROOT" && python -m pytest . 2>&1 | tee -a "$VALIDATION_LOG"; then
    TESTS_PASSED=1
    echo "✅ Pytest passed"
  else
    TESTS_PASSED=0
    ERRORS+=("pytest failed")
  fi
  
  # Lint Python
  echo "🧹 Running flake8..."
  if python -m flake8 . 2>&1 | tee -a "$VALIDATION_LOG"; then
    LINT_PASSED=1
    echo "✅ Python linting OK"
  else
    LINT_PASSED=0
    ERRORS+=("Python linting errors")
  fi
fi

# Summary
cat > "$VALIDATION_LOG" << JSON
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "tests_passed": $TESTS_PASSED,
  "typecheck_passed": $TYPECHECK_PASSED,
  "lint_passed": $LINT_PASSED,
  "build_passed": $BUILD_PASSED,
  "errors": $(printf '%s\n' "${ERRORS[@]}" | jq -Rs 'split("\n") | map(select(length > 0))')
}
JSON

echo ""
echo "✨ Validation Summary:"
echo "  Tests: $([ $TESTS_PASSED -eq 1 ] && echo '✅' || echo '❌')"
echo "  TypeCheck: $([ $TYPECHECK_PASSED -eq 1 ] && echo '✅' || echo '❌')"
echo "  Lint: $([ $LINT_PASSED -eq 1 ] && echo '✅' || echo '❌')"
echo "  Build: $([ $BUILD_PASSED -eq 1 ] && echo '✅' || echo '❌')"
echo "  Log: $VALIDATION_LOG"

if [ ${#ERRORS[@]} -gt 0 ]; then
  echo ""
  echo "❌ Validation failed. Rolling back..."
  git checkout -- .
  exit 1
fi

exit 0
