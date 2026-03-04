package repository

import (
	"context"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
)

// SortKey specifies how to sort metrics.
type SortKey int

const (
	SortByPID SortKey = iota
	SortByCPU
	SortByMem
)

// MetricRepository is the abstraction for fetching process metrics.
// Implementations may use procfs polling or eBPF event streaming.
type MetricRepository interface {
	// FetchAll returns metrics for all visible processes.
	FetchAll(ctx context.Context) ([]entity.Metric, error)

	// FetchSorted returns metrics sorted by the given key.
	FetchSorted(ctx context.Context, by SortKey) ([]entity.Metric, error)

	// Stream sends metrics continuously until ctx is cancelled.
	Stream(ctx context.Context) (<-chan entity.Metric, error)
}

// EventPublisher delivers domain events to interested consumers.
type EventPublisher interface {
	Publish(ctx context.Context, e entity.Event) error
}
