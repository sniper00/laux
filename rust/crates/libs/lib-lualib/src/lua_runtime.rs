use lib_core::context::CONTEXT;
use lib_lua::{self, cstr, ffi, ffi::luaL_Reg, laux, lreg, lreg_null};
use std::ffi::c_int;

extern "C-unwind" fn num_alive_tasks(state: *mut ffi::lua_State) -> c_int {
    laux::lua_push(
        state,
        CONTEXT.tokio_runtime.metrics().num_alive_tasks() as i64,
    );
    return 1;
}

/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer `state`.
/// The caller must ensure that `state` is a valid pointer to a `lua_State`
/// and that it remains valid for the duration of the function call.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C-unwind" fn luaopen_rust_runtime(state: *mut ffi::lua_State) -> c_int {
    let l = [lreg!("num_alive_tasks", num_alive_tasks), lreg_null!()];

    ffi::lua_createtable(state, 0, l.len() as c_int);
    ffi::luaL_setfuncs(state, l.as_ptr(), 0);
    1
}
