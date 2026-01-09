# Template Test - Internal Visibility

This template is configured with `default_visibility = "internal"` and is used to test:

- Internal visibility handling
- Enterprise environment detection
- Fallback behavior on non-Enterprise GitHub instances

**Note**: Internal visibility only works on GitHub Enterprise. On other GitHub instances,
the system should either reject the creation or fall back to private visibility depending
on the validation rules.
