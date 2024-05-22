//! 文件和文件系统相关系统调用

const FD_STDOUT: usize = 1;

/// fd（file depictor）文件标识符, 在缓冲区buf起始地址开始写入len个字节
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize //sys_write系统调用的返回值是写入的有效字节数
        },
        _ => panic!("Unsupported fd: {} in sys_write", fd),
    }
}