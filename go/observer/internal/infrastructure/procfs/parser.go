package procfs

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
)

// parseStat parses a single /proc/[pid]/stat line into a partial Metric.
// The stat line format: pid (comm) state ppid ...
// Returns pid, name, state, utime, stime, starttime (all in clock ticks), vsize (bytes), rss (pages).
func parseStat(line string) (pid int, name, state string, utime, stime, vsize uint64, rss int64, err error) {
	// The comm field may contain spaces and is wrapped in parentheses.
	// Find the last ')' to safely split.
	open := strings.Index(line, "(")
	close := strings.LastIndex(line, ")")
	if open == -1 || close == -1 || close <= open {
		return 0, "", "", 0, 0, 0, 0, fmt.Errorf("invalid stat line")
	}

	pidStr := strings.TrimSpace(line[:open])
	pid, err = strconv.Atoi(pidStr)
	if err != nil {
		return
	}
	name = line[open+1 : close]

	fields := strings.Fields(line[close+2:]) // skip ') '
	if len(fields) < 20 {
		return 0, "", "", 0, 0, 0, 0, fmt.Errorf("stat line too short")
	}
	state = fields[0]

	utime, err = strconv.ParseUint(fields[11], 10, 64)
	if err != nil {
		return
	}
	stime, err = strconv.ParseUint(fields[12], 10, 64)
	if err != nil {
		return
	}
	vsize, err = strconv.ParseUint(fields[20], 10, 64)
	if err != nil {
		return
	}
	rssVal, err := strconv.ParseInt(fields[21], 10, 64)
	if err != nil {
		return 0, "", "", 0, 0, 0, 0, err
	}
	rss = rssVal
	return
}

// parseMeminfo parses /proc/meminfo and returns MemTotal and MemAvailable in bytes.
func parseMeminfo(r *bufio.Reader) (memTotal, memAvailable uint64, err error) {
	for {
		line, readErr := r.ReadString('\n')
		if line != "" {
			fields := strings.Fields(line)
			if len(fields) >= 2 {
				val, parseErr := strconv.ParseUint(fields[1], 10, 64)
				if parseErr == nil {
					switch fields[0] {
					case "MemTotal:":
						memTotal = val * 1024
					case "MemAvailable:":
						memAvailable = val * 1024
					}
				}
			}
		}
		if readErr != nil {
			break
		}
	}
	return
}

// readProcStat reads /proc/stat and returns total and idle CPU ticks.
func readProcStat(procRoot string) (total, idle uint64, err error) {
	f, err := os.Open(filepath.Join(procRoot, "stat"))
	if err != nil {
		return
	}
	defer f.Close()

	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text()
		if !strings.HasPrefix(line, "cpu ") {
			continue
		}
		fields := strings.Fields(line)
		if len(fields) < 5 {
			return 0, 0, fmt.Errorf("cpu line too short: %s", line)
		}
		for i, f := range fields[1:] {
			v, e := strconv.ParseUint(f, 10, 64)
			if e != nil {
				return 0, 0, e
			}
			total += v
			if i == 3 { // idle field
				idle = v
			}
		}
		return total, idle, nil
	}
	return 0, 0, fmt.Errorf("cpu line not found in /proc/stat")
}

// metricFromStatFile reads /proc/[pid]/stat and returns a partial Metric.
// cpuPercent is computed by the repository using two samples.
func metricFromStatFile(procRoot string, pid int) (entity.Metric, error) {
	statPath := filepath.Join(procRoot, strconv.Itoa(pid), "stat")
	data, err := os.ReadFile(statPath)
	if err != nil {
		return entity.Metric{}, err
	}

	pidParsed, name, state, _, _, vsize, _, err := parseStat(strings.TrimRight(string(data), "\n"))
	if err != nil {
		return entity.Metric{}, err
	}
	_ = state // used by ProcessInfo, not Metric directly

	return entity.Metric{
		PID:       pidParsed,
		Name:      name,
		MemBytes:  vsize,
		Timestamp: time.Now(),
	}, nil
}
