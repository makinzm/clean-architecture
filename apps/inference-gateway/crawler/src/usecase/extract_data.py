from datetime import datetime
from typing import Any, cast

from domain.entity import ExtractionStats, RepositoryStats
from domain.repository import DataStorageRepository, GithubRepository


class ExtractDataUsecase:
    def __init__(
        self, github_repo: GithubRepository, storage_repo: DataStorageRepository
    ):
        self.github_repo = github_repo
        self.storage_repo = storage_repo

    def _clean_payload(
        self, payload: dict[str, object] | list[object] | object
    ) -> dict[str, object] | list[object] | object:
        """
        Recursively clean up the payload by removing known noisy fields
        such as *_url and *_id (except core structural ones if needed).
        """
        if isinstance(payload, dict):
            cleaned = {}
            for k, v in payload.items():
                if not isinstance(k, str):
                    cleaned[k] = self._clean_payload(v)
                    continue
                # Remove irrelevant keys
                if (k.endswith("url") and k != "html_url") or k in {
                    "node_id",
                    "gravatar_id",
                    "id",
                    "site_admin",
                }:
                    continue
                cleaned[k] = self._clean_payload(v)
            return cleaned
        elif isinstance(payload, list):
            return [self._clean_payload(item) for item in payload]
        else:
            return payload

    def execute(
        self, repos_by_category: dict[str, list[str]], limit: int, token: str | None
    ) -> None:
        self.storage_repo.clear_raw_storage()

        total_fetched = 0
        total_comments = 0
        repo_breakdown = []
        errors = []

        for category, repos in repos_by_category.items():
            for repo in repos:
                print(f"Fetching from {repo} (Category: {category})...")
                try:
                    records, rate_limit = self.github_repo.fetch_closed_issues(
                        repo, category, limit, token
                    )
                    print(
                        f"  -> Fetched {len(records)} items. "
                        f"Rate limit remaining: {rate_limit}"
                    )

                    repo_fetched = 0
                    repo_comments = 0

                    for record in records:
                        # 1. Filter merged PRs and completed issues
                        is_pr = "pull_request" in record.payload
                        if is_pr:
                            # Must be merged
                            pr_data_raw = record.payload.get("pull_request")
                            if isinstance(pr_data_raw, dict):
                                pr_data = cast(dict[str, Any], pr_data_raw)
                                if not pr_data.get("merged_at"):
                                    continue
                            else:
                                continue
                        else:
                            # Issue must be completed
                            if record.payload.get("state_reason") != "completed":
                                continue

                        # 2. Clean payload noise (URLs, ids)
                        cleaned_payload = self._clean_payload(record.payload)
                        record.payload = cast(dict[str, Any], cleaned_payload)

                        self.storage_repo.save_raw_record(record)
                        repo_fetched += 1
                        total_fetched += 1

                        # Basic heuristic stats
                        comments_raw = record.payload.get("comments", 0)
                        comments = (
                            int(str(comments_raw)) if comments_raw is not None else 0
                        )
                        repo_comments += comments
                        total_comments += comments

                    repo_breakdown.append(
                        RepositoryStats(
                            repo_name=repo,
                            category=category,
                            total_items=repo_fetched,
                            total_comments=repo_comments,
                        )
                    )

                except Exception as e:
                    error_msg = f"Error fetching {repo}: {e}"
                    print(error_msg)
                    errors.append(error_msg)
                    continue

        stats = ExtractionStats(
            timestamp=datetime.now().isoformat(),
            total_fetched=total_fetched,
            average_comments=(float(total_comments) / float(total_fetched))
            if total_fetched > 0
            else 0.0,
            repo_breakdown=repo_breakdown,
            errors=errors,
        )

        self.storage_repo.save_stats(stats)
        print(f"Extraction complete! Total records: {total_fetched}")
