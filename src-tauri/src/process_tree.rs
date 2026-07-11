#[cfg(windows)]
use std::{
    os::windows::io::AsRawHandle,
    sync::atomic::{AtomicU8, Ordering},
};

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, TerminateJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    },
};

#[cfg(windows)]
pub(crate) struct ProcessTreeGuard {
    job: HANDLE,
    termination_state: AtomicU8,
}

#[cfg(windows)]
impl ProcessTreeGuard {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            anyhow::bail!("failed to create Windows process containment job");
        }

        let mut limits = unsafe { std::mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if configured == 0 {
            unsafe { CloseHandle(job) };
            anyhow::bail!("failed to configure Windows process containment job");
        }

        Ok(Self {
            job,
            termination_state: AtomicU8::new(TERMINATION_IDLE),
        })
    }

    pub(crate) fn assign_tokio(&self, child: &tokio::process::Child) -> anyhow::Result<()> {
        let raw_handle = child
            .raw_handle()
            .ok_or_else(|| anyhow::anyhow!("owned child process handle is unavailable"))?;
        self.assign_raw(raw_handle)
    }

    pub(crate) fn assign_std(&self, child: &std::process::Child) -> anyhow::Result<()> {
        self.assign_raw(child.as_raw_handle())
    }

    fn assign_raw(&self, raw_handle: std::os::windows::io::RawHandle) -> anyhow::Result<()> {
        let assigned = unsafe { AssignProcessToJobObject(self.job, raw_handle as HANDLE) };
        if assigned == 0 {
            anyhow::bail!("failed to assign owned child to Windows process containment job");
        }
        Ok(())
    }

    pub(crate) fn terminate(&self) -> anyhow::Result<()> {
        loop {
            match self.termination_state.load(Ordering::Acquire) {
                TERMINATION_COMPLETE => return Ok(()),
                TERMINATION_IDLE => {
                    if self
                        .termination_state
                        .compare_exchange(
                            TERMINATION_IDLE,
                            TERMINATION_IN_PROGRESS,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        break;
                    }
                }
                TERMINATION_IN_PROGRESS => std::hint::spin_loop(),
                _ => unreachable!("invalid process tree termination state"),
            }
        }

        if unsafe { TerminateJobObject(self.job, 1) } == 0 {
            self.termination_state
                .store(TERMINATION_IDLE, Ordering::Release);
            anyhow::bail!("failed to terminate Windows process containment job");
        }
        self.termination_state
            .store(TERMINATION_COMPLETE, Ordering::Release);
        Ok(())
    }
}

#[cfg(windows)]
const TERMINATION_IDLE: u8 = 0;
#[cfg(windows)]
const TERMINATION_IN_PROGRESS: u8 = 1;
#[cfg(windows)]
const TERMINATION_COMPLETE: u8 = 2;

#[cfg(windows)]
impl Drop for ProcessTreeGuard {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.job) };
    }
}

#[cfg(not(windows))]
pub(crate) struct ProcessTreeGuard;

#[cfg(not(windows))]
impl ProcessTreeGuard {
    pub(crate) fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub(crate) fn assign_tokio(&self, _child: &tokio::process::Child) -> anyhow::Result<()> {
        Ok(())
    }

    pub(crate) fn assign_std(&self, _child: &std::process::Child) -> anyhow::Result<()> {
        Ok(())
    }

    pub(crate) fn terminate(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{ProcessTreeGuard, TERMINATION_IDLE};
    use std::{
        fs,
        io::{BufRead, BufReader},
        process::{Command, Stdio},
        sync::atomic::AtomicU8,
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    fn sleeping_child() -> std::process::Child {
        Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
            .spawn()
            .expect("spawn inert child")
    }

    #[test]
    fn creates_a_job_object() {
        let _guard = ProcessTreeGuard::new().expect("create job object");
    }

    #[test]
    fn assigns_a_directly_owned_std_child() {
        let mut child = sleeping_child();
        let guard = ProcessTreeGuard::new().expect("create job object");

        guard.assign_std(&child).expect("assign owned child");
        guard.terminate().expect("terminate job");
        child.wait().expect("reap child");
    }

    #[test]
    fn terminates_a_descendant_created_after_assignment() {
        let signal = std::env::temp_dir().join(format!(
            "extractum-process-tree-{}-{}.signal",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let script = concat!(
            "while (-not (Test-Path -LiteralPath $env:EXTRACTUM_PROCESS_TREE_SIGNAL)) ",
            "{ Start-Sleep -Milliseconds 10 }; ",
            "$descendant = Start-Process -FilePath powershell.exe ",
            "-ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 30' -PassThru; ",
            "Write-Output $descendant.Id; Start-Sleep -Seconds 30"
        );
        let mut child = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .env("EXTRACTUM_PROCESS_TREE_SIGNAL", &signal)
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn signal-waiting fixture");
        let guard = ProcessTreeGuard::new().expect("create job object");
        guard.assign_std(&child).expect("assign fixture");

        fs::write(&signal, []).expect("signal descendant creation");
        let descendant_pid = {
            let stdout = child.stdout.take().expect("fixture stdout");
            let mut line = String::new();
            BufReader::new(stdout)
                .read_line(&mut line)
                .expect("read descendant pid");
            line.trim().parse::<u32>().expect("parse descendant pid")
        };

        guard.terminate().expect("terminate job");
        child.wait().expect("reap fixture");
        let _ = fs::remove_file(&signal);

        for _ in 0..30 {
            let status = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "if (Get-Process -Id {descendant_pid} -ErrorAction SilentlyContinue) {{ exit 1 }}"
                    ),
                ])
                .status()
                .expect("query descendant");
            if status.success() {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }

        panic!("post-assignment descendant survived job termination");
    }

    #[test]
    fn terminate_is_idempotent() {
        let mut child = sleeping_child();
        let guard = ProcessTreeGuard::new().expect("create job object");
        guard.assign_std(&child).expect("assign owned child");

        guard.terminate().expect("first termination");
        guard.terminate().expect("second termination");
        child.wait().expect("reap child");
    }

    #[test]
    fn terminate_failure_remains_reportable_and_retryable() {
        let guard = ProcessTreeGuard {
            job: std::ptr::null_mut(),
            termination_state: AtomicU8::new(TERMINATION_IDLE),
        };

        assert!(guard.terminate().is_err());
        assert!(guard.terminate().is_err());
    }

    #[test]
    fn dropping_the_guard_closes_the_job_and_kills_its_children() {
        let mut child = sleeping_child();
        {
            let guard = ProcessTreeGuard::new().expect("create job object");
            guard.assign_std(&child).expect("assign owned child");
        }

        child.wait().expect("job close kills child");
    }
}
