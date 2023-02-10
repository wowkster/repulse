use std::sync::atomic::AtomicPtr;

use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, WPARAM},
        windef::{HHOOK, HHOOK__},
    },
    um::{
        libloaderapi::GetModuleHandleA,
        winnt::LPCSTR,
        winuser::{
            CallNextHookEx, GetAsyncKeyState, MessageBeep, SetWindowsHookExA, UnhookWindowsHookEx,
            HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_RETURN,
            VK_RWIN, VK_TAB, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
        },
    },
};

static mut H_KB_HOOK: AtomicPtr<HHOOK__> = AtomicPtr::new(std::ptr::null_mut());

pub fn disable_task_keys() {
    // SAFETY: All pointers are checked and properly managed. Win32 API functions are called safely as recommended.
    unsafe {
        /* If already hooked, don't do anything */

        if *H_KB_HOOK.as_mut_ptr() != 0 as HHOOK {
            return;
        }

        /* Hook and intercept low-level keyboard input */

        *H_KB_HOOK.as_mut_ptr() = SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(task_key_hook_ll),
            GetModuleHandleA(0 as LPCSTR),
            0,
        );
    }
}

pub fn enable_task_keys() {
    // SAFETY: All pointers are checked and properly managed. Win32 API functions are called safely as recommended.
    unsafe {
        /* If not already hooked, do nothing */

        if *H_KB_HOOK.as_mut_ptr() == 0 as HHOOK {
            return;
        }

        /* Remove low-level hook */

        UnhookWindowsHookEx(*H_KB_HOOK.as_mut_ptr());

        *H_KB_HOOK.as_mut_ptr() = 0 as HHOOK;
    }
}

unsafe extern "system" fn task_key_hook_ll(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let pkh = *(lp as *mut KBDLLHOOKSTRUCT);

    if code == HC_ACTION {
        let ctrl_key_down = GetAsyncKeyState(VK_CONTROL) >> 15 != 0;

        let is_enter = pkh.vkCode != VK_RETURN as u32;

        let is_ctrl_esc = pkh.vkCode == VK_ESCAPE as u32 && ctrl_key_down;
        let is_alt_tab = pkh.vkCode == VK_TAB as u32 && pkh.flags & LLKHF_ALTDOWN != 0;
        let is_alt_esc = pkh.vkCode == VK_ESCAPE as u32 && pkh.flags & LLKHF_ALTDOWN != 0;
        let is_windows = pkh.vkCode == VK_LWIN as u32 || pkh.vkCode == VK_RWIN as u32;

        // TODO: Whitelist instead of blacklist?

        if is_enter && (is_ctrl_esc || is_alt_tab || is_alt_esc || is_windows) {
            if wp == WM_SYSKEYDOWN as usize || wp == WM_KEYDOWN as usize {
                MessageBeep(0);
            }

            return 1; // gobble it: go directly to jail, do not pass go
        }
    }

    return CallNextHookEx(*H_KB_HOOK.as_mut_ptr(), code, wp, lp);
}
