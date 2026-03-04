// SPDX-License-Identifier: GPL-2.0
// Gaze eBPF program: hooks sched_process_exec and sched_process_exit
// tracepoints to detect process lifecycle events without polling.

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

// Event types sent to userspace
#define EVENT_EXEC 1
#define EVENT_EXIT 2

// Maximum length of process name (comm)
#define TASK_COMM_LEN 16

// Event structure passed through the ring buffer
struct proc_event {
    __u32 type;   // EVENT_EXEC or EVENT_EXIT
    __u32 pid;
    __u32 ppid;
    __u32 exit_code;
    char  comm[TASK_COMM_LEN];
};

// Ring buffer map for sending events to userspace
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024); // 256 KB
} events SEC(".maps");

// Hook: sched/sched_process_exec
// Fired when a process calls execve() successfully.
SEC("tracepoint/sched/sched_process_exec")
int handle_exec(struct trace_event_raw_sched_process_exec *ctx)
{
    struct proc_event *e;
    e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;

    e->type = EVENT_EXEC;
    e->pid  = bpf_get_current_pid_tgid() >> 32;
    e->ppid = 0; // filled by userspace if needed
    e->exit_code = 0;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));

    bpf_ringbuf_submit(e, 0);
    return 0;
}

// Hook: sched/sched_process_exit
// Fired when a process exits.
SEC("tracepoint/sched/sched_process_exit")
int handle_exit(struct trace_event_raw_sched_process_template *ctx)
{
    struct proc_event *e;
    e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;

    e->type      = EVENT_EXIT;
    e->pid       = bpf_get_current_pid_tgid() >> 32;
    e->ppid      = 0;
    e->exit_code = (BPF_CORE_READ(ctx, exit_code)) >> 8;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));

    bpf_ringbuf_submit(e, 0);
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
