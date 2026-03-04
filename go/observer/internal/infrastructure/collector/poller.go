package collector

import (
	"context"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// Poller periodically calls FetchAll on a MetricRepository and sends
// results to an output channel. It stops when ctx is cancelled.
type Poller struct {
	repo     repository.MetricRepository
	interval time.Duration
}

// NewPoller creates a Poller with the given repository and poll interval.
func NewPoller(repo repository.MetricRepository, interval time.Duration) *Poller {
	return &Poller{repo: repo, interval: interval}
}

// Run starts polling and returns a read-only channel of metrics snapshots.
// Each tick delivers a slice of all process metrics.
// The channel is closed when ctx is cancelled.
func (p *Poller) Run(ctx context.Context) <-chan []entity.Metric {
	ch := make(chan []entity.Metric, 4)
	go func() {
		defer close(ch)
		ticker := time.NewTicker(p.interval)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				metrics, err := p.repo.FetchAll(ctx)
				if err != nil {
					continue
				}
				select {
				case ch <- metrics:
				case <-ctx.Done():
					return
				}
			}
		}
	}()
	return ch
}
