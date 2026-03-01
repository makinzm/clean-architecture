import os
import sys

import yaml
from dotenv import load_dotenv

from infrastructure.file_storage import FileStorageRepository
from infrastructure.github_api import GithubApiRepository
from usecase.extract_data import ExtractDataUsecase


def main():
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        print("Error: GITHUB_TOKEN environment variable is missing.")
        sys.exit(1)

    config_path = os.path.join(os.path.dirname(__file__), "..", "config.yaml")
    try:
        with open(config_path, encoding="utf-8") as f:
            config = yaml.safe_load(f)
            repos_by_category = config.get("categories", {})
    except FileNotFoundError:
        print(f"Error: Could not find config file at {config_path}")
        sys.exit(1)
    except yaml.YAMLError as e:
        print(f"Error parsing config.yaml: {e}")
        sys.exit(1)

    github_repo = GithubApiRepository()
    storage_repo = FileStorageRepository()
    usecase = ExtractDataUsecase(github_repo, storage_repo)

    usecase.execute(repos_by_category=repos_by_category, limit=500, token=token)


if __name__ == "__main__":
    load_dotenv()
    main()
