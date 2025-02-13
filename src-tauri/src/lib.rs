// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::ffi::CString;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;
use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::VirtualAllocEx;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::EnumProcessModulesEx;
use winapi::um::psapi::GetModuleBaseNameA;
use winapi::um::psapi::LIST_MODULES_ALL;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE};
use winapi::um::winnt::{PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

fn get_process_handle(process_id: u32) -> Option<HANDLE> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id);
        if handle.is_null() {
            // println!(
            //     "Failed to open process. Error: {:?}",
            //     std::io::Error::last_os_error()
            // );
            None
        } else {
            // println!("Successfully opened process. Handle: {:?}", handle);
            Some(handle)
        }
    }
}

fn allocate_memory(handle: HANDLE, size: usize) -> Option<*mut u8> {
    unsafe {
        let address = VirtualAllocEx(
            handle,
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );
        if address.is_null() {
            // println!(
            //     "Failed to allocate memory. Error: {:?}",
            //     std::io::Error::last_os_error()
            // );
            None
        } else {
            // println!("Successfully allocated memory at: {:?}", address);
            Some(address as *mut u8)
        }
    }
}

#[tauri::command]
fn allocate_memory_command(process_id: u32, size: usize) -> Result<u64, String> {
    if let Some(handle) = get_process_handle(process_id) {
        if let Some(address) = allocate_memory(handle, size) {
            Ok(address as u64)
        } else {
            Err("Failed to allocate memory".to_string())
        }
    } else {
        Err("Failed to open process".to_string())
    }
}
#[tauri::command]
fn get_module_base_address(pid: u32, module_name: String) -> Result<usize, String> {
    unsafe {
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if process_handle.is_null() {
            return Err("Failed to open process".to_string());
        }

        let mut modules: [HMODULE; 1024] = [ptr::null_mut(); 1024];
        let mut cb_needed: DWORD = 0;
        let result = EnumProcessModulesEx(
            process_handle,
            modules.as_mut_ptr(),
            (modules.len() * std::mem::size_of::<HMODULE>()) as DWORD,
            &mut cb_needed,
            LIST_MODULES_ALL,
        );
        if result == 0 {
            return Err("Failed to enumerate process modules".to_string());
        }

        let module_count = (cb_needed / std::mem::size_of::<HMODULE>() as DWORD) as usize;

        for i in 0..module_count {
            let module_handle = modules[i];
            let mut module_name_buf = [0u8; 256];
            let result = GetModuleBaseNameA(
                process_handle,
                module_handle,
                module_name_buf.as_mut_ptr() as *mut i8,
                module_name_buf.len() as DWORD,
            );

            if result == 0 {
                continue;
            }

            let module_name_str =
                CString::from_vec_unchecked(module_name_buf[..result as usize].to_vec())
                    .into_string()
                    .map_err(|e| e.to_string())?;

            // println!("Found module: {}", module_name_str);

            if module_name_str == module_name {
                return Ok(module_handle as usize);
            }
        }

        Err("Module not found".to_string())
    }
}
#[tauri::command]
fn get_pid_by_process_name(process_name: &str) -> Option<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == null_mut() {
        eprintln!("Failed to create process snapshot");
        return None;
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    if unsafe { Process32FirstW(snapshot, &mut entry) } == 0 {
        eprintln!("Failed to get first process entry");
        unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
        return None;
    }

    loop {
        let name = OsString::from_wide(&entry.szExeFile)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_string();

        let file_name = Path::new(&name)
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("");
        // println!("Process: {}, PID: {}", file_name, entry.th32ProcessID);

        if file_name == process_name {
            unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
            return Some(entry.th32ProcessID);
        }

        if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
            break;
        }
    }

    unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
    None
}

#[tauri::command]
fn get_process_id(process_name: String) -> Option<u32> {
    get_pid_by_process_name(&process_name)
}
#[tauri::command]
fn read_memory(process_id: u32, address: usize, size: usize) -> Vec<u8> {
    unsafe {
        let process_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id);
        if process_handle.is_null() {
            return Vec::new();
        }

        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        if ReadProcessMemory(
            process_handle,
            address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            &mut bytes_read,
        ) != 0
        {
            CloseHandle(process_handle);
            buffer
        } else {
            CloseHandle(process_handle);
            Vec::new()
        }
    }
}

#[tauri::command]
fn write_memory(process_id: u32, address: usize, data: Vec<u8>) -> bool {
    unsafe {
        let process_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id);
        if process_handle.is_null() {
            return false;
        }

        let mut bytes_written = 0;

        let result = WriteProcessMemory(
            process_handle,
            address as *mut _,
            data.as_ptr() as *const _,
            data.len(),
            &mut bytes_written,
        );

        CloseHandle(process_handle);

        result != 0
    }
}

unsafe fn open_process(process_id: u32) -> *mut winapi::ctypes::c_void {
    let process_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id);
    if process_handle.is_null() {
        eprintln!("Failed to open process with ID: {}", process_id);
    }
    process_handle
}

unsafe fn read_pointer(
    process_handle: *mut winapi::ctypes::c_void,
    address: usize,
) -> Option<usize> {
    let mut buffer = vec![0u8; std::mem::size_of::<usize>()];
    let mut bytes_read = 0;

    if ReadProcessMemory(
        process_handle,
        address as *const _,
        buffer.as_mut_ptr() as *mut _,
        std::mem::size_of::<usize>(),
        &mut bytes_read,
    ) == 0
    {
        eprintln!("Failed to read memory at address: {:x}", address);
        return None;
    }

    Some(usize::from_ne_bytes(buffer.try_into().unwrap()))
}

#[tauri::command]
fn read_memory_with_offsets(
    process_id: u32,
    base_address: usize,
    offsets: Vec<usize>,
    size: usize,
) -> Vec<u8> {
    unsafe {
        let process_handle = open_process(process_id);
        if process_handle.is_null() {
            return Vec::new();
        }

        let mut current_address = base_address;

        for offset in offsets {
            let pointer_value = match read_pointer(process_handle, current_address) {
                Some(value) => value,
                None => {
                    CloseHandle(process_handle);
                    return Vec::new();
                }
            };

            current_address = match pointer_value.checked_add(offset) {
                Some(addr) => addr,
                None => {
                    eprintln!(
                        "Address overflow at base: {:x}, offset: {:x}",
                        pointer_value, offset
                    );
                    CloseHandle(process_handle);
                    return Vec::new();
                }
            };
        }

        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        if ReadProcessMemory(
            process_handle,
            current_address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            &mut bytes_read,
        ) != 0
        {
            CloseHandle(process_handle);
            buffer
        } else {
            eprintln!(
                "Failed to read memory at final address: {:x}",
                current_address
            );
            CloseHandle(process_handle);
            Vec::new()
        }
    }
}

#[tauri::command]
fn write_memory_with_offsets(
    process_id: u32,
    base_address: usize,
    offsets: Vec<usize>,
    data: Vec<u8>,
) -> bool {
    unsafe {
        let process_handle = open_process(process_id);
        if process_handle.is_null() {
            return false;
        }

        let mut current_address = base_address;

        for offset in offsets {
            let pointer_value = match read_pointer(process_handle, current_address) {
                Some(value) => value,
                None => {
                    CloseHandle(process_handle);
                    return false;
                }
            };

            current_address = match pointer_value.checked_add(offset) {
                Some(addr) => addr,
                None => {
                    eprintln!(
                        "Address overflow at base: {:x}, offset: {:x}",
                        pointer_value, offset
                    );
                    CloseHandle(process_handle);
                    return false;
                }
            };
        }

        let mut bytes_written = 0;

        let result = WriteProcessMemory(
            process_handle,
            current_address as *mut _,
            data.as_ptr() as *const _,
            data.len(),
            &mut bytes_written,
        );

        CloseHandle(process_handle);

        if result == 0 {
            eprintln!(
                "Failed to write memory at final address: {:x}",
                current_address
            );
        }

        result != 0
    }
}

#[tauri::command]
fn open_link_in_browser(url: String) {
    if let Err(e) = open::that(url) {
        eprintln!("Failed to open link: {}", e);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_memory,
            write_memory,
            read_memory_with_offsets,
            write_memory_with_offsets,
            get_process_id,
            get_module_base_address,
            allocate_memory_command,
            open_link_in_browser
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
