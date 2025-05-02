//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

use crate::syscall::{syscall, SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_INIT_COUNT, SYSCALL_TRACE, SYSCALL_WRITE, SYSCALL_YIELD};
// use crate::syscall::SYSCALL_TRACE;
// use crate::syscall::SYSCALL_WRITE;
// use crate::syscall::SYSCALL_YIELD;
// use crate::syscall::SYSCALL_EXIT;
// use crate::syscall::SYSCALL_GET_TIME;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

//自己实现的ch3:在当前文件中创建几个全局变量分别记录不同的系统调用的次数
static mut SYSCALL_COUNT_TRACE: usize = 0;
static mut SYSCALL_COUNT_WRITE: usize = 0;
static mut SYSCALL_COUNT_YIELD: usize = 0;
static mut SYSCALL_COUNT_EXIT: usize = 0;
static mut SYSCALL_COUNT_TIME: usize = 0;
static mut SYSCALL_COUNT_INIT: usize = 0;

///my_ch3 :通过传入的系统调用号将对应的系统调用计数器清零
/// 如果传入的系统调用号是SYSCALL_INIT_COUNT,则将所有的系统调用计数器清零
pub fn init_syscall_count(id : usize) -> usize {
    match id{
        SYSCALL_TRACE => unsafe {SYSCALL_COUNT_TRACE = 0},
        SYSCALL_WRITE => unsafe {SYSCALL_COUNT_WRITE = 0},
        SYSCALL_YIELD => unsafe {SYSCALL_COUNT_YIELD = 0},
        SYSCALL_EXIT => unsafe {SYSCALL_COUNT_EXIT = 0},
        SYSCALL_GET_TIME => unsafe {SYSCALL_COUNT_TIME = 0},
        SYSCALL_INIT_COUNT => unsafe {
            SYSCALL_COUNT_INIT = 0;
            SYSCALL_COUNT_TRACE = 0;
            SYSCALL_COUNT_WRITE = 0;
            SYSCALL_COUNT_YIELD = 0;
            SYSCALL_COUNT_EXIT = 0;
            SYSCALL_COUNT_TIME = 0;
        },
        _ => panic!("Unsupported syscall_id: {}", id),
    }
    1
}

///my_ch3 :根据系统调用号更新系统调用的全局变量
pub fn my_count_syscall(syscall_id: usize) {
    match syscall_id {
        SYSCALL_TRACE => unsafe {SYSCALL_COUNT_TRACE += 1},
        SYSCALL_WRITE => unsafe {SYSCALL_COUNT_WRITE += 1},
        SYSCALL_YIELD => unsafe {SYSCALL_COUNT_YIELD += 1},
        SYSCALL_EXIT => unsafe {SYSCALL_COUNT_EXIT += 1},
        SYSCALL_GET_TIME => unsafe {SYSCALL_COUNT_TIME += 1},
        SYSCALL_INIT_COUNT => unsafe {SYSCALL_COUNT_INIT += 1},
        _ => panic!("Unsupported syscall_id: {}", syscall_id),       
        
    }
}
///my_ch3 :根据系统调用号返回当前系统调用的调用次数
pub fn get_count(syscall_id:usize) -> usize {
    match syscall_id {
        SYSCALL_TRACE => unsafe {SYSCALL_COUNT_TRACE},
        SYSCALL_WRITE => unsafe {SYSCALL_COUNT_WRITE},
        SYSCALL_YIELD => unsafe {SYSCALL_COUNT_YIELD},
        SYSCALL_EXIT => unsafe {SYSCALL_COUNT_EXIT},
        SYSCALL_GET_TIME => unsafe {SYSCALL_COUNT_TIME},
        _ => panic!("Unsupported syscall_id: {}", syscall_id),       
        
    }
}
    
/// Initialize trap handling
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// enable timer interrupt in supervisor mode
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// trap handler
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
                               // trace!("into {:?}", scause.cause());
    match scause.cause() {
        //用户态请求系统调用
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            cx.sepc += 4;
            //ch3:在调用系统调用之前先将系统调用计数器更新
            my_count_syscall(cx.x[17]);   //将计数器修改到这里调用
            // get system call return value
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }

        //这两种异常表示在应用程序中发生了不当的内存写操作，可能是访问了未分配的内存。
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }

        //应用程序试图执行无效或不存在的指令
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}

pub use context::TrapContext;
