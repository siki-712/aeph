#!/bin/bash
set -e

if [ -z "$1" ]; then
  echo "Usage: ./scripts/bump-version.sh <version>"
  echo "Example: ./scripts/bump-version.sh 0.2.0"
  exit 1
fi

VERSION=$1

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update npm packages
for pkg in npm/aeph npm/aeph-darwin-arm64 npm/aeph-darwin-x64 npm/aeph-linux-x64; do
  sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$pkg/package.json"
done

# Update optionalDependencies versions in main package
sed -i '' "s/\"aeph-darwin-arm64\": \".*\"/\"aeph-darwin-arm64\": \"$VERSION\"/" npm/aeph/package.json
sed -i '' "s/\"aeph-darwin-x64\": \".*\"/\"aeph-darwin-x64\": \"$VERSION\"/" npm/aeph/package.json
sed -i '' "s/\"aeph-linux-x64\": \".*\"/\"aeph-linux-x64\": \"$VERSION\"/" npm/aeph/package.json

echo "Version bumped to $VERSION"
echo ""
echo "Next steps:"
echo "  git add -A"
echo "  git commit -m 'Release v$VERSION'"
echo "  git tag v$VERSION"
echo "  git push && git push --tags"
