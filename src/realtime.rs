// module required to make SmartPi work with preempt-rt
//use scheduler::*;
use libc::{pthread_setschedparam, SCHED_FIFO, pthread_self, sched_param, timespec};
pub fn schedule_fifo(priority: i32){
    unsafe {
        let param = sched_param { sched_priority: priority, sched_ss_init_budget: timespec {tv_sec:0, tv_nsec:0},               sched_ss_repl_period: timespec {tv_sec:0, tv_nsec:0}, sched_ss_low_priority:0, sched_ss_max_repl: 1};
        //let param = sched_param { sched_priority: priority };
        let t = pthread_self();
        pthread_setschedparam(t, SCHED_FIFO, &param as *const sched_param);
    }

}