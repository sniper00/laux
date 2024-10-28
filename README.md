# Moon Extensions

This library provides Lua extensions for [Moon](https://github.com/sniper00/moon), implemented in Rust and C/C++. By using Rust, we can leverage its ecosystem, including the `tokio` runtime.

# Usage

## Option 1: Use Precompiled Releases

You can directly use the precompiled releases.

## Option 2: Manual Compilation

To compile the project, follow these steps:

- [Install Premake5](https://premake.github.io/download).
- Run `premake5 build`.

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
    "sheet_name_1":{
        {"A1","B1","C1"},
        {"A2","B2","C2"},
        {"A3","B3","C3"}
    },
    "sheet_name_2":{
        {"A1","B1","C1"},
        {"A2","B2","C2"},
        {"A3","B3","C3"}
    },
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

## C/Cpp

### 1. lpeg

### 2. [math3d](https://github.com/cloudwu/math3d)

### 3. [sproto](https://github.com/cloudwu/sproto)
