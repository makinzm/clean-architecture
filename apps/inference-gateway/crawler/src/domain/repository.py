from abc import ABC, abstractmethod

from domain.entity import ExtractionStats, GitHubRecord


class GithubRepository(ABC):
    @abstractmethod
    def fetch_closed_issues(
        self, repo: str, category: str, limit: int, token: str | None
    ) -> tuple[list[GitHubRecord], str]:
        """Fetch closed issues/PRs from a repository."""
        pass


class DataStorageRepository(ABC):
    @abstractmethod
    def save_raw_record(self, record: GitHubRecord) -> None:
        """Save a raw record to the data lake (JSONL)."""
        pass

    @abstractmethod
    def save_stats(self, stats: ExtractionStats) -> None:
        """Save extraction statistics to a file (Markdown)."""
        pass

    @abstractmethod
    def clear_raw_storage(self) -> None:
        """Clear existing raw storage."""
        pass
