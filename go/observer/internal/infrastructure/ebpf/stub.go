//go:build !linux || noebpf

// Package ebpf provides a compile-time check that *Repository satisfies
// repository.MetricRepository. This file is compiled when eBPF is not
// available (e.g. non-Linux CI runners or when noebpf tag is set).
package ebpf

import (
	"context"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// stubRepository is a no-op implementation used only in tests/non-Linux builds
// to verify that the interface is satisfied structurally.
type stubRepository struct{}

func (s *stubRepository) FetchAll(_ context.Context) ([]entity.Metric, error) {
	return nil, nil
}

func (s *stubRepository) FetchSorted(_ context.Context, _ repository.SortKey) ([]entity.Metric, error) {
	return nil, nil
}

func (s *stubRepository) Stream(_ context.Context) (<-chan entity.Metric, error) {
	ch := make(chan entity.Metric)
	close(ch)
	return ch, nil
}

// Compile-time interface assertion.
var _ repository.MetricRepository = (*stubRepository)(nil)
