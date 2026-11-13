//! Running a command so that stopping it stops everything it started.

use std::fs::File;
use std::io::{Read, Seek};
use std::process::{ChildStdin, Command, ExitStatus, Output, Stdio};
use std::time::Duration;

use process_wrap::std::{ChildWrapper, CommandWrap};

/// Puts `command` in a process group of its own on Unix, so [`Tree::kill`]
/// reaches everything it starts. Does nothing elsewhere.
#[cfg(unix)]
pub fn own_group(command: &mut Command) -> &mut Command {
    use std::os::unix::process::CommandExt;
    command.process_group(0)
}

/// Puts `command` in a process group of its own on Unix, so [`Tree::kill`]
/// reaches everything it starts. Does nothing elsewhere.
#[cfg(not(unix))]
pub fn own_group(command: &mut Command) -> &mut Command {
    command
}

/// A running command and everything it started: its process group on Unix,
/// when [`own_group`] gave it one, and its job object on Windows.
pub struct Tree(Box<dyn ChildWrapper>);

/// Starts `command`. On Windows it runs in a job object of its own and
/// inherits none of this process's standard handles, only the ones `command`
/// names.
pub fn spawn(command: Command) -> std::io::Result<Tree> {
    let mut command = CommandWrap::from(command);
    #[cfg(windows)]
    {
        keep_own_stdio_out_of_children();
        command.wrap(process_wrap::std::JobObject);
    }
    command.spawn().map(Tree)
}

#[cfg(windows)]
fn keep_own_stdio_out_of_children() {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};

    for handle in [
        std::io::stdin().as_raw_handle(),
        std::io::stdout().as_raw_handle(),
        std::io::stderr().as_raw_handle(),
    ] {
        unsafe {
            SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
        }
    }
}

impl Tree {
    /// Takes the command's stdin, when it was piped.
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.0.stdin().take()
    }

    /// The command's exit status, once it has exited. What it left running
    /// is not waited for.
    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.0.try_wait()
    }

    /// Kills the command with everything it started, and reaps the command.
    pub fn kill(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0.id() as libc::pid_t), libc::SIGKILL);
        }
        let _ = self.0.start_kill();
        let _ = self.0.inner_mut().wait();
    }
}

/// Runs `command` like [`Command::output`] without holding a thread.
/// Dropping the future kills the command as [`Tree::kill`] does.
pub async fn output(mut command: Command) -> std::io::Result<Output> {
    let stdout = tempfile::tempfile()?;
    let stderr = tempfile::tempfile()?;
    command
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?);
    let status = status(command).await?;
    Ok(Output {
        status,
        stdout: read_from_start(stdout)?,
        stderr: read_from_start(stderr)?,
    })
}

/// Runs `command` like [`Command::status`] without holding a thread.
/// Dropping the future kills the command as [`Tree::kill`] does.
pub async fn status(command: Command) -> std::io::Result<ExitStatus> {
    let mut running = Running {
        tree: spawn(command)?,
        exited: false,
    };
    loop {
        if let Some(status) = running.tree.try_wait()? {
            running.exited = true;
            return Ok(status);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

struct Running {
    tree: Tree,
    exited: bool,
}

impl Drop for Running {
    fn drop(&mut self) {
        if !self.exited {
            self.tree.kill();
        }
    }
}

fn read_from_start(mut file: File) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.rewind()?;
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
