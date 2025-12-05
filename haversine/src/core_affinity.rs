#[cfg(target_family = "unix")]
use std::{io, mem};

use cfg_if::cfg_if;

#[cfg(target_family = "unix")]

cfg_if! {
    if #[cfg(unix)] {
        pub fn set_single_core() -> Result<(), io::Error> {
            unsafe {
                let cpu = libc::sched_getcpu();
                if cpu == -1 {
                    return Err(io::Error::last_os_error());
                }

                let mut cpuset: libc::cpu_set_t = mem::zeroed();
                libc::CPU_ZERO(&mut cpuset);

                libc::CPU_SET(cpu as usize, &mut cpuset);

                let ret = libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &cpuset);

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
    }
}
