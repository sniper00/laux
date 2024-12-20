pub mod lua_excel;
pub mod lua_http;
// pub mod lua_opendal;
pub mod lua_sqlx;
pub mod lua_json;
pub mod lua_runtime;

use lib_lua::{self, ffi};
use std::ffi::c_void;

pub type SendMessageFn =
    extern "C" fn(type_: u8, receiver: u32, session: i64, data: *const i8, len: usize);

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn get_send_message_fn(state: *mut ffi::lua_State, index: i32) -> Option<SendMessageFn> {
    unsafe {
        let p = ffi::lua_touserdata(state, index);
        if p.is_null() {
            return None;
            //ffi::luaL_error(state, cstr!("Invalid send_message function pointer"));
        }
        let send_message_fn = std::mem::transmute::<*mut c_void, SendMessageFn>(p);
        Some(send_message_fn)
    }
}

pub fn moon_send<T>(
    protocol_type: u8,
    owner: u32,
    session: i64,
    callback: &SendMessageFn,
    res: T,
) {
    if session == 0 {
        return;
    }
    let ptr = Box::into_raw(Box::new(res));
    let bytes = (ptr as isize).to_ne_bytes();

    callback(
        protocol_type,
        owner,
        session,
        bytes.as_ptr() as *const i8,
        bytes.len(),
    );
}

pub const PTYPE_ERROR: u8 = 4;
pub const PTYPE_LOG: u8 = 13;

pub const LOG_LEVEL_ERROR: u8 = 1;
pub const LOG_LEVEL_WARN: u8 = 2;
pub const LOG_LEVEL_INFO: u8 = 3;
pub const LOG_LEVEL_DEBUG: u8 = 4;

pub fn moon_log(owner: u32, callback: SendMessageFn, log_level: u8, data: String) {
    let message = format!("{}{}", log_level, data);
    callback(
        PTYPE_LOG,
        owner,
        0,
        message.as_ptr() as *const i8,
        message.len(),
    );
}
