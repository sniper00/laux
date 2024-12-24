pub mod lua_excel;
pub mod lua_http;
// pub mod lua_opendal;
pub mod lua_sqlx;
pub mod lua_json;
pub mod lua_runtime;

unsafe extern "C-unwind" {
    unsafe fn send_message(type_: u8, receiver: u32, session: i64, data: *const i8, len: usize);
}

pub fn moon_send<T>(
    protocol_type: u8,
    owner: u32,
    session: i64,
    res: T,
) {
    if session == 0 {
        return;
    }
    let ptr = Box::into_raw(Box::new(res));
    let bytes = (ptr as isize).to_ne_bytes();

    unsafe {
        send_message(
            protocol_type,
            owner,
            session,
            bytes.as_ptr() as *const i8,
            bytes.len(),
        );
    }
}

pub fn moon_send_string(
    protocol_type: u8,
    owner: u32,
    session: i64,
    data: String,
) {
    unsafe {
        send_message(
            protocol_type,
            owner,
            session,
            data.as_ptr() as *const i8,
            data.len(),
        );
    }
}

pub const PTYPE_ERROR: u8 = 4;
pub const PTYPE_LOG: u8 = 13;

pub const LOG_LEVEL_ERROR: u8 = 1;
pub const LOG_LEVEL_WARN: u8 = 2;
pub const LOG_LEVEL_INFO: u8 = 3;
pub const LOG_LEVEL_DEBUG: u8 = 4;

pub fn moon_log(owner: u32, log_level: u8, data: String) {
    let message = format!("{}{}", log_level, data);
    unsafe {
        send_message(
            PTYPE_LOG,
            owner,
            0,
            message.as_ptr() as *const i8,
            message.len(),
        );
    }
}
