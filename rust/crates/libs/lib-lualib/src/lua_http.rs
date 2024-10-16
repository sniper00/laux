use lib_core::{buffer::Buffer, context::CONTEXT};
use lib_lua::{self, cstr, ffi, ffi::luaL_Reg, laux, lreg, lreg_null};
use reqwest::{header::HeaderMap, Method};
use std::{error::Error, ffi::c_int, str::FromStr};
use url::form_urlencoded::{self};

use crate::{get_send_message_fn, SendMessageFn};

struct HttpRequest {
    owner: u32,
    session: i64,
    method: String,
    url: String,
    body: String,
    headers: HeaderMap,
    timeout: u64,
    proxy: String,
}

fn version_to_string(version: &reqwest::Version) -> &str {
    match *version {
        reqwest::Version::HTTP_09 => "HTTP/0.9",
        reqwest::Version::HTTP_10 => "HTTP/1.0",
        reqwest::Version::HTTP_11 => "HTTP/1.1",
        reqwest::Version::HTTP_2 => "HTTP/2.0",
        reqwest::Version::HTTP_3 => "HTTP/3.0",
        _ => "Unknown",
    }
}

async fn http_request(
    req: HttpRequest,
    callback: SendMessageFn,
    protocol_type: u8,
) -> Result<(), Box<dyn Error>> {
    let http_client = &CONTEXT.get_http_client(req.timeout, &req.proxy);

    let response = http_client
        .request(Method::from_str(req.method.as_str())?, req.url)
        .headers(req.headers)
        .body(req.body)
        .send()
        .await?;

    let mut buffer = Buffer::with_capacity(256);

    buffer.commit(std::mem::size_of::<u32>());

    buffer.write_str(
        format!(
            "{} {} {}\r\n",
            version_to_string(&response.version()),
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        )
        .as_str(),
    );

    for (key, value) in response.headers().iter() {
        buffer.write_str(
            format!(
                "{}: {}\r\n",
                key.to_string().to_lowercase(),
                value.to_str().unwrap_or("")
            )
            .as_str(),
        );
    }

    buffer.write_str("\r\n\r\n");

    buffer.seek(std::mem::size_of::<u32>() as isize);
    buffer.write_front((buffer.len() as u32).to_le_bytes().as_ref());

    let body = response.bytes().await?;
    buffer.write_slice(body.as_ref());

    callback(
        protocol_type,
        req.owner,
        req.session,
        buffer.as_ptr() as *const i8,
        buffer.len(),
    );
    Ok(())
}

fn extract_headers(state: *mut ffi::lua_State, index: i32) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    laux::push_c_string(state, cstr!("headers"));
    if laux::lua_rawget(state, index) == ffi::LUA_TTABLE {
        // [+1]
        laux::lua_pushnil(state);
        while laux::lua_next(state, -2) {
            let key: &str = laux::lua_opt(state, -2).unwrap_or_default();
            let value: &str = laux::lua_opt(state, -1).unwrap_or_default();
            match key.parse::<reqwest::header::HeaderName>() {
                Ok(name) => match value.parse::<reqwest::header::HeaderValue>() {
                    Ok(value) => {
                        headers.insert(name, value);
                    }
                    Err(err) => return Err(err.to_string()),
                },
                Err(err) => return Err(err.to_string()),
            }
            laux::lua_pop(state, 1);
        }
        laux::lua_pop(state, 1); //pop headers table
    }

    Ok(headers)
}

extern "C-unwind" fn lua_http_request(state: *mut ffi::lua_State) -> c_int {
    laux::lua_checktype(state, 1, ffi::LUA_TTABLE);

    let protocol_type = laux::lua_get::<u8>(state, 2);

    let callback = get_send_message_fn(state, 3);
    if callback.is_none() {
        laux::lua_push(state, false);
        laux::lua_push(state, "Invalid send_message function pointer");
        return 2;
    }

    let callback = callback.unwrap();

    let headers = match extract_headers(state, 1) {
        Ok(headers) => headers,
        Err(err) => {
            laux::lua_push(state, false);
            laux::lua_push(state, err);
            return 2;
        }
    };

    let session = laux::opt_field(state, 1, "session").unwrap_or(0);

    let req = HttpRequest {
        owner: laux::opt_field(state, 1, "owner").unwrap_or_default(),
        session,
        method: laux::opt_field(state, 1, "method").unwrap_or("GET".to_string()),
        url: laux::opt_field(state, 1, "url").unwrap_or_default(),
        body: laux::opt_field(state, 1, "body").unwrap_or_default(),
        headers,
        timeout: laux::opt_field(state, 1, "timeout").unwrap_or(5),
        proxy: laux::opt_field(state, 1, "proxy").unwrap_or_default(),
    };

    if let Some(runtime) = CONTEXT.get_tokio_runtime().as_ref() {
        runtime.spawn(async move {
            let session = req.session;
            let owner = req.owner;
            if let Err(err) = http_request(req, callback, protocol_type).await {
                callback(
                    4,
                    owner,
                    session,
                    err.to_string().as_ptr() as *const i8,
                    err.to_string().len(),
                );
            }
        });
    } else {
        laux::lua_push(state, false);
        laux::lua_push(state, "No tokio runtime");
        return 2;
    }

    laux::lua_push(state, session);
    1
}

extern "C-unwind" fn lua_http_form_urlencode(state: *mut ffi::lua_State) -> c_int {
    laux::lua_checktype(state, 1, ffi::LUA_TTABLE);
    laux::lua_pushnil(state);
    let mut result = String::new();
    while laux::lua_next(state, 1) {
        if !result.is_empty() {
            result.push('&');
        }
        let key = laux::to_string_unchecked(state, -2);
        let value = laux::to_string_unchecked(state, -1);
        result.push_str(
            form_urlencoded::byte_serialize(key.as_bytes())
                .collect::<String>()
                .as_str(),
        );
        result.push('=');
        result.push_str(
            form_urlencoded::byte_serialize(value.as_bytes())
                .collect::<String>()
                .as_str(),
        );
        laux::lua_pop(state, 1);
    }
    laux::lua_push(state, result);
    1
}

extern "C-unwind" fn lua_http_form_urldecode(state: *mut ffi::lua_State) -> c_int {
    let query_string = laux::lua_get::<&str>(state, 1);

    unsafe { ffi::lua_createtable(state, 0, 8) };

    let decoded: Vec<(String, String)> = form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect();

    for pair in decoded {
        laux::lua_push(state, pair.0);
        laux::lua_push(state, pair.1);
        unsafe {
            ffi::lua_rawset(state, -3);
        }
    }
    1
}

#[no_mangle]
pub unsafe extern "C-unwind" fn luaopen_rust_httpc(state: *mut ffi::lua_State) -> c_int {
    let l = [
        lreg!("request", lua_http_request),
        lreg!("form_urlencode", lua_http_form_urlencode),
        lreg!("form_urldecode", lua_http_form_urldecode),
        lreg_null!(),
    ];

    ffi::lua_createtable(state, 0, l.len() as c_int);
    ffi::luaL_setfuncs(state, l.as_ptr(), 0);
    1
}
