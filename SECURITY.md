# Security policy

## Reporting a vulnerability

If you discover a security vulnerability in RusTUI, please report it
responsibly:

1. **Do not** open a public GitHub issue.
2. Email the maintainer at **roliumgens@gmail.com** with a description and
   reproduction steps.
3. You should receive an acknowledgment within 72 hours.

## Scope

RusTUI is a terminal UI library. Security issues we care about:

- Input parsing bugs that could cause panics or memory-safety issues with
  malformed terminal input.
- Any `unsafe` code introduced (the crate is `#![forbid(unsafe_code)]`).
- Backend bugs that could leave the terminal in an unusable state.

Out of scope:

- Visual bugs, layout issues, or cosmetic problems (file a regular issue).
- Bugs in dependencies (report upstream).

## Disclosure

Once a fix is released, we will publish a GitHub Security Advisory and credit
the reporter (unless they prefer to remain anonymous).
