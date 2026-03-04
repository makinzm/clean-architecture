package entity

// ProcessInfo holds basic information about a running process.
type ProcessInfo struct {
	PID   int
	Name  string
	State string // R=Running, S=Sleeping, D=Disk sleep, Z=Zombie, T=Stopped
}
