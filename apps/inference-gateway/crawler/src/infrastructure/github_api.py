import requests

from domain.entity import GitHubRecord
from domain.repository import GithubRepository


class GithubApiRepository(GithubRepository):
    def __init__(self, base_url: str = "https://api.github.com"):
        self.base_url = base_url

    def fetch_closed_issues(
        self, repo: str, category: str, limit: int, token: str | None
    ) -> tuple[list[GitHubRecord], str]:
        # GitHub API Terms of Service Compliance:
        # This script interacts strictly with the official GitHub REST API
        # (api.github.com) using standard authenticated requests.
        # It is completely compliant with GitHub's ToS
        # Reference: https://docs.github.com/en/site-policy/github-terms/github-terms-of-service

        url = f"{self.base_url}/repos/{repo}/issues?state=closed&per_page={limit}"
        headers = {"Accept": "application/vnd.github.v3+json"}

        if token:
            headers["Authorization"] = f"Bearer {token}"

        response = requests.get(url, headers=headers, timeout=10)

        if response.status_code != 200:
            limit_rem = response.headers.get("X-RateLimit-Remaining", "Unknown")
            raise Exception(
                f"GitHub API failed for {repo} with {response.status_code}. "
                f"Rate limit remaining: {limit_rem}. Message: {response.text}"
            )

        payloads = response.json()
        records = [
            GitHubRecord(payload=p, repo_name=repo, category=category) for p in payloads
        ]
        rate_limit = response.headers.get("X-RateLimit-Remaining", "Unknown")

        return records, rate_limit
