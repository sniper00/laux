use lib_core::context::CONTEXT;
use lib_lua::{
    self, cstr,
    ffi::{self, luaL_Reg},
    laux, lreg, lreg_null, luaL_newlib,
};
use std::ffi::c_int;

extern "C-unwind" fn num_alive_tasks(state: *mut ffi::lua_State) -> c_int {
    laux::lua_push(
        state,
        CONTEXT.tokio_runtime.metrics().num_alive_tasks() as i64,
    );
    1
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C-unwind" fn luaopen_rust_runtime(state: *mut ffi::lua_State) -> c_int {
    let l = [lreg!("num_alive_tasks", num_alive_tasks), lreg_null!()];
    luaL_newlib!(state, l);
    1
}
