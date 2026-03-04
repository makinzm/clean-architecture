package usecase

import (
	"context"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// SortedSnapshot fetches the top-N processes sorted by the given key.
type SortedSnapshot struct {
	repo repository.MetricRepository
}

// NewSortedSnapshot creates a SortedSnapshot usecase.
func NewSortedSnapshot(repo repository.MetricRepository) *SortedSnapshot {
	return &SortedSnapshot{repo: repo}
}

// TopN returns the top n processes sorted by key.
// If n <= 0 all processes are returned.
func (s *SortedSnapshot) TopN(ctx context.Context, by repository.SortKey, n int) ([]entity.Metric, error) {
	metrics, err := s.repo.FetchSorted(ctx, by)
	if err != nil {
		return nil, err
	}
	if n > 0 && n < len(metrics) {
		metrics = metrics[:n]
	}
	return metrics, nil
}
