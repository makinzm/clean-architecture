package procfs

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

const (
	defaultProcRoot  = "/proc"
	clockTicksPerSec = 100 // USER_HZ, typically 100 on Linux
	pageSize         = 4096
)

// Repository implements repository.MetricRepository using /proc filesystem.
type Repository struct {
	procRoot string

	// CPU calculation state (two-sample delta)
	prevCPUTimes map[int][2]uint64 // pid -> [utime, stime]
	prevTotal    uint64
	prevIdle     uint64
}

// New creates a procfs Repository targeting the given proc root.
// Pass "" to use the default "/proc".
func New(procRoot string) *Repository {
	if procRoot == "" {
		procRoot = defaultProcRoot
	}
	return &Repository{
		procRoot:     procRoot,
		prevCPUTimes: make(map[int][2]uint64),
	}
}

// FetchAll returns metrics for all visible processes.
func (r *Repository) FetchAll(ctx context.Context) ([]entity.Metric, error) {
	pids, err := r.listPIDs()
	if err != nil {
		return nil, err
	}

	total, idle, err := readProcStat(r.procRoot)
	if err != nil {
		return nil, fmt.Errorf("read /proc/stat: %w", err)
	}
	totalDelta := float64(total - r.prevTotal)
	idleDelta := float64(idle - r.prevIdle)
	r.prevTotal = total
	r.prevIdle = idle

	var metrics []entity.Metric
	for _, pid := range pids {
		select {
		case <-ctx.Done():
			return metrics, ctx.Err()
		default:
		}

		m, err := r.fetchOne(pid, totalDelta)
		if err != nil {
			continue // process may have exited
		}
		_ = idleDelta
		metrics = append(metrics, m)
	}
	return metrics, nil
}

// FetchSorted returns metrics sorted by the given key (descending for CPU/Mem, ascending for PID).
func (r *Repository) FetchSorted(ctx context.Context, by repository.SortKey) ([]entity.Metric, error) {
	metrics, err := r.FetchAll(ctx)
	if err != nil {
		return nil, err
	}

	switch by {
	case repository.SortByCPU:
		sort.Slice(metrics, func(i, j int) bool {
			return metrics[i].CPUPercent > metrics[j].CPUPercent
		})
	case repository.SortByMem:
		sort.Slice(metrics, func(i, j int) bool {
			return metrics[i].MemBytes > metrics[j].MemBytes
		})
	default: // SortByPID
		sort.Slice(metrics, func(i, j int) bool {
			return metrics[i].PID < metrics[j].PID
		})
	}
	return metrics, nil
}

// Stream sends metrics snapshots to the returned channel at the given interval.
// The channel is closed when ctx is cancelled.
func (r *Repository) Stream(ctx context.Context) (<-chan entity.Metric, error) {
	ch := make(chan entity.Metric, 128)
	go func() {
		defer close(ch)
		ticker := time.NewTicker(2 * time.Second)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				metrics, err := r.FetchAll(ctx)
				if err != nil {
					continue
				}
				for _, m := range metrics {
					select {
					case ch <- m:
					case <-ctx.Done():
						return
					}
				}
			}
		}
	}()
	return ch, nil
}

// fetchOne builds a Metric for a single PID.
func (r *Repository) fetchOne(pid int, totalDelta float64) (entity.Metric, error) {
	statPath := filepath.Join(r.procRoot, strconv.Itoa(pid), "stat")
	data, err := os.ReadFile(statPath)
	if err != nil {
		return entity.Metric{}, err
	}

	line := string(data)
	_, name, _, utime, stime, _, rss, err := parseStat(line[:len(line)-1]) // strip trailing \n
	if err != nil {
		return entity.Metric{}, err
	}

	prev := r.prevCPUTimes[pid]
	utimeDelta := float64(utime - prev[0])
	stimeDelta := float64(stime - prev[1])
	r.prevCPUTimes[pid] = [2]uint64{utime, stime}

	var cpuPct float64
	if totalDelta > 0 {
		cpuPct = (utimeDelta + stimeDelta) / totalDelta * 100
	}

	return entity.Metric{
		PID:        pid,
		Name:       name,
		CPUPercent: cpuPct,
		MemBytes:   uint64(rss) * pageSize,
		Timestamp:  time.Now(),
	}, nil
}

// listPIDs returns all numeric entries in procRoot (i.e. all running PIDs).
func (r *Repository) listPIDs() ([]int, error) {
	entries, err := os.ReadDir(r.procRoot)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", r.procRoot, err)
	}

	var pids []int
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		pid, err := strconv.Atoi(e.Name())
		if err != nil {
			continue
		}
		pids = append(pids, pid)
	}
	return pids, nil
}

// ListProcesses returns ProcessInfo for all visible processes.
func (r *Repository) ListProcesses() ([]entity.ProcessInfo, error) {
	pids, err := r.listPIDs()
	if err != nil {
		return nil, err
	}
	var procs []entity.ProcessInfo
	for _, pid := range pids {
		statPath := filepath.Join(r.procRoot, strconv.Itoa(pid), "stat")
		data, err := os.ReadFile(statPath)
		if err != nil {
			continue
		}
		line := string(data)
		_, name, state, _, _, _, _, err := parseStat(line[:len(line)-1])
		if err != nil {
			continue
		}
		procs = append(procs, entity.ProcessInfo{PID: pid, Name: name, State: state})
	}
	return procs, nil
}

// Meminfo returns total and available memory.
func (r *Repository) Meminfo() (memTotal, memAvailable uint64, err error) {
	f, err := os.Open(filepath.Join(r.procRoot, "meminfo"))
	if err != nil {
		return 0, 0, err
	}
	defer f.Close()
	return parseMeminfo(bufio.NewReader(f))
}
