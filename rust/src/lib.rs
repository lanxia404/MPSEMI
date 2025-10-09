use std::ffi::{c_void, CStr, CString};
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
                if self.buf.pop().is_some() {
                    self.generate_candidates();
                    return true;
                }
                if !self.cands.is_empty() {
                    self.cands.clear();
                }
                return false;
            }
            "\u{1b}" => {
                self.buf.clear();
                self.cands.clear();
                return true;
            }
            "\n" | " " => {
                if self.buf.is_empty() {
                    return false;
                }
                self.generate_candidates();
            }
            _ => {
                self.buf.push_str(s);
                self.generate_candidates();
            }
        }
        true
    }
    fn generate_candidates(&mut self) {
        if self.buf.is_empty() {
            self.cands.clear();
            return;
        }
        let mut candidates = vec![self.buf.clone()];
        let upper = self.buf.to_uppercase();
        if upper != self.buf {
            candidates.push(upper);
        }
        self.cands = candidates;
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
    fn adjust_selection(&mut self, offset: i32) -> bool {
        if self.cands.is_empty() {
            return false;
        }
        let len = self.cands.len() as i32;
        let shift = offset.rem_euclid(len);
        if shift == 0 {
            return false;
        }
        self.cands.rotate_left(shift as usize);
        true
    }

    #[cfg(test)]
    fn candidates(&self) -> &[String] {
        &self.cands
    }
}

// ---- C ABI ----
#[no_mangle]
/// # Safety
/// Caller must free the returned pointer with `mpsemi_engine_free`.
pub unsafe extern "C" fn mpsemi_engine_new() -> *mut c_void {
    Box::into_raw(Box::new(Engine::new())) as *mut c_void
}

#[no_mangle]
/// # Safety
/// `ptr` must be null or a pointer previously created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_engine_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr as *mut Engine));
        }
    }
}

#[no_mangle]
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

#[no_mangle]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_preedit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.preedit()).unwrap_or_default().into_raw()
}

#[no_mangle]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_candidate_count(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    eng.cands.len() as u32
}

#[no_mangle]
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

#[no_mangle]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_commit(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    CString::new(eng.commit()).unwrap_or_default().into_raw()
}

#[no_mangle]
/// # Safety
/// `ptr` must point to an engine created by `mpsemi_engine_new`.
pub unsafe extern "C" fn mpsemi_adjust_selection(ptr: *mut c_void, offset: i32) -> bool {
    if ptr.is_null() {
        return false;
    }
    let eng = unsafe { &mut *(ptr as *mut Engine) };
    eng.adjust_selection(offset)
}

#[no_mangle]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn typing_appends_buffer_and_candidate() {
        let mut eng = Engine::new();
        assert!(eng.process("a"));
        assert_eq!(eng.preedit(), "a");
        assert_eq!(eng.candidates(), ["a", "A"]);

        assert!(eng.process("b"));
        assert_eq!(eng.preedit(), "ab");
        assert_eq!(eng.candidates(), ["ab", "AB"]);
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
        assert_eq!(eng.candidates(), ["hi", "HI"]);
    }

    #[test]
    fn backspace_removes_last_character() {
        let mut eng = Engine::new();
        eng.process("a");
        eng.process("b");
        assert!(eng.process("\u{08}"));
        assert_eq!(eng.preedit(), "a");
        assert_eq!(eng.candidates(), ["a", "A"]);

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
    fn adjust_selection_rotates_candidates() {
        let mut eng = Engine::new();
        eng.process("hi");
        eng.process(" ");
        assert_eq!(eng.candidates(), ["hi", "HI"]);
        assert!(eng.adjust_selection(1));
        assert_eq!(eng.candidates(), ["HI", "hi"]);
        assert!(eng.adjust_selection(-1));
        assert_eq!(eng.candidates(), ["hi", "HI"]);
    }

    #[test]
    fn adjust_selection_ignores_empty_candidates() {
        let mut eng = Engine::new();
        assert!(!eng.adjust_selection(1));
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
    fn ffi_roundtrip_process_commit_and_selection() {
        unsafe {
            let engine = mpsemi_engine_new();
            assert!(!engine.is_null());

            let input = CString::new("hi").unwrap();
            assert!(mpsemi_process_utf8(engine, input.as_ptr()));
            let blank = CString::new(" ").unwrap();
            assert!(mpsemi_process_utf8(engine, blank.as_ptr()));

            assert_eq!(mpsemi_candidate_count(engine), 2);

            // rotate selection forward
            assert!(mpsemi_adjust_selection(engine, 1));
            let cand = mpsemi_candidate_at(engine, 0);
            let candidate = CStr::from_ptr(cand).to_string_lossy().to_string();
            mpsemi_free_cstr(cand);
            assert_eq!(candidate, "HI");

            // commit the rotated candidate resets state
            let committed_ptr = mpsemi_commit(engine);
            let committed = CStr::from_ptr(committed_ptr).to_string_lossy().to_string();
            mpsemi_free_cstr(committed_ptr);
            assert_eq!(committed, "HI");

            mpsemi_engine_free(engine);
        }
    }
}
