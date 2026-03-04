package usecase_test

import (
	"context"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/infrastructure/collector"
	"makinzm/cleanarchitecture/gaze/internal/infrastructure/procfs"
	"makinzm/cleanarchitecture/gaze/internal/usecase"
)

func TestIntegration_Pipeline(t *testing.T) {
	root := t.TempDir()

	// Helper to write proc state
	writeState := func(totalTicks uint64, pid int, utime uint64) {
		procStat := "cpu  " + strconv.FormatUint(totalTicks, 10) + " 0 0 0 0 0 0 0 0 0\n"
		os.WriteFile(filepath.Join(root, "stat"), []byte(procStat), 0644)

		pidDir := filepath.Join(root, strconv.Itoa(pid))
		os.MkdirAll(pidDir, 0755)
		statLine := strconv.Itoa(pid) + " (test) S 1 1 1 0 1 0 0 0 0 0 " + strconv.FormatUint(utime, 10) + " 0 0 0 20 0 1 0 0 1024000 100 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0\n"
		os.WriteFile(filepath.Join(pidDir, "stat"), []byte(statLine), 0644)
	}

	// 1. Initial State
	pid := 999
	writeState(100, pid, 10)

	repo := procfs.New(root)
	pub := &mockPublisher{}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// 2. Start Poller
	interval := 100 * time.Millisecond
	p := collector.NewPoller(repo, interval)
	in := p.Run(ctx)

	// Wrap channel to log values
	loggedIn := make(chan []entity.Metric, 4)
	go func() {
		defer close(loggedIn)
		for m := range in {
			for _, metric := range m {
				t.Logf("Poller Tick: PID=%d Name=%s CPU=%.2f%%", metric.PID, metric.Name, metric.CPUPercent)
			}
			loggedIn <- m
		}
	}()

	// 3. Start Threshold Monitor (> 50% CPU alert)
	monitor := usecase.NewThresholdMonitor(pub, usecase.ThresholdConfig{CPUPercent: 50})

	done := make(chan struct{})
	go func() {
		monitor.Run(ctx, loggedIn)
		close(done)
	}()

	// Wait for baseline poll
	time.Sleep(250 * time.Millisecond)

	// 4. Update state to trigger alert (delta 100 total, 60 utime -> 60%)
	t.Log("Advancing state to 60%% CPU...")
	writeState(200, pid, 70)

	// Wait for events
	var found bool
	deadline := time.After(3 * time.Second)
loop:
	for {
		select {
		case <-deadline:
			break loop
		case <-time.After(interval):
			events := pub.Events()
			for _, e := range events {
				if e.Kind == entity.EventThresholdBreached && e.Metric.CPUPercent >= 50 {
					found = true
					t.Logf("Event Found: Kind=%s CPU=%.2f%%", e.Kind, e.Metric.CPUPercent)
					break loop
				}
			}
		}
	}

	if !found {
		t.Errorf("expected threshold breached event with CPU >= 50%%, got %d events", len(pub.Events()))
	}

	cancel()
	<-done
}
