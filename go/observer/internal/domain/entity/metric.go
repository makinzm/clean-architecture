package entity

import "time"

// Metric holds resource metrics for a single process at a point in time.
type Metric struct {
	PID        int
	Name       string
	CPUPercent float64
	MemBytes   uint64
	Timestamp  time.Time
}
