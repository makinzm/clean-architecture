# GitHub Crawler - Usage Guide

This module is designed to collect GitHub Issue/PR data for machine learning training and analysis. It follows Clean Architecture principles and includes professional tooling for quality assurance.

## 🛡️ Crawling Policy & Compliance

This crawler is built to be a "Good Citizen" of the GitHub ecosystem:
- **API-First**: It uses the official GitHub REST API. We **never** scrape HTML pages.
- **ToS Compliant**: Our fetching logic resides within the bounds of [GitHub's Terms of Service](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service).
- **Rate Limit Aware**: The crawler monitors and reports remaining API quota after every repository fetch.
- **Identify via Token**: It requires a `GITHUB_TOKEN` to ensure all requests are authenticated and traceable.

## 🚀 How to Crawl

### 1. Prerequisites
Ensure you have `uv` installed. If not, install it via:
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### 2. Setup Environment
You **must** provide a GitHub Personal Access Token (PAT) with `repo` scope.
```bash
export GITHUB_TOKEN=your_token_here
```

### 3. Install Dependencies
```bash
cd apps/inference-gateway/crawler
uv sync
```

### 4. Execute the Crawler
Run the main script using `uv`:
```bash
uv run src/main.py
```
The data will be stored in `data/<timestamp>/knowledge_base.jsonl` (raw data) and `data/<timestamp>/repository_stats.json` (statistical summary).

## 🧪 Development & Quality Control

We use a suite of tools to maintain high code quality:

- **Type Checking**: `uv run ty check --extra-search-path src src`
- **Linting/Formatting**: `uv run ruff check src`
- **Unit Tests**: `export PYTHONPATH=src && uv run pytest`

Alternatively, use **Lefthook** to run all checks at once:
```bash
lefthook run pre-commit
```
