pub mod sys_info {
    use std::mem;
    use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    pub struct SysData {
        pub mem_used: u64,
        pub mem_total: u64,
        pub memory_load: u32,
    }

    impl Default for SysData {
        fn default() -> Self {
            unsafe { mem::zeroed() }
        }
    }

    pub fn get_memory_status() -> SysData {
        let mut memory_status: MEMORYSTATUSEX = unsafe { mem::zeroed() };
        memory_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

        unsafe {
            GlobalMemoryStatusEx(&mut memory_status);
        }
        let total_memory = memory_status.ullTotalPhys; // 总物理内存
        let free_memory = memory_status.ullAvailPhys; // 可用物理内存
        let memory_load = memory_status.dwMemoryLoad; // 内存使用百分比
        SysData {
            mem_used: total_memory - free_memory,
            mem_total: total_memory,
            memory_load: memory_load,
        }
    }
}
