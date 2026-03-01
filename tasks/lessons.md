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
## 3. Exploratory Data Analysis (EDA)
- **Avoid Genericness**: When performing NLP or ML EDA, strictly filter out generic library names, project template words ("issue", "bug", "reproduce"), and common programming keywords ("def", "return"). The analysis must reflect the specific domain (e.g., using SpaCy to extract only nouns/adjectives).
- **Naming Conventions**: EDA script directories should never have literal placeholder names like `<YYYY-MM-DD>-title`. They must be explicitly descriptive, like `2026-03-01-ml-knowledge-base`.
- **Exclude from CI**: Experimental EDA scripts and notebooks must be explicitly excluded from CI formatting/linting (e.g., `lefthook` or Github Actions).
