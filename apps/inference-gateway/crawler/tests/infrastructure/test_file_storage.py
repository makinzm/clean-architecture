import os

from domain.entity import ExtractionStats, GitHubRecord, RepositoryStats
from infrastructure.file_storage import FileStorageRepository


def test_file_storage_repository_save(tmp_path):
    # Arrange
    timestamp = "2024-01-01-00-00"
    repo = FileStorageRepository(base_dir=str(tmp_path), timestamp=timestamp)
    kb_file = tmp_path / timestamp / "knowledge_base.jsonl"
    stats_file = tmp_path / timestamp / "crawler_stats.md"

    record = GitHubRecord(
        payload={"test": "data"}, repo_name="repo1", category="test_category"
    )

    # Act
    repo.clear_raw_storage()
    repo.save_raw_record(record)

    stats = ExtractionStats(
        timestamp=timestamp,
        total_fetched=10,
        average_comments=5.0,
        repo_breakdown=[
            RepositoryStats(
                repo_name="repo1",
                category="test_category",
                total_items=10,
                total_comments=50,
            )
        ],
    )
    repo.save_stats(stats)

    # Assert
    assert os.path.exists(kb_file)
    assert os.path.exists(stats_file)

    with open(kb_file) as f:
        content = f.read()
        assert '"repo_name": "repo1"' in content
        assert '"category": "test_category"' in content
        assert '"test": "data"' in content

    with open(stats_file) as f:
        content = f.read()
        assert "| test_category | repo1 | 10 | 50 |" in content
