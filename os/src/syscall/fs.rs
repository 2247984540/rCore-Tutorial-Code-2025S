//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;




/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel: sys_write");

    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            match core::str::from_utf8(slice){
                Ok(s) => {
                    print!("{}", s);
                    len as isize
                },
                Err(_) => {
                    crate::task::exit_current_and_run_next();
                    0
                },
            }
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
