package entity

import "time"

// EventKind describes the type of domain event.
type EventKind string

const (
	EventProcessStarted    EventKind = "process_started"
	EventProcessExited     EventKind = "process_exited"
	EventThresholdBreached EventKind = "threshold_breached"
)

// Event represents a significant change detected by the observer.
type Event struct {
	Kind    EventKind
	Process ProcessInfo
	Metric  *Metric // non-nil when Kind == EventThresholdBreached
	At      time.Time
}
