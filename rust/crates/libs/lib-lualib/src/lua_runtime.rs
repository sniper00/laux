use lib_core::context::CONTEXT;
use lib_lua::{self, cstr, ffi, ffi::luaL_Reg, laux, lreg, lreg_null};
use std::ffi::c_int;

extern "C-unwind" fn num_alive_tasks(state: *mut ffi::lua_State) -> c_int {
    if let Some(runtime) = CONTEXT.get_tokio_runtime().as_ref() {
        laux::lua_push(state, runtime.metrics().num_alive_tasks() as i64);
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C-unwind" fn luaopen_rust_runtime(state: *mut ffi::lua_State) -> c_int {
    let l = [lreg!("num_alive_tasks", num_alive_tasks), lreg_null!()];

    ffi::lua_createtable(state, 0, l.len() as c_int);
    ffi::luaL_setfuncs(state, l.as_ptr(), 0);
    1
}
