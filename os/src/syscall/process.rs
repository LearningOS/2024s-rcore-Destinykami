//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{va_to_pa, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task, mmap, suspend_current_and_run_next, ummap, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

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
/// 引入虚存机制后，原来内核的 sys_get_time 和 sys_task_info 函数实现就无效了
/// 指针访问的是虚拟地址，需要通过这个地址找到物理地址，从物理地址获取实际的指针
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let token=current_user_token(); //当前任务的token
    let va=VirtAddr::from(ts as usize);//虚拟地址
    let pa=va_to_pa(token,va);//物理地址
    let p_ts:&mut TimeVal=pa.get_mut(); 
    let us=get_time_us();
    *p_ts=TimeVal{
        sec:us/1_000_000,
        usec:us,
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
/// 相比于ch3有较大的改动
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let taskinfo=get_current_task();
    let token=current_user_token(); //当前任务的token
    let va=VirtAddr::from(ti as usize);//虚拟地址
    let pa=va_to_pa(token,va);//物理地址
    let p_ti:&mut TaskInfo=pa.get_mut(); 

    (*p_ti).time=get_time_ms()-taskinfo.time;
    (*p_ti).syscall_times=taskinfo.syscall_times;
    (*p_ti).status=taskinfo.status;
    0
}

// YOUR JOB: Implement mmap.
//申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），
//将其映射到 start 开始的虚存，内存页属性为 port
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    //port除了后三位之外无效且必须为0,start按页号对齐
    if start%PAGE_SIZE!=0 || port&(!(0x7))!=0 ||port & 0x7 == 0 {
        return -1; 
    }
    mmap(start,len,port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start %  PAGE_SIZE != 0 {
        return -1;
    }
    ummap(start, len)
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
