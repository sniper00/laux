# Moon Extensions

This library provides Lua extensions for [Moon](https://github.com/sniper00/moon), implemented in Rust and C/C++. By using Rust, we can use its ecosystem, including the `tokio` runtime.

# Usage

## Option 1: Use Precompiled Releases

You can directly use the precompiled releases.

## Option 2: Manual Compilation

To compile the project, follow these steps:

- [Install Premake5](https://premake.github.io/download).
- Run `premake5 build` `premake5 publish`.

After compiling, the `clib` and `lualib` directories will be automatically copied to the moon directory.

# Libraries

## Rust

### 1. Excel Reader

```lua
local excel = require "rust.excel"

local res = excel.read("example.xlsx")

--[[
---format
{
    {
        "data":{
            {"A1","B1","C1"},
            {"A2","B2","C2"},
            {"A3","B3","C3"}
        },
        "sheet_name":"Sheet1"
    },
    {
        "data":{
            {"A1","B1","C1"},
            {"A2","B2","C2"},
            {"A3","B3","C3"}
        },
        "sheet_name":"Sheet2"
    }
}
]]

```

### 2. Https Client

```lua
    local httpc = require("ext.httpc")
    local moon = require("moon")
    moon.async(function()
        local response = httpc.get("https://bing.com")
        print(response.status_code==200)
    end)
```

### 3. Sqlx

```lua
local moon = require "moon"
local sqlx = require "ext.sqlx"

moon.async(function()
    local info = {
        cc      = 300,
        gpsname = "gps1",
        track   = {
            segments = {
                [1] = {
                    HR        = 73,
                    location  = {
                        [1] = 47.763,
                        [2] = 13.4034,
                    },
                    starttime = "2018-10-14 10:05:14",
                },
                [2] = {
                    HR        = 130,
                    location  = {
                        [1] = 47.706,
                        [2] = 13.2635,
                    },
                    starttime = "2018-10-14 10:39:21",
                },
            },
        },
    }

    local sql = string.format([[
        --create userdata table
        drop table if exists userdata;
    ]])

    local sql2 = [[
        create table userdata (
            uid	bigint,
            key		text,
            value   text,
            CONSTRAINT pk_userdata PRIMARY KEY (uid, key)
           );
    ]]


    local db = sqlx.connect("postgres://bruce:123456@localhost/postgres", "test")
    if db.kind then
        print("connect failed", db.message)
        return
    end

    print_r(db:query(sql))
    print_r(db:query(sql2))

    local result = db:query(
        "INSERT INTO userdata (uid, key, value) values($1, $2, $3) on conflict (uid, key) do update set value = excluded.value;",
        235, "info2", info)
    print_r(result)

    local st = moon.clock()
    for i = 1, 10000 do
        local res = db:query(
            "INSERT INTO userdata (uid, key, value) values($1, $2, $3) on conflict (uid, key) do update set value = excluded.value;",
            235, "info2", info)

        if res.kind then
            print("error", res.message)
            break
        end
    end
    print("cost", moon.clock() - st)

    ---sqlite
    local sqlitedb = sqlx.connect("sqlite://memory:", "test2")

    print_r(sqlitedb:query("CREATE TABLE test (id INTEGER PRIMARY KEY, content TEXT);"))
    print_r(sqlitedb:query("INSERT INTO test (content) VALUES ('Hello, World!');"))
    print_r(sqlitedb:query("SELECT * FROM test;"))

    print_r(sqlx.stats()) -- Query sqlx left task count
end)

```

## C/Cpp

### 1. lpeg

### 2. [math3d](https://github.com/cloudwu/math3d)

### 3. [sproto](https://github.com/cloudwu/sproto)
