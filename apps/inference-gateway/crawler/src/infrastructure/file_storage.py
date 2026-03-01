import json
import os
from datetime import UTC, datetime

from domain.entity import ExtractionStats, GitHubRecord
from domain.repository import DataStorageRepository


class FileStorageRepository(DataStorageRepository):
    def __init__(
        self,
        base_dir: str = "data",
        timestamp: str | None = None,
    ):
        self.timestamp = timestamp or datetime.now(UTC).strftime("%Y-%m-%d-%H-%M")
        self.base_dir = base_dir
        self.run_dir = os.path.join(self.base_dir, self.timestamp)
        self.stats_path = os.path.join(self.run_dir, "crawler_stats.md")

    def clear_raw_storage(self) -> None:
        # Ensure directory exists
        os.makedirs(self.run_dir, exist_ok=True)

    def save_raw_record(self, record: GitHubRecord) -> None:
        kb_path = os.path.join(self.run_dir, "knowledge_base.jsonl")

        with open(kb_path, "a", encoding="utf-8") as f:
            dump_data = {
                "repo_name": record.repo_name,
                "category": record.category,
                **record.payload,
            }
            f.write(json.dumps(dump_data) + "\n")

    def save_stats(self, stats: ExtractionStats) -> None:
        with open(self.stats_path, "w", encoding="utf-8") as f:
            f.write("# GitHub Crawler Statistics\n\n")
            f.write(f"**Last Run:** {stats.timestamp}\n")
            f.write(f"**Total Records Fetched:** {stats.total_fetched}\n")
            f.write(
                f"**Average Comments per Record:** {stats.average_comments:.2f}\n\n"
            )

            f.write("## Repository Breakdown\n")
            f.write("| Category | Repository | Records Fetched | Total Comments |\n")
            f_stats_table = "|---|---|---|---|\n"
            for brk in stats.repo_breakdown:
                f_stats_table += (
                    f"| {brk.category} | {brk.repo_name} | {brk.total_items} "
                    f"| {brk.total_comments} |\n"
                )
            f.write(f_stats_table)

            f.write("\n## Errors\n")
            if not stats.errors:
                f.write("エラーはありませんでした。\n")
            else:
                for error in stats.errors:
                    f.write(f"- {error}\n")

            f.write(
                "\n> Note: Raw JSON payloads are stored "
                "in `knowledge_base.jsonl` (git-ignored).\n"
            )
