use std::num::NonZeroU8;

pub enum CpuSchedulingPolicy {
    Other = 0,
    Batch = 3,
    Idle = 5,
    Fifo = 1,
    RoundRobin = 2,
}

/// The CPU scheduling for running a transient service on the system service
/// manager.
/// See `CPUSchedulingPolicy=`, `CPUSchedulingPriority=`, and
/// `CPUSchedulingResetOnFork=` in [systemd.exec(5)](man:systemd.exec(5))
/// and [sched_setscheduler(2)](man:sched_setscheduler(2)) for details.
pub struct CpuScheduling {
    policy: CpuSchedulingPolicy,
    real_time_priority: Option<u8>,
    reset_on_fork: bool,
}

pub fn marshal(sched: CpuScheduling) -> (i32, Option<i32>, bool) {
    let a = sched.policy as i32;
    let b = sched.real_time_priority.map(u8::into);
    (a, b, sched.reset_on_fork)
}

impl Default for CpuScheduling {
    /// The default CPU scheduling policy, `SCHED_OTHER`.
    fn default() -> Self {
        Self {
            policy: CpuSchedulingPolicy::Other,
            real_time_priority: None,
            reset_on_fork: false,
        }
    }
}

impl CpuScheduling {
    /// For "batch" style execution of processes, `SCHED_BATCH`.
    pub fn batch() -> Self {
        Self {
            policy: CpuSchedulingPolicy::Batch,
            ..Self::default()
        }
    }

    /// For running very low priority background jobs, `SCHED_IDLE`.
    pub fn idle() -> Self {
        Self {
            policy: CpuSchedulingPolicy::Idle,
            ..Self::default()
        }
    }

    /// A first-in, first-out real-time policy, `SCHED_FIFO`, with specified
    /// priority. The priority must be in [1, 99].
    pub fn fifo(p: NonZeroU8) -> Self {
        Self {
            policy: CpuSchedulingPolicy::Fifo,
            real_time_priority: Some(p.into()),
            reset_on_fork: false,
        }
    }

    /// A round-robin real-time policy, `SCHED_RR`, with specified priority.
    /// The priority must be in [1, 99].
    pub fn round_robin(p: NonZeroU8) -> Self {
        Self {
            policy: CpuSchedulingPolicy::RoundRobin,
            real_time_priority: Some(p.into()),
            reset_on_fork: false,
        }
    }

    /// Make the children created by fork(2) do not inherit privileged
    /// scheduling policies.
    pub fn reset_on_fork(self) -> Self {
        Self {
            reset_on_fork: true,
            ..self
        }
    }
}
