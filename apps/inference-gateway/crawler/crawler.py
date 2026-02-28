import csv
import json
import os

import requests

GITHUB_RATES_MESSAGE = "Rate limited by GitHub API. Using fallback mock data."


def fetch_issue_pr_pairs(repo="rust-lang/cargo", limit=50):
    url = f"https://api.github.com/repos/{repo}/pulls?state=closed&per_page={limit}"
    headers = {"Accept": "application/vnd.github.v3+json"}

    # Optional token
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"

    try:
        response = requests.get(url, headers=headers, timeout=10)
        if response.status_code != 200:
            print(
                f"Warning: API failed with {response.status_code}. "
                f"{GITHUB_RATES_MESSAGE}"
            )
            return generate_mock_data()

        prs = response.json()
        pairs = []
        for pr in prs:
            if not pr.get("merged_at"):
                continue  # Skip unmerged ones

            title = pr.get("title", "")
            body = pr.get("body") or ""

            # Use PR title/body as the problem/solution pair for the sake of demo
            # In a real scenario, we would parse "Fixes #123" and fetch the issue.
            problem = f"{title}\n{body[:200]}"
            solution = pr.get("html_url", "")

            pairs.append(
                {
                    "id": str(pr["id"]),
                    "problem": problem,
                    "solution": solution,
                    "score": pr.get("comments", 0) + pr.get("review_comments", 0),
                }
            )

        if not pairs:
            return generate_mock_data()
        return pairs
    except Exception as e:
        print(f"Error fetching from GitHub: {e}. {GITHUB_RATES_MESSAGE}")
        return generate_mock_data()


def generate_mock_data():
    return [
        {
            "id": "1",
            "problem": "App crashes on startup when config is missing",
            "solution": "Add default config fallback in main.rs",
            "score": 5,
        },
        {
            "id": "2",
            "problem": "Memory leak in connection pool",
            "solution": "Ensure connections are dropped when scope ends",
            "score": 12,
        },
        {
            "id": "3",
            "problem": "Typo in README.md",
            "solution": "Fix spelling of 'architecture'",
            "score": 1,
        },
        {
            "id": "4",
            "problem": "Slow query on users endpoint",
            "solution": "Add index on email column in database",
            "score": 8,
        },
        {
            "id": "5",
            "problem": "Uncaught exception in parser",
            "solution": "Add try-catch block around JSON.parse",
            "score": 3,
        },
    ]


def main():
    print("Fetching data from GitHub...")
    pairs = fetch_issue_pr_pairs()

    kb_path = "knowledge_base.jsonl"
    features_path = "ranking_features.csv"

    print(f"Saving {len(pairs)} records to {kb_path} and {features_path}")

    with (
        open(kb_path, "w", encoding="utf-8") as f_kb,
        open(features_path, "w", encoding="utf-8", newline="") as f_feat,
    ):
        writer = csv.writer(f_feat)
        writer.writerow(["id", "score"])  # Ranking features

        for pair in pairs:
            # write jsonl
            f_kb.write(
                json.dumps(
                    {
                        "id": pair["id"],
                        "problem": pair["problem"],
                        "solution": pair["solution"],
                    }
                )
                + "\n"
            )

            # write csv
            writer.writerow([pair["id"], pair["score"]])

    print("Done!")


if __name__ == "__main__":
    main()
