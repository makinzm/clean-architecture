from infrastructure.github_api import GithubApiRepository


def test_github_api_fetch_closed_issues(mocker):
    # Arrange
    mock_response = mocker.MagicMock()
    mock_response.status_code = 200
    mock_response.json.return_value = [{"id": 1, "comments": 5}]
    mock_response.headers = {"X-RateLimit-Remaining": "4999"}

    mocker.patch("requests.get", return_value=mock_response)
    repo = GithubApiRepository()

    # Act
    records, rate_limit = repo.fetch_closed_issues(
        "test/repo", "test_category", limit=1, token="token"
    )

    # Assert
    assert len(records) == 1
    assert records[0].category == "test_category"
    assert rate_limit == "4999"
