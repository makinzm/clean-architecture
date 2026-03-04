package usecase_test

import (
	"context"
	"sync"
	"testing"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
	"makinzm/cleanarchitecture/gaze/internal/usecase"
)

// ---------------------------------------------------------------------------
// Mock EventPublisher
// ---------------------------------------------------------------------------

type mockPublisher struct {
	mu     sync.Mutex
	events []entity.Event
}

func (m *mockPublisher) Publish(_ context.Context, e entity.Event) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.events = append(m.events, e)
	return nil
}

func (m *mockPublisher) Events() []entity.Event {
	m.mu.Lock()
	defer m.mu.Unlock()
	out := make([]entity.Event, len(m.events))
	copy(out, m.events)
	return out
}

// ---------------------------------------------------------------------------
// Mock MetricRepository
// ---------------------------------------------------------------------------

type mockRepo struct {
	metrics []entity.Metric
}

func (r *mockRepo) FetchAll(_ context.Context) ([]entity.Metric, error) {
	return r.metrics, nil
}

func (r *mockRepo) FetchSorted(_ context.Context, by repository.SortKey) ([]entity.Metric, error) {
	cp := make([]entity.Metric, len(r.metrics))
	copy(cp, r.metrics)
	switch by {
	case repository.SortByCPU:
		for i := 0; i < len(cp)-1; i++ {
			for j := i + 1; j < len(cp); j++ {
				if cp[j].CPUPercent > cp[i].CPUPercent {
					cp[i], cp[j] = cp[j], cp[i]
				}
			}
		}
	case repository.SortByMem:
		for i := 0; i < len(cp)-1; i++ {
			for j := i + 1; j < len(cp); j++ {
				if cp[j].MemBytes > cp[i].MemBytes {
					cp[i], cp[j] = cp[j], cp[i]
				}
			}
		}
	}
	return cp, nil
}

func (r *mockRepo) Stream(_ context.Context) (<-chan entity.Metric, error) {
	ch := make(chan entity.Metric)
	close(ch)
	return ch, nil
}

// ---------------------------------------------------------------------------
// ThresholdMonitor tests
// ---------------------------------------------------------------------------

func feedChannel(metrics []entity.Metric) <-chan []entity.Metric {
	ch := make(chan []entity.Metric, 1)
	ch <- metrics
	close(ch)
	return ch
}

func TestThresholdMonitor_CPUBreached(t *testing.T) {
	pub := &mockPublisher{}
	monitor := usecase.NewThresholdMonitor(pub, usecase.ThresholdConfig{CPUPercent: 50})

	metrics := []entity.Metric{
		{PID: 1, Name: "heavy", CPUPercent: 80, Timestamp: time.Now()},
		{PID: 2, Name: "light", CPUPercent: 10, Timestamp: time.Now()},
	}
	monitor.Run(context.Background(), feedChannel(metrics))

	events := pub.Events()
	if len(events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(events))
	}
	if events[0].Kind != entity.EventThresholdBreached {
		t.Errorf("event kind: got %q, want %q", events[0].Kind, entity.EventThresholdBreached)
	}
	if events[0].Process.Name != "heavy" {
		t.Errorf("process name: got %q, want heavy", events[0].Process.Name)
	}
}

func TestThresholdMonitor_MemBreached(t *testing.T) {
	pub := &mockPublisher{}
	monitor := usecase.NewThresholdMonitor(pub, usecase.ThresholdConfig{MemBytes: 1024 * 1024 * 100}) // 100 MB

	metrics := []entity.Metric{
		{PID: 3, Name: "bloated", MemBytes: 1024 * 1024 * 200, Timestamp: time.Now()},
		{PID: 4, Name: "lean", MemBytes: 1024 * 1024 * 10, Timestamp: time.Now()},
	}
	monitor.Run(context.Background(), feedChannel(metrics))

	events := pub.Events()
	if len(events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(events))
	}
	if events[0].Process.Name != "bloated" {
		t.Errorf("expected bloated, got %q", events[0].Process.Name)
	}
}

func TestThresholdMonitor_NoBreach(t *testing.T) {
	pub := &mockPublisher{}
	monitor := usecase.NewThresholdMonitor(pub, usecase.ThresholdConfig{CPUPercent: 90})
	metrics := []entity.Metric{
		{PID: 5, Name: "ok", CPUPercent: 30, Timestamp: time.Now()},
	}
	monitor.Run(context.Background(), feedChannel(metrics))

	if n := len(pub.Events()); n != 0 {
		t.Errorf("expected 0 events, got %d", n)
	}
}

// ---------------------------------------------------------------------------
// ProcessTracker tests
// ---------------------------------------------------------------------------

func TestProcessTracker_DetectsStart(t *testing.T) {
	pub := &mockPublisher{}
	tracker := usecase.NewProcessTracker(pub, []string{"qdrant"})

	// First tick: qdrant appears
	tracker.Run(context.Background(), feedChannel([]entity.Metric{
		{PID: 100, Name: "qdrant"},
	}))

	events := pub.Events()
	if len(events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(events))
	}
	if events[0].Kind != entity.EventProcessStarted {
		t.Errorf("kind: got %q, want process_started", events[0].Kind)
	}
}

func TestProcessTracker_DetectsExit(t *testing.T) {
	pub := &mockPublisher{}
	tracker := usecase.NewProcessTracker(pub, []string{"ollama"})

	ch := make(chan []entity.Metric, 2)
	ch <- []entity.Metric{{PID: 200, Name: "ollama"}} // appears
	ch <- []entity.Metric{}                           // disappears
	close(ch)

	tracker.Run(context.Background(), ch)

	events := pub.Events()
	kinds := make(map[entity.EventKind]int)
	for _, e := range events {
		kinds[e.Kind]++
	}
	if kinds[entity.EventProcessStarted] != 1 {
		t.Errorf("expected 1 started event, got %d", kinds[entity.EventProcessStarted])
	}
	if kinds[entity.EventProcessExited] != 1 {
		t.Errorf("expected 1 exited event, got %d", kinds[entity.EventProcessExited])
	}
}

func TestProcessTracker_IgnoresNonTargets(t *testing.T) {
	pub := &mockPublisher{}
	tracker := usecase.NewProcessTracker(pub, []string{"qdrant"})

	tracker.Run(context.Background(), feedChannel([]entity.Metric{
		{PID: 300, Name: "bash"},
		{PID: 301, Name: "nginx"},
	}))

	if n := len(pub.Events()); n != 0 {
		t.Errorf("expected 0 events for non-targets, got %d", n)
	}
}

// ---------------------------------------------------------------------------
// SortedSnapshot tests
// ---------------------------------------------------------------------------

func TestSortedSnapshot_TopN_ByCPU(t *testing.T) {
	repo := &mockRepo{metrics: []entity.Metric{
		{PID: 1, CPUPercent: 10},
		{PID: 2, CPUPercent: 90},
		{PID: 3, CPUPercent: 50},
	}}
	ss := usecase.NewSortedSnapshot(repo)

	top2, err := ss.TopN(context.Background(), repository.SortByCPU, 2)
	if err != nil {
		t.Fatal(err)
	}
	if len(top2) != 2 {
		t.Fatalf("expected 2, got %d", len(top2))
	}
	if top2[0].PID != 2 {
		t.Errorf("first should be PID 2 (CPU 90%%), got PID %d", top2[0].PID)
	}
	if top2[1].PID != 3 {
		t.Errorf("second should be PID 3 (CPU 50%%), got PID %d", top2[1].PID)
	}
}

func TestSortedSnapshot_TopN_Zero(t *testing.T) {
	repo := &mockRepo{metrics: []entity.Metric{
		{PID: 1, MemBytes: 1000},
		{PID: 2, MemBytes: 2000},
	}}
	ss := usecase.NewSortedSnapshot(repo)

	all, err := ss.TopN(context.Background(), repository.SortByMem, 0)
	if err != nil {
		t.Fatal(err)
	}
	if len(all) != 2 {
		t.Errorf("n=0 should return all, got %d", len(all))
	}
}
