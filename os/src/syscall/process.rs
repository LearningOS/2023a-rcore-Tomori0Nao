//! Process management syscalls

use crate::{
    mm::{virt_addr_to_phy_addr, VirtAddr},
    task::{
        change_program_brk, exit_current_and_run_next, get_task_info, suspend_current_and_run_next, task_mmap, task_unmap, TaskInfo
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

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
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();

    let v_addr = VirtAddr::from(_ts as usize);
    // // from virtAddr to PhysAddr
    trace!("kernel: v_addr is {}", v_addr.0);
    let p_addr = virt_addr_to_phy_addr(v_addr);

    trace!("kernel: p_addr is {}", p_addr.0);
    trace!("kernel: time is {}", us);

    let ts = p_addr.0 as *mut TimeVal;
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
        // panic!("!!!!");
        // trace!("kernel: sec is  {}",(*ts).sec);
        // print!("sec is {}",(*ts).sec);
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let v_addr = VirtAddr::from(_ti as usize);
    let p_addr = virt_addr_to_phy_addr(v_addr);

    let ti = p_addr.0 as *mut TaskInfo;

    get_task_info(ti);

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _port &! 0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    } 
    if _start % 4096 != 0 {
        return  -1;
    }
    // -1
    task_mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % 4096 != 0 {
        return  -1;
    }
    task_unmap(_start, _len)
    // -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
