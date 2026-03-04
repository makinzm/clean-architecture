//go:build linux && generatebpf

// Package ebpf provides a MetricRepository that uses eBPF tracepoints
// (sched_process_exec / sched_process_exit) to detect process lifecycle
// events without polling.
//
// Code generation:
//
//	go generate ./internal/infrastructure/ebpf/...
//
// requires clang + bpf2go (cilium/ebpf).
package ebpf

//go:generate go run github.com/cilium/ebpf/cmd/bpf2go -cc clang -cflags "-O2 -g -Wall -Werror" Gaze bpf/gaze.c -- -I/usr/include/bpf

import (
	"context"
	"encoding/binary"
	"fmt"
	"time"
	"unsafe"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"

	"github.com/cilium/ebpf"
	"github.com/cilium/ebpf/link"
	"github.com/cilium/ebpf/ringbuf"
)

const (
	eventExec   = 1
	eventExit   = 2
	taskCommLen = 16
)

// procEvent mirrors the C struct proc_event.
type procEvent struct {
	Type     uint32
	PID      uint32
	PPID     uint32
	ExitCode uint32
	Comm     [taskCommLen]byte
}

// Repository implements repository.MetricRepository using eBPF.
// It streams EventProcessStarted/Exited via the ring buffer and delegates
// FetchAll/FetchSorted to a backing procfs repository for metric snapshots.
type Repository struct {
	backing   repository.MetricRepository // procfs repo for snapshot data
	objs      *GazeObjects
	traceExec link.Link
	traceExit link.Link
	reader    *ringbuf.Reader
}

// New loads the eBPF program and attaches the tracepoints.
// Call Close() when done.
func New(backing repository.MetricRepository) (*Repository, error) {
	objs := &GazeObjects{}
	if err := LoadGazeObjects(objs, nil); err != nil {
		return nil, fmt.Errorf("load eBPF objects: %w", err)
	}

	traceExec, err := link.Tracepoint("sched", "sched_process_exec", objs.HandleExec, nil)
	if err != nil {
		objs.Close()
		return nil, fmt.Errorf("attach exec tracepoint: %w", err)
	}

	traceExit, err := link.Tracepoint("sched", "sched_process_exit", objs.HandleExit, nil)
	if err != nil {
		traceExec.Close()
		objs.Close()
		return nil, fmt.Errorf("attach exit tracepoint: %w", err)
	}

	rd, err := ringbuf.NewReader(objs.Events)
	if err != nil {
		traceExec.Close()
		traceExit.Close()
		objs.Close()
		return nil, fmt.Errorf("open ring buffer: %w", err)
	}

	return &Repository{
		backing:   backing,
		objs:      objs,
		traceExec: traceExec,
		traceExit: traceExit,
		reader:    rd,
	}, nil
}

// Close releases all eBPF resources.
func (r *Repository) Close() {
	r.reader.Close()
	r.traceExec.Close()
	r.traceExit.Close()
	r.objs.Close()
}

// FetchAll delegates to the backing procfs repository.
func (r *Repository) FetchAll(ctx context.Context) ([]entity.Metric, error) {
	return r.backing.FetchAll(ctx)
}

// FetchSorted delegates to the backing procfs repository.
func (r *Repository) FetchSorted(ctx context.Context, by repository.SortKey) ([]entity.Metric, error) {
	return r.backing.FetchSorted(ctx, by)
}

// Stream reads process events from the eBPF ring buffer and emits them
// as synthetic Metric entries. Cancel ctx to stop.
func (r *Repository) Stream(ctx context.Context) (<-chan entity.Metric, error) {
	ch := make(chan entity.Metric, 64)
	go func() {
		defer close(ch)
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}
			record, err := r.reader.Read()
			if err != nil {
				if ebpf.ErrClosed == err {
					return
				}
				continue
			}
			e := parseEvent(record.RawSample)
			if e == nil {
				continue
			}
			name := commToString(e.Comm)
			select {
			case ch <- entity.Metric{
				PID:       int(e.PID),
				Name:      name,
				Timestamp: time.Now(),
			}:
			case <-ctx.Done():
				return
			}
		}
	}()
	return ch, nil
}

// Events streams domain events (started/exited) from the eBPF ring buffer.
// This is an extension beyond the MetricRepository interface for consumers
// that want direct event access.
func (r *Repository) Events(ctx context.Context) (<-chan entity.Event, error) {
	ch := make(chan entity.Event, 64)
	go func() {
		defer close(ch)
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}
			record, err := r.reader.Read()
			if err != nil {
				return
			}
			e := parseEvent(record.RawSample)
			if e == nil {
				continue
			}
			name := commToString(e.Comm)
			kind := entity.EventProcessStarted
			if e.Type == eventExit {
				kind = entity.EventProcessExited
			}
			select {
			case ch <- entity.Event{
				Kind:    kind,
				Process: entity.ProcessInfo{PID: int(e.PID), Name: name},
				At:      time.Now(),
			}:
			case <-ctx.Done():
				return
			}
		}
	}()
	return ch, nil
}

func parseEvent(raw []byte) *procEvent {
	if len(raw) < int(unsafe.Sizeof(procEvent{})) {
		return nil
	}
	e := &procEvent{}
	e.Type = binary.LittleEndian.Uint32(raw[0:4])
	e.PID = binary.LittleEndian.Uint32(raw[4:8])
	e.PPID = binary.LittleEndian.Uint32(raw[8:12])
	e.ExitCode = binary.LittleEndian.Uint32(raw[12:16])
	copy(e.Comm[:], raw[16:16+taskCommLen])
	return e
}

func commToString(comm [taskCommLen]byte) string {
	for i, b := range comm {
		if b == 0 {
			return string(comm[:i])
		}
	}
	return string(comm[:])
}
