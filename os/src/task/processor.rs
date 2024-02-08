//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::task::TaskInfo;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::mm::{MapPermission, VirtAddr};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    /// Set syscall times for current task
    pub fn set_syscall_times(&mut self, syscall_id: usize) {
        let inner = &mut self.current.as_ref().unwrap().inner_exclusive_access();
        inner.task_info.syscall_times[syscall_id] += 1;
    }
    /// pass task info to the pointer
    fn get_task_info(&self, _ti: *mut TaskInfo) {
        let inner = &mut self.current.as_ref().unwrap().inner_exclusive_access();
        // let current_id = inner.current_task;
        unsafe {
            (*_ti).status = TaskStatus::Running;
            (*_ti).syscall_times = inner.task_info.syscall_times;
            // 参考 https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/6answer.html 关于时间计算的实现
            (*_ti).time = get_time_ms() - inner.task_info.time;
        }
    }
    /// mmap for current task
    fn task_mmap(&self, _start: usize, _len: usize, _port: usize) -> isize {
        let inner = &mut self.current.as_ref().unwrap().inner_exclusive_access();
        // let current_id = inner.current_task;
        let mm_set = &mut inner.memory_set;

        let start_va = VirtAddr::from(_start);
        let end_va = VirtAddr::from(_start + _len);
        let permission = match _port {
            1 => MapPermission::R | MapPermission::U,
            2 => MapPermission::W | MapPermission::U,
            3 => {
                // println!("3!!!");
                MapPermission::R | MapPermission::W | MapPermission::U
            }
            4 => MapPermission::X | MapPermission::U,
            5 => MapPermission::R | MapPermission::X | MapPermission::U,
            6 => MapPermission::W | MapPermission::X | MapPermission::U,
            7 => MapPermission::R | MapPermission::W | MapPermission::X | MapPermission::U,
            _ => MapPermission::R,
        };
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        println!("start_vpn is {}, end_vpn is {}", start_vpn.0, end_vpn.0);
        // for mm_area in mm_set
        if mm_set.addr_exits(start_va, end_va) == -1 {
            return -1;
        }

        mm_set.insert_framed_area(start_va, end_va, permission);
        0
    }
    /// unmap for current task
    fn task_unmap(&self, _start: usize, _len: usize) -> isize {
        let inner = &mut self.current.as_ref().unwrap().inner_exclusive_access();
        let mm_set = &mut inner.memory_set;

        let start_va = VirtAddr::from(_start);
        let end_va = VirtAddr::from(_start + _len);

        mm_set.unmap(start_va, end_va)
        // 0
    }
    fn task_set_priority(&self, _prio: isize) -> isize {
        if _prio < 2 {
            -1
        } else {
            let inner = &mut self.current.as_ref().unwrap().inner_exclusive_access();
            inner.priority = _prio;
            inner.priority
        }
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
/// Set syscall times for current task
pub fn set_syscall_times(syscall_id: usize) {
    PROCESSOR.exclusive_access().set_syscall_times(syscall_id);
}
/// pass task info to the pointer
pub fn get_task_info(_ti: *mut TaskInfo) {
    PROCESSOR.exclusive_access().get_task_info(_ti);
}
/// mmap for current task
pub fn task_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    PROCESSOR.exclusive_access().task_mmap(_start, _len, _port)
}
/// unmap for current task
pub fn task_unmap(_start: usize, _len: usize) -> isize {
    PROCESSOR.exclusive_access().task_unmap(_start, _len)
}
/// set priority for current task
pub fn task_set_priority(_prio: isize) -> isize {
    PROCESSOR.exclusive_access().task_set_priority(_prio)
}
