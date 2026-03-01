# Lessons Learned

*This file will be updated with patterns and rules from any corrections received during implementation.*

## 🚨 MANDATORY AT THE START OF EVERY SESSION 🚨
**EVERY sub-agent or new session MUST read this `tasks/lessons.md` file FIRST before taking any action.**

## 1. Test-First Development (TDD)
- **NEVER** modify implementation code (`src/`) before writing a failing test.
- The workflow must strictly be: Write Test -> Watch it Fail (Red) -> Write Implementation (Green) -> Refactor.
- Modifying implementation code without a failing test is considered a critical failure ("論外").

## 2. Test Directory Structure
- The `tests/` directory must strictly mirror the `src/` directory structure.
- For example, if the implementation is `src/usecase/extract_data.py`, the corresponding test file must be `tests/usecase/test_extract_data.py`.
- Do not dump all tests into the root `tests/` directory (e.g., `tests/test_usecase.py` is unacceptable).
