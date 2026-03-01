# Crawler Improvements Plan

- [x] Investigate why URLs are stripped from the payload and fix it.
- [x] Ensure `repo_name` and `category` are properly attached to the stored JSON records.
- [x] Review directories structure for `DataStorageRepository` (e.g. `vector_databases/knowledge_base.jsonl` vs a flat structure).
- [x] Fix the `GitHubRecord` entity to reflect the actual stored fields instead of using `dict[str, Any]`.
- [x] Remove usages of `Any` where possible and fix typing issues caught by Pyre (ty).
