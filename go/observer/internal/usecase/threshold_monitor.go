package usecase

import (
	"context"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// ThresholdConfig holds the alerting thresholds.
type ThresholdConfig struct {
	CPUPercent float64 // alert when a process exceeds this CPU %
	MemBytes   uint64  // alert when a process exceeds this memory (bytes)
}

// ThresholdMonitor reads from a metrics channel and publishes
// EventThresholdBreached events when configured limits are exceeded.
type ThresholdMonitor struct {
	pub    repository.EventPublisher
	config ThresholdConfig
}

// NewThresholdMonitor creates a ThresholdMonitor.
func NewThresholdMonitor(pub repository.EventPublisher, cfg ThresholdConfig) *ThresholdMonitor {
	return &ThresholdMonitor{pub: pub, config: cfg}
}

// Run consumes metrics from in and publishes events. Stops when ctx is done or in is closed.
func (m *ThresholdMonitor) Run(ctx context.Context, in <-chan []entity.Metric) {
	for {
		select {
		case <-ctx.Done():
			return
		case metrics, ok := <-in:
			if !ok {
				return
			}
			for _, metric := range metrics {
				m.evaluate(ctx, metric)
			}
		}
	}
}

func (m *ThresholdMonitor) evaluate(ctx context.Context, metric entity.Metric) {
	if (m.config.CPUPercent > 0 && metric.CPUPercent > m.config.CPUPercent) ||
		(m.config.MemBytes > 0 && metric.MemBytes > m.config.MemBytes) {
		_ = m.pub.Publish(ctx, entity.Event{
			Kind:    entity.EventThresholdBreached,
			Process: entity.ProcessInfo{PID: metric.PID, Name: metric.Name},
			Metric:  &metric,
			At:      time.Now(),
		})
	}
}
