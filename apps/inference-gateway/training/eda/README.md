# Exploratory Data Analysis (EDA) Naming Conventions

This directory contains scripts and reports for performing Exploratory Data Analysis (EDA) on the crawled dataset before feeding it into the ML models. 

Because we collect new snapshots of data over time and may experiment with different NLP filtering techniques or target specific domains, we organize our EDA runs into uniquely named date-stamped directories.

## Directory Naming Rule

Directories under `apps/inference-gateway/training/eda/` MUST follow this naming convention:

`<YYYY-MM-DD>-<topic-or-dataset-description>`

### Examples:
- `2026-03-01-ml-knowledge-base`: EDA focused specifically on filtering and extracting ML-domain keywords from the crawler knowledge base payload.
- `2026-03-15-full-corpus-baseline`: A general EDA processing the entire text without strict ML NLP filters.
- `2026-04-01-closed-issues-only`: EDA strictly analyzing issues that have been marked as closed/resolved.

## Usage
Each subdirectory should ideally be self-contained:
1. `main.py`: The script to generate the statistics and graphs.
2. `img/`: Subdirectory for the generated graphs/wordclouds.
3. `README.md`: The output report summarizing the findings for the user.

*Note: The `eda` directory is explicitly excluded from CI linting (e.g., in `.github/workflows/ci.yml`) because these are experimental scratch scripts that do not necessarily adhere to the strict production linting rules.*
