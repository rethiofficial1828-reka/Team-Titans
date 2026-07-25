//! Deterministic containment via Windows Job Object deep-freeze.
//! Unlike NtSuspendProcess (race-condition vulnerable),
//! JobObjectFreezeInformation uses PsFreezeProcess: race-free, covers future threads.

use std::ffi::c_void;
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, TerminateJobObject,
    JOBOBJECTINFOCLASS,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

pub struct FrozenAttacker {
    job: HANDLE,
    pub pid: u32,
}

impl FrozenAttacker {
    /// Freeze the offending process the instant a decoy trips.
    /// Returns a handle we hold to keep it frozen (closing = unfreeze).
    pub fn contain(pid: u32) -> Result<Self, String> {
        unsafe {
            let job = CreateJobObjectW(None, None).map_err(|e| format!("CreateJobObject failed: {}", e))?;

            let proc = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|e| {
                let _ = CloseHandle(job);
                format!("OpenProcess({}) failed: {}", pid, e)
            })?;

            if let Err(e) = AssignProcessToJobObject(job, proc) {
                let _ = CloseHandle(proc);
                let _ = CloseHandle(job);
                return Err(format!("AssignProcessToJobObject failed: {}", e));
            }

            // JOBOBJECT_FREEZE_INFORMATION { Flags=1 (freeze), Freeze=TRUE }
            // Freeze info class = 18. Deep-freeze: race-free, catches new threads.
            #[repr(C)]
            struct JobFreezeInfo {
                flags: u32,
                freeze: u8,
                _pad: [u8; 3],
            }
            let info = JobFreezeInfo {
                flags: 1,
                freeze: 1,
                _pad: [0; 3],
            };
            const JOB_FREEZE_CLASS: i32 = 18;

            let ok = SetInformationJobObject(
                job,
                JOBOBJECTINFOCLASS(18),
                &info as *const _ as *const c_void,
                std::mem::size_of::<JobFreezeInfo>() as u32,
            );
            
            let _ = CloseHandle(proc);
            
            if let Err(e) = ok {
                let _ = CloseHandle(job);
                return Err(format!("Freeze failed: {}", e));
            }
            Ok(FrozenAttacker { job, pid })
        }
    }

    /// After forensic capture, terminate the whole job (all attacker threads).
    pub fn terminate(self) {
        unsafe {
            let _ = TerminateJobObject(self.job, 1);
            let _ = CloseHandle(self.job);
        }
    }
}

pub enum ContainmentStrategy {
    /// Instant freeze + forensic capture (default). Attacker sees nothing.
    SilentFreeze,
    /// Freeze + fake-success I/O responses for a window, harvesting its
    /// key-derivation behavior before terminating. (GuardFS OBF mode.)
    ObserveThenTerminate { observe_ms: u64 },
}
