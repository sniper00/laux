use dashmap::DashMap;
use lazy_static::lazy_static;
use lib_lua::{self, cstr, ffi, ffi::luaL_Reg, laux, lreg, lreg_null, lua_rawsetfield};
use reqwest::ClientBuilder;
use reqwest::{header::HeaderMap, Method};
use std::{error::Error, ffi::c_int, str::FromStr};
use std::{
    sync::{atomic::AtomicI64, mpsc},
    time::Duration,
};
use tokio::runtime::Builder;
use url::form_urlencoded::{self};

lazy_static! {
    pub static ref CONTEXT: HttpContext = {
        let tokio_runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_time()
            .enable_io()
            .build();

        HttpContext {
            http_clients: DashMap::new(),
            session: AtomicI64::new(1),
            tokio_runtime: if let Ok(rt) = tokio_runtime {
                Some(rt)
            } else {
                None
            },
        }
    };
}

pub struct HttpContext {
    session: AtomicI64,
    http_clients: DashMap<String, reqwest::Client>,
    tokio_runtime: Option<tokio::runtime::Runtime>,
}

struct HttpRequest {
    session: i64,
    method: String,
    url: String,
    body: String,
    headers: HeaderMap,
    timeout: u64,
    proxy: String,
}

struct HttpResponse {
    version: String,
    status_code: i32,
    headers: HeaderMap,
    body: Vec<u8>,
}

type Message = (i64, Result<HttpResponse, String>);
type ChannelUserData = *mut Channel;

struct Channel {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
}

impl HttpContext {
    pub fn get_http_client(&self, timeout: u64, proxy: &String) -> reqwest::Client {
        let name = format!("{}_{}", timeout, proxy);
        if let Some(client) = self.http_clients.get(&name) {
            return client.clone();
        }

        let builder = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .use_rustls_tls()
            .tcp_nodelay(true);

        let client = if proxy.is_empty() {
            builder.build().unwrap_or_default()
        } else {
            match reqwest::Proxy::all(proxy) {
                Ok(proxy) => builder.proxy(proxy).build().unwrap_or_default(),
                Err(_) => builder.build().unwrap_or_default(),
            }
        };

        self.http_clients.insert(name.to_string(), client.clone());
        client
    }
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
    sender: &mpsc::Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let http_client = &CONTEXT.get_http_client(req.timeout, &req.proxy);

    let response = http_client
        .request(Method::from_str(req.method.as_str())?, req.url)
        .headers(req.headers)
        .body(req.body)
        .send()
        .await?;

    let response = HttpResponse {
        version: version_to_string(&response.version()).to_string(),
        status_code: response.status().as_u16() as i32,
        headers: response.headers().clone(),
        body: response.bytes().await?.to_vec(),
    };

    let _ = sender.send((req.session, Ok(response)));
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

    let headers = match extract_headers(state, 1) {
        Ok(headers) => headers,
        Err(err) => {
            laux::lua_push(state, false);
            laux::lua_push(state, err);
            return 2;
        }
    };

    let session = CONTEXT
        .session
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let req = HttpRequest {
        session,
        method: laux::opt_field(state, 1, "method").unwrap_or("GET".to_string()),
        url: laux::opt_field(state, 1, "url").unwrap_or_default(),
        body: laux::opt_field(state, 1, "body").unwrap_or_default(),
        headers,
        timeout: laux::opt_field(state, 1, "timeout").unwrap_or(5),
        proxy: laux::opt_field(state, 1, "proxy").unwrap_or_default(),
    };

    let channel = get_channel(state, ffi::lua_upvalueindex(1));

    let sender = channel.sender.clone();

    if let Some(runtime) = CONTEXT.tokio_runtime.as_ref() {
        runtime.spawn(async move {
            let session = req.session;
            if let Err(err) = http_request(req, &sender).await {
                let _ = sender.send((session, Err(err.to_string())));
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

extern "C-unwind" fn lua_http_poll(state: *mut ffi::lua_State) -> c_int {
    let channel = get_channel(state, ffi::lua_upvalueindex(1));

    match channel.receiver.try_recv() {
        Ok((session, result)) => match result {
            Ok(response) => unsafe {
                laux::lua_push(state, session);
                ffi::lua_createtable(state, 0, 6);
                lua_rawsetfield!(
                    state,
                    -3,
                    "version",
                    laux::lua_push(state, response.version.as_str())
                );
                lua_rawsetfield!(
                    state,
                    -3,
                    "status_code",
                    laux::lua_push(state, response.status_code)
                );

                lua_rawsetfield!(
                    state,
                    -3,
                    "body",
                    laux::lua_push(state, response.body.as_slice())
                );

                ffi::lua_pushstring(state, cstr!("headers"));
                ffi::lua_createtable(state, 0, 16);
                for (key, value) in response.headers.iter() {
                    laux::lua_push(state, key.to_string().to_lowercase());
                    laux::lua_push(state, value.to_str().unwrap_or(""));
                    ffi::lua_rawset(state, -3);
                }
                ffi::lua_rawset(state, -3);

                2
            },
            Err(err) => unsafe {
                laux::lua_push(state, session);
                ffi::lua_createtable(state, 0, 6);
                lua_rawsetfield!(state, -3, "status_code", laux::lua_push(state, -1));
                lua_rawsetfield!(state, -3, "body", laux::lua_push(state, err));
                2
            },
        },
        Err(mpsc::TryRecvError::Empty) => {
            laux::lua_push(state, false);
            laux::lua_push(state, "Again");
            2
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            laux::lua_push(state, false);
            laux::lua_push(state, "Closed");
            2
        }
    }
}

fn get_channel(state: *mut ffi::lua_State, index: i32) -> &'static mut Channel {
    unsafe {
        let ud = ffi::lua_touserdata(state, index) as *mut ChannelUserData;
        &mut *(*ud)
    }
}

// The __gc method to release the channel pointer
unsafe extern "C-unwind" fn lua_channel_gc(state: *mut ffi::lua_State) -> c_int {
    let ud = ffi::lua_touserdata(state, 1) as *mut ChannelUserData;
    if !ud.is_null() && !(*ud).is_null() {
        let _ = Box::from_raw(*ud); // This will drop the Box and release the memory
        *ud = std::ptr::null_mut();
    }
    0
}

#[no_mangle]
pub unsafe extern "C-unwind" fn luaopen_httpc(state: *mut ffi::lua_State) -> c_int {
    let l = [
        lreg!("request", lua_http_request),
        lreg!("poll", lua_http_poll),
        lreg!("form_urlencode", lua_http_form_urlencode),
        lreg!("form_urldecode", lua_http_form_urldecode),
        lreg_null!(),
    ];

    let (sender, receiver) = mpsc::channel();
    let channel = Box::new(Channel { sender, receiver });

    ffi::lua_createtable(state, 0, l.len() as c_int);
    // Create a new userdata and set its metatable
    let ud =
        ffi::lua_newuserdata(state, std::mem::size_of::<ChannelUserData>()) as *mut ChannelUserData;
    *ud = Box::into_raw(channel);
    // Create a metatable for the userdata
    if ffi::luaL_newmetatable(state, cstr!("HTTP_CHANNEL_MT")) > 0 {
        ffi::lua_pushcfunction(state, lua_channel_gc);
        ffi::lua_setfield(state, -2, cstr!("__gc"));
    }
    ffi::lua_setmetatable(state, -2);
    ffi::luaL_setfuncs(state, l.as_ptr(), 1);
    1
}
