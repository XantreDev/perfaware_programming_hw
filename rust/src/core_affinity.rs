use std::{io, mem};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(unix)] {
        const CPU_SET_SIZE: usize = size_of::<libc::cpu_set_t>();
        pub fn get_core_affinity() -> Result<libc::cpu_set_t, io::Error> {
            unsafe {
                let mut cpuset: libc::cpu_set_t = mem::zeroed();
                let res = libc::sched_getaffinity(0, CPU_SET_SIZE, &mut cpuset);

                if res != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(cpuset)
            }
        }

        pub fn set_core_affinity(cpuset: &libc::cpu_set_t) -> Result<(), io::Error> {
            unsafe {
                let ret = libc::sched_setaffinity(0, CPU_SET_SIZE, cpuset);
                if ret != 0 {
                    return Err(io::Error::last_os_error());
                }

                Ok(())
            }
        }
        pub fn set_single_core() -> Result<(), io::Error> {
            unsafe {
                let cpu = libc::sched_getcpu();
                if cpu == -1 {
                    return Err(io::Error::last_os_error());
                }

                let mut cpuset: libc::cpu_set_t = mem::zeroed();
                libc::CPU_ZERO(&mut cpuset);

                libc::CPU_SET(cpu as usize, &mut cpuset);

                let ret = libc::sched_setaffinity(0, CPU_SET_SIZE, &cpuset);

                if ret != 0 {
                    return Err(io::Error::last_os_error());
                }

                Ok(())
            }
        }
    } else {
        pub fn set_single_core() -> Result<(), std::io:Error> {
            Ok(())
        }
        pub fn get_core_affinity()  -> Result<libc::cpu_set_t, io::Error> {
            let cpuset: libc::cpu_set_t = mem::zeroed();
            return cpuset;
        }

        pub fn set_core_affinity(cpuset: &libc::cpu_set_t) -> Result<(), io::Error> {
            Ok(())
        }
    }
}
