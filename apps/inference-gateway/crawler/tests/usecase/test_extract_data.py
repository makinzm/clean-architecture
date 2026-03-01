from unittest.mock import MagicMock

from domain.entity import GitHubRecord
from usecase.extract_data import ExtractDataUsecase


def test_extract_data_usecase_success(mocker):
    # Arrange
    mock_github = MagicMock()
    mock_storage = MagicMock()

    mock_github.fetch_closed_issues.return_value = (
        [
            GitHubRecord(
                payload={"id": 1, "comments": 5, "state_reason": "completed"},
                repo_name="repo1",
                category="test_category",
            )
        ],
        "5000",
    )

    usecase = ExtractDataUsecase(mock_github, mock_storage)
    repos_by_category = {"test_category": ["repo1"]}

    # Act
    usecase.execute(repos_by_category, limit=10, token="fake-token")

    # Assert
    mock_storage.clear_raw_storage.assert_called_once()
    mock_storage.save_raw_record.assert_called_once()
    mock_storage.save_stats.assert_called_once()

    stats = mock_storage.save_stats.call_args[0][0]
    assert stats.total_fetched == 1
    assert stats.average_comments == 5.0
    assert len(stats.repo_breakdown) == 1
    assert stats.repo_breakdown[0].repo_name == "repo1"
    assert stats.repo_breakdown[0].category == "test_category"
    assert len(stats.errors) == 0


def test_extract_data_usecase_continue_on_error(mocker):
    # Arrange
    mock_github = MagicMock()
    mock_storage = MagicMock()

    # Suppose repo1 fails, but repo2 succeeds
    def mock_fetch(repo, category, limit, token):
        if repo == "repo1":
            raise Exception("API error")
        return [
            GitHubRecord(
                payload={"id": 2, "comments": 3, "state_reason": "completed"},
                repo_name=repo,
                category=category,
            )
        ], "5000"

    mock_github.fetch_closed_issues.side_effect = mock_fetch

    usecase = ExtractDataUsecase(mock_github, mock_storage)

    # Act
    usecase.execute({"test_category": ["repo1", "repo2"]}, limit=10, token="fake-token")

    # Assert
    # Should not stop execution for repo1, but continue to repo2
    mock_storage.save_stats.assert_called_once()
    stats = mock_storage.save_stats.call_args[0][0]
    assert stats.total_fetched == 1
    assert stats.repo_breakdown[0].repo_name == "repo2"
    assert stats.repo_breakdown[0].category == "test_category"
    assert len(stats.errors) == 1
    assert "Error fetching repo1: API error" in stats.errors[0]


def test_clean_payload():
    mock_github = MagicMock()
    mock_storage = MagicMock()
    uc = ExtractDataUsecase(mock_github, mock_storage)

    sample_payload = {
        "id": 123,
        "node_id": "abc",
        "url": "https://api.github.com/foo",
        "html_url": "https://github.com/user/repo/pull/1",
        "comments_url": "https://api.github.com/foo/comments",
        "title": "Test PR",
        "body": "This is a test",
        "user": {
            "login": "testuser",
            "id": 456,
            "avatar_url": "https://test.com/avatar",
            "site_admin": False,
        },
        "pull_request": {
            "merged_at": "2023-01-01T00:00:00Z",
            "patch_url": "https://api.github.com/patch",
        },
        "nested": [{"id": 789, "name": "something"}],
    }

    cleaned = uc._clean_payload(sample_payload)

    assert "id" not in cleaned
    assert "node_id" not in cleaned
    assert "url" not in cleaned
    assert "html_url" in cleaned
    assert cleaned["html_url"] == "https://github.com/user/repo/pull/1"
    assert "comments_url" not in cleaned
    assert "title" in cleaned
    assert "body" in cleaned
    assert "user" in cleaned
    assert "login" in cleaned["user"]
    assert "id" not in cleaned["user"]
    assert "avatar_url" not in cleaned["user"]
    assert "site_admin" not in cleaned["user"]
    assert "pull_request" in cleaned
    assert "merged_at" in cleaned["pull_request"]
    assert "patch_url" not in cleaned["pull_request"]
    assert len(cleaned["nested"]) == 1
    assert "id" not in cleaned["nested"][0]
    assert "name" in cleaned["nested"][0]
