use anyhow::Result;

const APP_RUN_VALUE: &str = "RustGauge";

#[cfg(target_os = "windows")]
mod platform {
    use std::{env, ffi::OsStr, os::windows::ffi::OsStrExt, ptr};

    use anyhow::{bail, Context, Result};
    use windows_sys::Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Registry::{
            RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
            HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
        },
    };

    use super::APP_RUN_VALUE;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const ERROR_SUCCESS: u32 = 0;

    pub fn is_enabled() -> Result<bool> {
        let key = open_run_key(KEY_QUERY_VALUE)?;
        let value_name = wide(APP_RUN_VALUE);

        let result = unsafe {
            RegQueryValueExW(
                key.raw,
                value_name.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        match result {
            ERROR_SUCCESS => Ok(true),
            ERROR_FILE_NOT_FOUND => Ok(false),
            error => bail!("failed to query startup registry value: {error}"),
        }
    }

    pub fn set_enabled(enabled: bool) -> Result<()> {
        if enabled {
            enable()
        } else {
            disable()
        }
    }

    fn enable() -> Result<()> {
        let key = open_run_key(KEY_SET_VALUE)?;
        let value_name = wide(APP_RUN_VALUE);
        let command = startup_command()?;
        let command_wide = wide(&command);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                command_wide.as_ptr() as *const u8,
                command_wide.len() * std::mem::size_of::<u16>(),
            )
        };

        let result = unsafe {
            RegSetValueExW(
                key.raw,
                value_name.as_ptr(),
                0,
                REG_SZ,
                bytes.as_ptr(),
                bytes.len() as u32,
            )
        };

        if result != ERROR_SUCCESS {
            bail!("failed to write startup registry value: {result}");
        }

        Ok(())
    }

    fn disable() -> Result<()> {
        let key = open_run_key(KEY_SET_VALUE)?;
        let value_name = wide(APP_RUN_VALUE);
        let result = unsafe { RegDeleteValueW(key.raw, value_name.as_ptr()) };

        match result {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
            error => bail!("failed to delete startup registry value: {error}"),
        }
    }

    fn startup_command() -> Result<String> {
        let exe = env::current_exe().context("failed to resolve current executable path")?;
        Ok(format!("\"{}\"", exe.display()))
    }

    fn open_run_key(access: u32) -> Result<RegKey> {
        let subkey = wide(RUN_KEY);
        let mut raw: HKEY = ptr::null_mut();
        let result =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, access, &mut raw) };

        if result != ERROR_SUCCESS {
            bail!("failed to open startup registry key: {result}");
        }

        Ok(RegKey { raw })
    }

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }

    struct RegKey {
        raw: HKEY,
    }

    impl Drop for RegKey {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.raw);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::{bail, Result};

    pub fn is_enabled() -> Result<bool> {
        Ok(false)
    }

    pub fn set_enabled(_enabled: bool) -> Result<()> {
        bail!("startup registration is only supported on Windows")
    }
}

pub fn is_enabled() -> Result<bool> {
    platform::is_enabled()
}

pub fn set_enabled(enabled: bool) -> Result<()> {
    platform::set_enabled(enabled)
}
