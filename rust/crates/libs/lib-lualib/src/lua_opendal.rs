use serde_json::json;
use std::str::FromStr;
use std::{collections::HashMap, ffi::c_int};

use ::opendal as od;

use lib_core::context::CONTEXT;
use lib_lua::{self, cstr, ffi, ffi::luaL_Reg, laux, lreg, lreg_null};

use crate::get_send_message_fn;

fn lua_to_schema(
    state: *mut ffi::lua_State,
    index: i32,
    schema: &str,
) -> Result<od::Operator, od::Error> {
    // [+1]
    let mut map = HashMap::<String, String>::default();
    laux::lua_pushnil(state);
    while laux::lua_next(state, index) {
        let key: &str = laux::lua_opt(state, -2).unwrap_or_default();
        let value: &str = laux::lua_opt(state, -1).unwrap_or_default();
        map.insert(key.to_string(), value.to_string());
        laux::lua_pop(state, 1);
    }

    let od_schema = od::Scheme::from_str(&schema)?;

    let op = od::Operator::via_iter(od_schema, map)?;
    op.blocking();
    Ok(op)
}

extern "C-unwind" fn operator_new(state: *mut ffi::lua_State) -> c_int {
    laux::lua_checktype(state, 2, ffi::LUA_TTABLE);
    let schema: &str = laux::lua_get(state, 1);
    if schema.is_empty() {
        laux::lua_push(state, false);
        laux::lua_push(state, "schema is empty");
        return 2;
    }

    let op: opendal::Operator = match lua_to_schema(state, 2, schema) {
        Ok(op) => op,
        Err(e) => {
            laux::lua_push(state, false);
            laux::lua_push(state, e.to_string());
            return 2;
        }
    };

    let l = [lreg!("operators", operators), lreg_null!()];
    if laux::lua_newuserdata(state, op, cstr!("opendal_metatable"), l.as_ref()).is_none() {
        laux::lua_push(state, false);
        laux::lua_push(state, "laux::lua_newuserdata failed");
        return 2;
    }

    1
}

extern "C-unwind" fn operators(state: *mut ffi::lua_State) -> c_int {
    laux::lua_checktype(state, 1, ffi::LUA_TUSERDATA);

    let op = laux::lua_touserdata::<opendal::Operator>(state, 1);
    if op.is_none() {
        laux::lua_error(state, "Invalid operator pointer");
    }
    let op = op.unwrap();

    let protocol_type = laux::lua_get::<u8>(state, 2);
    let callback = get_send_message_fn(state, 3);
    if callback.is_none() {
        laux::lua_error(state, "Invalid send_message function pointer");
    }

    let callback = callback.unwrap();
    let session: i64 = laux::lua_get(state, 4);
    let owner = laux::lua_get(state, 5);
    let op_name = laux::lua_get::<&str>(state, 6);

    if let Some(runtime) = CONTEXT.get_tokio_runtime().as_ref() {
        let path = laux::lua_get::<&str>(state, 7);
        if path.is_empty() {
            laux::lua_error(state, "path is empty");
        }

        let handle_result = move |result: opendal::Result<Vec<u8>>| match result {
            Ok(data) => {
                let vec = data.to_vec();
                callback(
                    protocol_type,
                    owner,
                    session,
                    vec.as_ptr() as *const i8,
                    vec.len(),
                );
            }
            Err(err) => {
                let err_str = err.to_string();
                callback(
                    4,
                    owner,
                    session,
                    err_str.as_ptr() as *const i8,
                    err_str.len(),
                );
            }
        };

        match op_name {
            "read" => {
                runtime.spawn(async move {
                    handle_result(op.read(path).await.map(|v| v.to_vec()));
                });
            }
            "write" => {
                let data = laux::lua_get::<&[u8]>(state, 8);
                runtime.spawn(async move {
                    handle_result(op.write(path, data).await.map(|_| vec![]));
                });
            }
            "delete" => {
                runtime.spawn(async move {
                    handle_result(op.delete(path).await.map(|_| vec![]));
                });
            }
            "exists" => {
                runtime.spawn(async move {
                    handle_result(
                        op.exists(path)
                            .await
                            .map(|exist| exist.to_string().into_bytes()),
                    );
                });
            }
            "create_dir" => {
                runtime.spawn(async move {
                    handle_result(op.create_dir(path).await.map(|_| vec![]));
                });
            }
            "rename" => {
                let to = laux::lua_get::<&str>(state, 8);
                if to.is_empty() {
                    laux::lua_error(state, "to is empty");
                }
                runtime.spawn(async move {
                    handle_result(op.rename(path, to).await.map(|_| vec![]));
                });
            }
            "stat" => {
                runtime.spawn(async move {
                    handle_result(op.stat(path).await.map(|stat| {
                        let json_obj = json!({
                            "content_length": stat.content_length(),
                            "content_md5": stat.content_md5(),
                            "content_type": stat.content_type(),
                            "is_dir": stat.is_dir(),
                            "is_file": stat.is_file()
                        });
                        json_obj.to_string().into_bytes()
                    }));
                });
            }
            "list"=> {
                runtime.spawn(async move {
                    handle_result(op.list(path).await.map(|list| {
                        let json_obj = list.into_iter().map(|stat| {
                            let (path, metadata) = stat.into_parts();
                            json!({
                                "path": path,
                                "memtadata": {
                                    "content_length": metadata.content_length(),
                                    "content_md5": metadata.content_md5(),
                                    "content_type": metadata.content_type(),
                                    "is_dir": metadata.is_dir(),
                                    "is_file": metadata.is_file()
                                }
                            })
                        }).collect::<Vec<_>>();
                        serde_json::to_string(&json_obj).unwrap_or_default().into_bytes()
                    }));
                });
            }
            _ => {
                laux::lua_push(state, false);
                laux::lua_push(state, "Invalid operator name");
                return 2;
            }
        }
    } else {
        laux::lua_push(state, false);
        laux::lua_push(state, "No tokio runtime");
        return 2;
    }

    laux::lua_push(state, session);
    1
}

#[no_mangle]
pub unsafe extern "C-unwind" fn luaopen_rust_opendal(state: *mut ffi::lua_State) -> c_int {
    let l = [lreg!("new", operator_new), lreg_null!()];

    ffi::lua_createtable(state, 0, l.len() as c_int);
    ffi::luaL_setfuncs(state, l.as_ptr(), 0);
    1
}
