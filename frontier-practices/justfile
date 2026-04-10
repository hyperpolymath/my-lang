# 7-tentacles - JavaScript/TypeScript Development Tasks
set shell := ["bash", "-uc"]
set dotenv-load := true

project := "7-tentacles"

# Show all recipes
default:
    @just --list --unsorted

# Install dependencies
install:
    npm install

# Build
build:
    npm run build

# Run tests
test:
    npm test

# Lint
lint:
    npm run lint

# Format (if prettier configured)
fmt:
    npm run format || npx prettier --write .

# Clean
clean:
    rm -rf node_modules dist build .next

# Start dev server
dev:
    npm run dev

# Type check (if TypeScript)
typecheck:
    npm run typecheck || npx tsc --noEmit

# All checks before commit
pre-commit: lint test
    @echo "All checks passed!"
