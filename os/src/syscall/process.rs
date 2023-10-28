//! Process management syscalls
use crate::{
    // config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, get_task_info,TaskInfo},
    timer::{get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

// TaskInfo 定义在 Task 相关 crate 中更合适
/// Task information
// #[allow(dead_code)]
// pub struct TaskInfo {
//     /// Task status in it's life cycle
//     status: TaskStatus,
//     /// The numbers of syscall called by task
//     syscall_times: [u32; MAX_SYSCALL_NUM],
//     /// Total running time of task
//     time: usize,
// }

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// 此处即为修改传入的TaskInfo引用与当前Task的TaskInfo相同
/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // let mut inner = TASK_MANAGER.inner.exclusive_access();
    // let current_id = inner.current_task;
    // let inner.tasks[current_id].task_info;
    // *(_ti).status = TaskStatus::Running; 
    
    // 参考 https://werifu.github.io/posts/rcore-camp-2022-lab1/ 关于 TASK_MANAGER 对外接口的想法
    get_task_info(_ti);
    // _ti.syscall_times = inner.tasks[current_id].task_info.syscall_times;
    // _ti.time = get_time() - inner.tasks[current_id].task_info.time;
    0
}
