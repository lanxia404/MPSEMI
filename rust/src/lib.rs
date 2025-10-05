use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::ptr;

pub struct Engine {
    buf: String,
    cands: Vec<String>,
}

impl Engine {
    fn new() -> Self {
        Self {
            buf: String::new(),
            cands: vec![],
        }
    }
    fn process(&mut self, s: &str) -> bool {
        match s {
            "\n" | " " => {
                // 生成提交候選（最小示範：原樣或 upper）
                if self.buf.is_empty() {
                    return true;
                }
                self.cands = vec![self.buf.clone()];
            }
            _ => {
                self.buf.push_str(s);
                self.cands = vec![self.buf.clone()];
            }
        }
        true
    }
    fn preedit(&self) -> String {
        if self.buf.is_empty() {
            String::new()
        } else {
            self.buf.clone()
        }
    }
    fn commit(&mut self) -> String {
        let out = self.cands.first().cloned().unwrap_or_default();
        self.buf.clear();
        self.cands.clear();
        out
    }
}

// ---- C ABI ----
#[unsafe(no_mangle)]
/// # Safety
/// Caller must free the returned pointer with `mpsemi_engine_free`.
pub unsafe extern "C" fn mpsemi_engine_new() -> *mut c_void {
    Box::into_raw(Box::new(Engine::new())) as *mut c_void
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be null or a pointer previously created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_engine_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr as *mut Engine));
        }
    }
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be valid and `s` must point to a null-terminated UTF-8 string.
pub unsafe extern "C" fn mpsemi_process_utf8(ptr: *mut c_void, s: *const c_char) -> bool {
    if ptr.is_null() || s.is_null() {
        return false;
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    let s = unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string();
    eng.process(&s)
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_preedit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.preedit()).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_candidate_count(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    eng.cands.len() as u32
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_candidate_at(ptr: *mut c_void, idx: u32) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    match eng.cands.get(idx as usize) {
        Some(s) => CString::new(s.as_str()).unwrap_or_default().into_raw(),
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_commit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.commit()).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
/// # Safety
/// `s` must be a pointer obtained from this library and not yet freed.
pub unsafe extern "C" fn mpsemi_free_cstr(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
// ---- End of C ABI ----
