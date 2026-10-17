# Security policy

Jixel decodes files that may come from anywhere, so we take parsing bugs
seriously.

## Reporting a vulnerability

**Please do not open a public issue.** Use GitHub's private
[security advisory](https://github.com/lofiorin/jixel/security/advisories/new)
form. Include the affected version, platform, steps to reproduce and, if
possible, the file that triggers the problem.

You will get an answer within 7 days. Once fixed, we will publish an advisory
and credit you unless you prefer otherwise.

## Supported versions

Only the latest release receives security fixes.

## Scope notes

- **libjxl** vulnerabilities should also be reported upstream at
  <https://github.com/libjxl/libjxl/security>. Jixel pins a libjxl version in
  `core/crates/jixel-libjxl-sys/libjxl` and updates it for security releases.
- Jixel performs no network requests and has no server component.
- The background-removal model is embedded at build time and verified by its
  SHA-256 in `models/README.md`.
