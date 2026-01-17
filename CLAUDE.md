# Claude Code Instructions

## Release

When asked to release a new version:

```bash
./scripts/bump-version.sh <version>
git add -A
git commit -m "Release v<version>"
git tag v<version>
git push && git push --tags
```
