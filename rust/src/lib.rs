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
            "\u{08}" => {
                // Backspace
                if self.buf.pop().is_some() {
                    self.cands = if self.buf.is_empty() {
                        vec![]
                    } else {
                        vec![self.buf.clone()]
                    };
                }
                return true;
            }
            "\u{1b}" => {
                // Esc
                self.buf.clear();
                self.cands.clear();
                return true;
            }
            "\n" | " " => {
                if self.buf.is_empty() {
                    return false;
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
            "".into()
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

    #[cfg(test)]
    fn candidates(&self) -> &[String] {
        &self.cands
    }
}

// ---- C ABI ----
#[unsafe(no_mangle)]
/// # Safety
/// Caller must free the returned pointer with `mpsemi_engine_free` when done.
pub unsafe extern "C" fn mpsemi_engine_new() -> *mut c_void {
    Box::into_raw(Box::new(Engine::new())) as *mut c_void
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be null or a pointer created by `mpsemi_engine_new`.
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
/// `ptr` must be a valid engine pointer created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_preedit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.preedit()).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a valid engine pointer created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_candidate_count(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    eng.cands.len() as u32
}

#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a valid engine pointer created by `mpsemi_engine_new`.
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
/// `ptr` must be a valid engine pointer created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_commit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.commit()).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
/// # Safety
/// `s` must have been allocated by this library and not already freed.
pub unsafe extern "C" fn mpsemi_free_cstr(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
// ---- End of C ABI ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_appends_buffer_and_candidate() {
        let mut eng = Engine::new();
        assert!(eng.process("a"));
        assert_eq!(eng.preedit(), "a");
        assert_eq!(eng.candidates(), ["a"]);

        assert!(eng.process("b"));
        assert_eq!(eng.preedit(), "ab");
        assert_eq!(eng.candidates(), ["ab"]);
    }

    #[test]
    fn space_with_empty_buffer_does_not_consume_state() {
        let mut eng = Engine::new();
        assert!(!eng.process(" "));
        assert_eq!(eng.preedit(), "");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn space_after_text_keeps_candidate_ready() {
        let mut eng = Engine::new();
        eng.process("hi");
        assert!(eng.process(" "));
        assert_eq!(eng.preedit(), "hi");
        assert_eq!(eng.candidates(), ["hi"]);
    }

    #[test]
    fn backspace_removes_last_character() {
        let mut eng = Engine::new();
        eng.process("a");
        eng.process("b");
        assert!(eng.process("\u{08}"));
        assert_eq!(eng.preedit(), "a");
        assert_eq!(eng.candidates(), ["a"]);

        assert!(eng.process("\u{08}"));
        assert_eq!(eng.preedit(), "");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn escape_clears_buffer_and_candidates() {
        let mut eng = Engine::new();
        eng.process("test");
        assert!(eng.process("\u{1b}"));
        assert_eq!(eng.preedit(), "");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn commit_returns_candidate_and_clears_state() {
        let mut eng = Engine::new();
        eng.process("ok");
        let committed = eng.commit();
        assert_eq!(committed, "ok");
        assert_eq!(eng.preedit(), "");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn commit_without_candidate_returns_empty() {
        let mut eng = Engine::new();
        let committed = eng.commit();
        assert_eq!(committed, "");
        assert_eq!(eng.preedit(), "");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn candidate_count_and_access_align() {
        let mut eng = Engine::new();
        assert!(eng.candidates().is_empty());
        eng.process("ha");
        eng.process(" ");
        assert_eq!(eng.candidates(), ["ha"]);
        assert_eq!(eng.candidates().first().unwrap(), "ha");
    }

    #[test]
    fn ffi_roundtrip_process_commit() {
        unsafe {
            let engine = mpsemi_engine_new();
            assert!(!engine.is_null());

            assert_eq!(mpsemi_candidate_count(engine), 0);

            let input = CString::new("hi").unwrap();
            assert!(mpsemi_process_utf8(engine, input.as_ptr()));

            let blank = CString::new(" ").unwrap();
            assert!(mpsemi_process_utf8(engine, blank.as_ptr()));

            let pre = mpsemi_preedit(engine);
            assert!(!pre.is_null());
            let preedit = CStr::from_ptr(pre).to_string_lossy().to_string();
            mpsemi_free_cstr(pre);
            assert_eq!(preedit, "hi");

            assert_eq!(mpsemi_candidate_count(engine), 1);
            let cand = mpsemi_candidate_at(engine, 0);
            let candidate = CStr::from_ptr(cand).to_string_lossy().to_string();
            mpsemi_free_cstr(cand);
            assert_eq!(candidate, "hi");

            let committed_ptr = mpsemi_commit(engine);
            let committed = CStr::from_ptr(committed_ptr).to_string_lossy().to_string();
            mpsemi_free_cstr(committed_ptr);
            assert_eq!(committed, "hi");

            mpsemi_engine_free(engine);
        }
    }
}
