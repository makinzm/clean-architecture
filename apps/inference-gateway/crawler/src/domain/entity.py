from dataclasses import dataclass, field

type JSONDict = dict[str, object]


@dataclass
class GitHubRecord:
    payload: JSONDict
    repo_name: str
    category: str


@dataclass
class RepositoryStats:
    repo_name: str
    category: str
    total_items: int = 0
    total_comments: int = 0


@dataclass
class ExtractionStats:
    timestamp: str
    total_fetched: int
    average_comments: float
    repo_breakdown: list[RepositoryStats] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
