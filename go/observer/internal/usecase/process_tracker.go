package usecase

import (
	"context"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// ProcessTracker watches for named processes appearing or disappearing.
// It compares successive snapshots and emits started/exited events.
type ProcessTracker struct {
	pub      repository.EventPublisher
	targets  map[string]struct{} // process names to watch; empty = watch all
	snapshot map[int]string      // pid -> name from the last tick
}

// NewProcessTracker creates a tracker that watches the given process names.
// Pass nil or empty to watch every process.
func NewProcessTracker(pub repository.EventPublisher, targets []string) *ProcessTracker {
	t := &ProcessTracker{
		pub:      pub,
		targets:  make(map[string]struct{}),
		snapshot: make(map[int]string),
	}
	for _, name := range targets {
		t.targets[name] = struct{}{}
	}
	return t
}

// Run consumes metric snapshots and emits lifecycle events.
func (t *ProcessTracker) Run(ctx context.Context, in <-chan []entity.Metric) {
	for {
		select {
		case <-ctx.Done():
			return
		case metrics, ok := <-in:
			if !ok {
				return
			}
			t.diff(ctx, metrics)
		}
	}
}

func (t *ProcessTracker) diff(ctx context.Context, metrics []entity.Metric) {
	current := make(map[int]string, len(metrics))
	for _, m := range metrics {
		if !t.isTarget(m.Name) {
			continue
		}
		current[m.PID] = m.Name
	}

	// Detect new processes
	for pid, name := range current {
		if _, existed := t.snapshot[pid]; !existed {
			_ = t.pub.Publish(ctx, entity.Event{
				Kind:    entity.EventProcessStarted,
				Process: entity.ProcessInfo{PID: pid, Name: name},
				At:      time.Now(),
			})
		}
	}

	// Detect exited processes
	for pid, name := range t.snapshot {
		if _, still := current[pid]; !still {
			_ = t.pub.Publish(ctx, entity.Event{
				Kind:    entity.EventProcessExited,
				Process: entity.ProcessInfo{PID: pid, Name: name},
				At:      time.Now(),
			})
		}
	}

	t.snapshot = current
}

func (t *ProcessTracker) isTarget(name string) bool {
	if len(t.targets) == 0 {
		return true
	}
	_, ok := t.targets[name]
	return ok
}
