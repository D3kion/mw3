//! Windows `CreateProcessA` (suspended) + `VirtualProtectEx` + `WriteProcessMemory` + `ResumeThread`.

use std::ffi::CString;
use std::path::Path;

use thiserror::Error;
use windows::core::{PCSTR, PSTR};
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Memory::{
    VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::Threading::{
    CreateMutexA, CreateProcessA, ResumeThread, TerminateProcess, WaitForSingleObject,
    CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOA, WAIT_OBJECT_0,
};

use crate::dll_name_bytes;
use crate::locate_steam_api_dll_va;

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("PE locate: {0}")]
    Pe(#[from] super::PeLocateError),
    #[error("{0}")]
    Message(String),
    #[error("Windows API error code {0}")]
    Win(u32),
}

fn last_win() -> u32 {
    windows::Win32::Foundation::GetLastError().0
}

/// Inputs for [`launch_suspended_patch_resume`].
pub struct LaunchOptions<'a> {
    pub exe_path: &'a Path,
    /// Extra arguments after the quoted executable (e.g. `+dedicated 1 +set net_port 27015`).
    pub game_args: &'a str,
    pub dll_name: &'a str,
}

/// Create the process suspended, patch the DLL import string, create the Tekno mutex, resume, then wait for exit.
pub fn launch_suspended_patch_resume(opt: LaunchOptions<'_>) -> Result<u32, LaunchError> {
    let write_va = locate_steam_api_dll_va(opt.exe_path)?;
    let dll_buf = dll_name_bytes(opt.dll_name).map_err(LaunchError::Message)?;
    let exe_str = opt
        .exe_path
        .to_str()
        .ok_or_else(|| LaunchError::Message("exe path is not valid UTF-8".into()))?;

    let exe_c = CString::new(exe_str).map_err(|e| LaunchError::Message(e.to_string()))?;
    let cmd = format!("\"{exe_str}\" {}", opt.game_args.trim());
    let mut cmd_c = CString::new(cmd).map_err(|e| LaunchError::Message(e.to_string()))?;

    let mut si = STARTUPINFOA {
        cb: std::mem::size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    let ok = unsafe {
        CreateProcessA(
            PCSTR(exe_c.as_ptr().cast()),
            PSTR(cmd_c.as_mut_ptr().cast()),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &si,
            &mut pi,
        )
    };
    if !ok.as_bool() {
        return Err(LaunchError::Win(last_win()));
    }

    let process = pi.hProcess;
    let thread = pi.hThread;

    let patch = (|| -> Result<(), LaunchError> {
        let mut old_prot = PAGE_PROTECTION_FLAGS(0);
        let ok = unsafe {
            VirtualProtectEx(
                process,
                write_va as *const std::ffi::c_void,
                dll_buf.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_prot,
            )
        };
        if !ok.as_bool() {
            return Err(LaunchError::Win(last_win()));
        }

        let mut written = 0usize;
        let ok = unsafe {
            WriteProcessMemory(
                process,
                write_va as *const std::ffi::c_void,
                dll_buf.as_ptr().cast(),
                dll_buf.len(),
                Some(&mut written),
            )
        };
        if !ok.as_bool() || written != dll_buf.len() {
            return Err(LaunchError::Win(last_win()));
        }

        let mut ignored = PAGE_PROTECTION_FLAGS(0);
        let _ = unsafe {
            VirtualProtectEx(
                process,
                write_va as *const std::ffi::c_void,
                dll_buf.len(),
                old_prot,
                &mut ignored,
            )
        };

        let pid = unsafe { pi.dwProcessId };
        let mtx_name = format!("TeknoMW3{:08X}", pid ^ 0x57);
        let mtx_c = CString::new(mtx_name).map_err(|e| LaunchError::Message(e.to_string()))?;
        let mtx = unsafe { CreateMutexA(None, BOOL::from(true), PCSTR(mtx_c.as_ptr().cast())) };
        if mtx.is_invalid() {
            return Err(LaunchError::Win(last_win()));
        }
        let _mtx_close = HandleClose(mtx);

        let resume = unsafe { ResumeThread(thread) };
        if resume == u32::MAX {
            return Err(LaunchError::Win(last_win()));
        }
        Ok(())
    })();

    if let Err(e) = patch {
        unsafe {
            let _ = TerminateProcess(process, 1);
            let _ = CloseHandle(thread);
            let _ = CloseHandle(process);
        }
        return Err(e);
    }

    let wait = unsafe { WaitForSingleObject(process, windows::Win32::System::Threading::INFINITE) };
    let mut exit_code = 0u32;
    let _ =
        unsafe { windows::Win32::System::Threading::GetExitCodeProcess(process, &mut exit_code) };

    unsafe {
        let _ = CloseHandle(thread);
        let _ = CloseHandle(process);
    }

    if wait != WAIT_OBJECT_0 {
        return Err(LaunchError::Win(last_win()));
    }

    Ok(exit_code)
}

struct HandleClose(HANDLE);

impl Drop for HandleClose {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
