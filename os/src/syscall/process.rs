//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

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

// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    use crate::trap::get_count;
    match _trace_request {
        0 => {
            let id_ptr = _id as *const u8; // 将 usize 转换为 *const u8
            let value = unsafe { *id_ptr }; // 读取指针指向的值
            return value as isize; // 返回值
        },

        1 =>{
            let id_ptr = _id as *mut u8; // 将 usize 转换为 *mut u8
            unsafe { *id_ptr = _data as u8 }; // 写入值
            return 0; // 返回 0 表示成功
        },

        2 => get_count(_id) as isize,
        _ => return -1,
    }
}
