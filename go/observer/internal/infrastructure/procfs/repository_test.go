package procfs

import (
	"bufio"
	"context"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"

	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
)

// ---------------------------------------------------------------------------
// parseStat tests
// ---------------------------------------------------------------------------

func TestParseStat_Simple(t *testing.T) {
	data, err := os.ReadFile("testdata/stat_simple")
	if err != nil {
		t.Fatal(err)
	}
	line := strings.TrimRight(string(data), "\n")

	pid, name, state, utime, stime, vsize, rss, err := parseStat(line)
	if err != nil {
		t.Fatalf("parseStat error: %v", err)
	}

	if pid != 1234 {
		t.Errorf("pid: got %d, want 1234", pid)
	}
	if name != "bash" {
		t.Errorf("name: got %q, want %q", name, "bash")
	}
	if state != "S" {
		t.Errorf("state: got %q, want S", state)
	}
	if utime != 42 {
		t.Errorf("utime: got %d, want 42", utime)
	}
	if stime != 10 {
		t.Errorf("stime: got %d, want 10", stime)
	}
	if vsize == 0 {
		t.Errorf("vsize: got 0, want non-zero")
	}
	_ = rss
}

func TestParseStat_SpacesInName(t *testing.T) {
	data, err := os.ReadFile("testdata/stat_spaces_in_name")
	if err != nil {
		t.Fatal(err)
	}
	line := strings.TrimRight(string(data), "\n")

	pid, name, state, utime, stime, _, _, err := parseStat(line)
	if err != nil {
		t.Fatalf("parseStat error: %v", err)
	}
	if pid != 5678 {
		t.Errorf("pid: got %d, want 5678", pid)
	}
	if name != "my (proc) name" {
		t.Errorf("name: got %q, want %q", name, "my (proc) name")
	}
	if state != "R" {
		t.Errorf("state: got %q, want R", state)
	}
	if utime != 100 {
		t.Errorf("utime: got %d, want 100", utime)
	}
	if stime != 20 {
		t.Errorf("stime: got %d, want 20", stime)
	}
}

func TestParseStat_InvalidLine(t *testing.T) {
	_, _, _, _, _, _, _, err := parseStat("garbage line without parens")
	if err == nil {
		t.Error("expected error for invalid stat line, got nil")
	}
}

// ---------------------------------------------------------------------------
// parseMeminfo tests
// ---------------------------------------------------------------------------

func TestParseMeminfo(t *testing.T) {
	f, err := os.Open("testdata/meminfo")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	total, avail, err := parseMeminfo(bufio.NewReader(f))
	if err != nil {
		t.Fatalf("parseMeminfo error: %v", err)
	}

	wantTotal := uint64(16384000 * 1024)
	wantAvail := uint64(8192000 * 1024)
	if total != wantTotal {
		t.Errorf("MemTotal: got %d, want %d", total, wantTotal)
	}
	if avail != wantAvail {
		t.Errorf("MemAvailable: got %d, want %d", avail, wantAvail)
	}
}

// ---------------------------------------------------------------------------
// Repository integration test (uses a synthetic /proc-like directory)
// ---------------------------------------------------------------------------

func makeFakeProcRoot(t *testing.T, cpuLine string, processes []struct {
	pid   int
	utime string
}) string {
	t.Helper()
	root := t.TempDir()
	if err := os.WriteFile(filepath.Join(root, "stat"), []byte(cpuLine), 0644); err != nil {
		t.Fatal(err)
	}
	for _, p := range processes {
		pidStr := strconv.Itoa(p.pid)
		pidDir := filepath.Join(root, pidStr)
		if err := os.MkdirAll(pidDir, 0755); err != nil {
			t.Fatal(err)
		}
		s := pidStr + " (proc" + pidStr + ") S 1 " + pidStr + " " + pidStr +
			" 0 " + pidStr + " 4194304 0 0 0 0 " + p.utime + " 0 0 0 20 0 1 0 100 4096000 100" +
			" 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0\n"
		if err := os.WriteFile(filepath.Join(pidDir, "stat"), []byte(s), 0644); err != nil {
			t.Fatal(err)
		}
	}
	return root
}

func TestRepository_FetchAll(t *testing.T) {
	root := t.TempDir()

	if err := os.WriteFile(filepath.Join(root, "stat"), []byte("cpu  100 0 50 800 0 0 0 0 0 0\n"), 0644); err != nil {
		t.Fatal(err)
	}
	pidDir := filepath.Join(root, "42")
	if err := os.MkdirAll(pidDir, 0755); err != nil {
		t.Fatal(err)
	}
	statLine := "42 (test_proc) S 1 42 42 0 42 4194304 0 0 0 0 10 5 0 0 20 0 1 0 100 8192000 500 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0\n"
	if err := os.WriteFile(filepath.Join(pidDir, "stat"), []byte(statLine), 0644); err != nil {
		t.Fatal(err)
	}

	repo := New(root)
	metrics, err := repo.FetchAll(context.Background())
	if err != nil {
		t.Fatalf("FetchAll error: %v", err)
	}
	if len(metrics) != 1 {
		t.Fatalf("expected 1 metric, got %d", len(metrics))
	}
	m := metrics[0]
	if m.PID != 42 {
		t.Errorf("PID: got %d, want 42", m.PID)
	}
	if m.Name != "test_proc" {
		t.Errorf("Name: got %q, want test_proc", m.Name)
	}
	if m.MemBytes == 0 {
		t.Errorf("MemBytes: got 0, want non-zero")
	}
}

func TestRepository_FetchSorted_ByCPU(t *testing.T) {
	procs := []struct {
		pid   int
		utime string
	}{
		{10, "100"},
		{20, "500"},
		{30, "50"},
	}
	root := makeFakeProcRoot(t, "cpu  1000 0 500 5000 0 0 0 0 0 0\n", procs)

	repo := New(root)
	// First call to initialise prev CPU state
	_, _ = repo.FetchAll(context.Background())

	// Advance system CPU ticks
	if err := os.WriteFile(filepath.Join(root, "stat"), []byte("cpu  2000 0 1000 8000 0 0 0 0 0 0\n"), 0644); err != nil {
		t.Fatal(err)
	}

	sorted, err := repo.FetchSorted(context.Background(), repository.SortByCPU)
	if err != nil {
		t.Fatalf("FetchSorted error: %v", err)
	}
	if len(sorted) == 0 {
		t.Fatal("expected at least one metric")
	}
	for i := 1; i < len(sorted); i++ {
		if sorted[i-1].CPUPercent < sorted[i].CPUPercent {
			t.Errorf("not sorted descending by CPU at index %d: %.2f < %.2f",
				i, sorted[i-1].CPUPercent, sorted[i].CPUPercent)
		}
	}
}

func TestRepository_FetchSorted_ByMem(t *testing.T) {
	procs := []struct {
		pid   int
		utime string
	}{
		{10, "0"},
		{20, "0"},
		{30, "0"},
	}
	root := makeFakeProcRoot(t, "cpu  1000 0 500 5000 0 0 0 0 0 0\n", procs)

	repo := New(root)
	sorted, err := repo.FetchSorted(context.Background(), repository.SortByMem)
	if err != nil {
		t.Fatalf("FetchSorted error: %v", err)
	}
	for i := 1; i < len(sorted); i++ {
		if sorted[i-1].MemBytes < sorted[i].MemBytes {
			t.Errorf("not sorted descending by Mem at index %d", i)
		}
	}
}
