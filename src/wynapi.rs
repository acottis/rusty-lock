//! This is our user abstraction that turns windows API into
//! more of a rust friendly interface
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::MaybeUninit;
use core::ptr::{null, null_mut};
use tracing::{event, Level};

type HANDLE = *mut c_void;
type LPVOID = *mut c_void;
type HWND = HANDLE;
type HMENU = HANDLE;
type HINSTANCE = HANDLE;
type HMODULE = HANDLE;
type DWORD = i32;
type CHAR = c_char;
type LPCSTR = *const CHAR;
type INT = c_int;
type UINT = u32;
type LRESULT = isize;
type ATOM = u16;
type HICON = HANDLE;
type HCURSOR = HICON;
type HBRUSH = HANDLE;
type UINT_PTR = usize;
type WPARAM = UINT_PTR;
type LONG_PTR = isize;
type LPARAM = LONG_PTR;
type LONG = i32;

type WNDPROC = Option<
    unsafe extern "system" fn(
        hwnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT,
>;

const NOTIFY_FOR_THIS_SESSION: DWORD = 0;
const NOTIFY_FOR_ALL_SESSIONS: DWORD = 1;
const HWND_MESSAGE: HWND = -3isize as HWND;

#[derive(Debug)]
pub enum WtsState {
    ConsoleConnect,
    ConsoleDisconnect,
    RemoteConnect,
    RemoteDisconnnect,
    Logon,
    Logoff,
    Lock,
    Unlock,
    RemoteControl,
}

impl TryFrom<usize> for WtsState {
    type Error = ();

    fn try_from(lparam: usize) -> Result<Self, Self::Error> {
        match lparam {
            0x1 => Ok(Self::ConsoleConnect),
            0x2 => Ok(Self::ConsoleDisconnect),
            0x3 => Ok(Self::RemoteConnect),
            0x4 => Ok(Self::RemoteDisconnnect),
            0x5 => Ok(Self::Logon),
            0x6 => Ok(Self::Logoff),
            0x7 => Ok(Self::Lock),
            0x8 => Ok(Self::Unlock),
            0x9 => Ok(Self::RemoteControl),
            _ => {
                event!(Level::ERROR, "{lparam} is not a valid WtsState");
                Err(())
            }
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct WNDCLASSEXA {
    cbSize: UINT,
    style: UINT,
    lpfnWndProc: WNDPROC,
    cbClsExtra: c_int,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: LPCSTR,
    lpszClassName: LPCSTR,
    hIconSm: HICON,
}

#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINT,
    lPrivate: DWORD,
}

#[repr(C)]
#[derive(Debug)]
struct POINT {
    x: LONG,
    y: LONG,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Error {
    PROC_NOT_FOUND,
    NOACCESS,
    INVALID_PARAMETER,
    NOT_SUPPORTED,
    INVALID_HANDLE,
    ERROR_CANNOT_FIND_WND_CLASS,
    ERROR_WINDOW_OF_OTHER_THREAD,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::write(f, format_args!("{:?}", self))
    }
}

impl std::error::Error for self::Error {}

impl Error {
    /// Rust wrapper around GetLastError()
    pub fn get_last() -> Self {
        let err = unsafe { GetLastError() };
        match err {
            5 => Self::NOT_SUPPORTED,
            6 => Self::INVALID_HANDLE,
            87 => Self::INVALID_PARAMETER,
            127 => Self::PROC_NOT_FOUND,
            998 => Self::NOACCESS,
            1407 => Self::ERROR_CANNOT_FIND_WND_CLASS,
            1408 => Self::ERROR_WINDOW_OF_OTHER_THREAD,
            _ => unimplemented!("GetLastError code: {err} not yet handled"),
        }
    }
}

#[link(name = "Wtsapi32")]
extern "system" {
    fn WTSRegisterSessionNotification(hWnd: HWND, dwFlags: DWORD) -> bool;
    fn WTSUnRegisterSessionNotification(hWnd: HWND);
}

#[link(name = "Kernel32")]
extern "system" {
    fn GetLastError() -> DWORD;
    fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE;
}

#[link(name = "User32")]
extern "system" {
    fn RegisterClassExA(unnamedParam1: WNDCLASSEXA) -> ATOM;
    fn DefWindowProcA(
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
    fn CreateWindowExA(
        dwExStyle: DWORD,
        lpClassName: LPCSTR,
        lpWindowName: LPCSTR,
        dwStyle: DWORD,
        X: INT,
        Y: INT,
        nWidth: INT,
        nHeight: INT,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn GetMessageA(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> bool;
}

// Rust wrapper for GetModuleHandleA
pub fn get_module_handle_a() -> HANDLE {
    unsafe { GetModuleHandleA(null()) }
}

// Rust wrapper for RegisterClassExA
fn register_class_ex_a(window_class: WNDCLASSEXA) -> Option<ATOM> {
    let res = unsafe { RegisterClassExA(window_class) };

    if res == 0 {
        event!(Level::ERROR, "RegisterClassExA {}", Error::get_last());
        return None;
    }
    event!(Level::INFO, "RegisterClassExA {}", res);
    Some(res)
}

// Rust wrapper for CreateWindowExA
pub fn create_window_ex_a() -> Option<HWND> {
    let class_name = "rustylock\0".as_ptr() as *const i8;
    let h_instance = get_module_handle_a();

    let window_class = WNDCLASSEXA {
        cbSize: core::mem::size_of::<WNDCLASSEXA>() as u32,
        style: 0,
        lpfnWndProc: Some(DefWindowProcA),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: class_name,
        hIconSm: null_mut(),
    };
    register_class_ex_a(window_class).expect("Cannot Register Class");

    let handle = unsafe {
        CreateWindowExA(
            0,
            class_name,
            "rusty-lock\0".as_ptr() as *const i8,
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            h_instance,
            null_mut(),
        )
    };
    if handle.is_null() {
        event!(Level::ERROR, "CreateWindowExA {}", Error::get_last());
        return None;
    }
    event!(Level::INFO, "CreateWindowExA handle: {:?}", handle);
    Some(handle)
}

// Rust wrapper for WTSRegisterSessionNotification
pub fn wts_register_session_notification(handle: HWND) -> Option<()> {
    let res = unsafe {
        WTSRegisterSessionNotification(handle, NOTIFY_FOR_THIS_SESSION)
    };
    if res == false {
        event!(
            Level::ERROR,
            "WTSRegisterSessionNotification {}",
            Error::get_last()
        );
        return None;
    }
    event!(Level::INFO, "WTSRegisterSessionNotification Registered");
    Some(())
}
// Rust wrapper for GetMessageA
pub fn get_message_a(handle: HWND) -> Option<WtsState> {
    let mut msg: MaybeUninit<MSG> = MaybeUninit::uninit();
    let res = unsafe { GetMessageA(msg.as_mut_ptr(), handle, 0, 0) };
    if res == false {
        event!(Level::ERROR, "GetMessageA {}", Error::get_last());
        return None;
    }
    // We assume msg has data because result was not false
    let msg = unsafe { msg.assume_init() };

    event!(Level::INFO, "Message {:?}", msg);

    // Convert to Rust Enum
    let state: Option<WtsState> = msg.wParam.try_into().ok();
    state
}

// Rust wrapper for WTSUnRegisterSessionNotification
pub fn wts_unregister_session_notification(handle: HWND) {
    unsafe { WTSUnRegisterSessionNotification(handle) };
    event!(Level::INFO, "WTSRegisterSessionNotification Unregistered");
}
