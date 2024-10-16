pub mod lua_excel;
pub mod lua_http;

use lib_lua::{self, ffi};
use std::ffi::c_void;

pub type SendMessageFn =
    extern "C" fn(type_: u8, receiver: u32, session: i64, data: *const i8, len: usize);

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
