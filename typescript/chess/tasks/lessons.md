# Lessons Learned

## 2026-03-05: E2E Playwright Testing Infinite Loop
**Pattern:**
When an E2E test fails to interact with an element (e.g., `click()` fails or expected class is not added), I kept trying to force the interaction using Playwright workarounds (`force: true`, `element.evaluate(el => el.click())`, adding explicit delays) without understanding why standard user simulation failed. This led to an infinite loop of trial-and-error that wasted time and frustrated the user.

**Rule to Avoid:**
- **Stop guessing and forcing:** If a standard Playwright interaction like `click()` fails or its side effects aren't seen, DO NOT immediately resort to `force: true` or programmatic JS evaluation. 
- **Investigate the root cause FIRST:** The failure usually indicates a real problem in the application (e.g., race condition in state initialization, CSS `pointer-events` or `z-index` blocking the element, or a detached DOM node). 
- **Three-strike rule:** If an E2E test fails to pass after 3 attempts with minor tweaks, completely STOP the automated test loop. Step back, re-evaluate the DOM setup, or ask the user to manually verify the UI state locally before writing more automated checks.
- **Manual verification first:** When E2E actions like clicking fail, it could be a bad Playwright selector/timing setup OR a fundamentally broken UI implementation. **Always ask the user to manually verify the UI in the browser first** to isolate whether the issue lies in the application code itself or the test code.
