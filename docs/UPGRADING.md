# Upgrading Guide

Versioning (vMAJOR.MINOR.PATCH) follows semantic rules for compatibility:
- PATCH
  - **Full backwards compatibility**
  - Bug fixes and UI/CLI improvements
  - No API or conf.yml schema changes
- MINOR
  - **Partial backwards compatibility**
  - API schema changes only
  - conf.yml remains compatible
- MAJOR
  - **No backwards compatibility**
  - conf.yml schema changes.
  - Old configs won’t work without manual/guided updates.
