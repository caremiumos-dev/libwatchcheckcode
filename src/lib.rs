use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_void};
use std::ptr;
use std::sync::Mutex;

type JObject = *mut c_void;
type JString = *mut c_void;
type JBoolean = u8;
type JInt = i32;

const JNI_FALSE: JBoolean = 0;
const JNI_TRUE: JBoolean = 1;

const NEW_STRING_UTF: usize = 167;
const GET_STRING_UTF_CHARS: usize = 169;
const RELEASE_STRING_UTF_CHARS: usize = 170;

type NewStringUtf = unsafe extern "system" fn(*mut JNIEnv, *const c_char) -> JString;
type GetStringUtfChars =
    unsafe extern "system" fn(*mut JNIEnv, JString, *mut JBoolean) -> *const c_char;
type ReleaseStringUtfChars =
    unsafe extern "system" fn(*mut JNIEnv, JString, *const c_char);

#[repr(C)]
pub struct JNIEnv {
    functions: *const *const c_void,
}

extern "C" {
    fn srand(seed: c_uint);
    fn rand() -> c_int;
    fn clock() -> c_long;
}

struct State {
    random: String,
    code: String,
    count: [i32; 10],
}

static STATE: Mutex<State> = Mutex::new(State {
    random: String::new(),
    code: String::new(),
    count: [0; 10],
});

unsafe fn jni_fn<T>(env: *mut JNIEnv, index: usize) -> T {
    let table = (*env).functions;
    let slot = *table.add(index);
    std::mem::transmute_copy::<*const c_void, T>(&slot)
}

fn is_acceptable(a: &[i32; 8]) -> bool {
    let mut histogram = [0i32; 10];
    for &value in a.iter() {
        let index = value as usize;
        histogram[index] += 1;
        if histogram[index] >= 4 {
            return false;
        }
    }
    true
}

fn gen_random() -> String {
    let mut a = [0i32; 8];
    loop {
        unsafe {
            srand(clock() as c_uint);
        }
        for i in 0..7 {
            a[i] = unsafe { rand() } % 10;
        }
        a[7] = unsafe { rand() } % 7;
        if is_acceptable(&a) {
            break;
        }
    }

    let mut result = String::new();
    for i in 0..8 {
        result.push_str(&a[i].to_string());
    }
    result
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_xtc_utils_WatchCheckCode_generateCheckCode(
    env: *mut JNIEnv,
    _this: JObject,
    type_: JInt,
) -> JString {
    if type_ != 1 && type_ != 2 {
        return ptr::null_mut();
    }

    let code = {
        let mut state = STATE.lock().unwrap();
        state.random = gen_random();

        let mut a = [0i32; 8];
        for i in 0..8 {
            a[i] = (state.random.as_bytes()[i] - b'0') as i32;
        }

        let x = a[7] as usize;
        let value = a[x];

        state.count = [0; 10];
        let mut code = String::new();
        for i in 0..7 {
            let offset = if i == x { x as i32 } else { value };
            let digit = (a[i] + offset) % 10;
            state.count[digit as usize] += 1;
            code.push_str(&digit.to_string());
        }

        let tail = (x as i32) ^ type_;
        state.count[tail as usize] += 1;
        code.push_str(&tail.to_string());
        state.code = code;
        state.code.clone()
    };

    let c_code = CString::new(code).unwrap();
    let new_string_utf: NewStringUtf = jni_fn(env, NEW_STRING_UTF);
    new_string_utf(env, c_code.as_ptr())
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_xtc_utils_WatchCheckCode_check(
    env: *mut JNIEnv,
    _this: JObject,
    input: JString,
    type_: JInt,
) -> JBoolean {
    if type_ != 1 && type_ != 2 {
        return JNI_FALSE;
    }
    if input.is_null() {
        return JNI_FALSE;
    }

    let get_string_utf_chars: GetStringUtfChars = jni_fn(env, GET_STRING_UTF_CHARS);
    let release_string_utf_chars: ReleaseStringUtfChars =
        jni_fn(env, RELEASE_STRING_UTF_CHARS);

    let raw = get_string_utf_chars(env, input, ptr::null_mut());
    let bytes = if raw.is_null() {
        Vec::new()
    } else {
        CStr::from_ptr(raw).to_bytes().to_vec()
    };
    release_string_utf_chars(env, input, raw);

    if bytes.len() != 8 {
        return JNI_FALSE;
    }

    let mut d = [0i32; 8];
    for i in 0..8 {
        let byte = bytes[i];
        if !byte.is_ascii_digit() {
            return JNI_FALSE;
        }
        d[i] = (byte - b'0') as i32;
    }

    let state = STATE.lock().unwrap();

    let mut histogram = [0i32; 10];
    for i in 0..8 {
        histogram[d[i] as usize] += 1;
    }

    let mut same = true;
    for k in 0..10 {
        if histogram[k] != state.count[k] {
            same = false;
            break;
        }
    }
    if same {
        return JNI_FALSE;
    }

    let target: Vec<u8> = state
        .random
        .as_bytes()
        .iter()
        .take(7)
        .copied()
        .collect();

    for i in 0..8 {
        let k = d[i] ^ type_;
        if k >= 7 {
            continue;
        }
        let k = k as usize;

        let mut smaller = [0i32; 7];
        let mut n = 0;
        for j in 0..8 {
            if j != i {
                smaller[n] = d[j];
                n += 1;
            }
        }

        let pivot = (smaller[k] - k as i32 + 10) % 10;

        let mut built = Vec::with_capacity(7);
        for j in 0..7 {
            let digit = if j == k {
                pivot
            } else {
                (smaller[j] - pivot + 10) % 10
            };
            built.push(digit as u8 + b'0');
        }

        if built == target {
            return JNI_TRUE;
        }
    }

    JNI_FALSE
}
