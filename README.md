# laux
These Lua utility libraries, written in C++ and Rust, are primarily provided as Lua extension libraries for [Moon](https://github.com/sniper00/moon). However, they can also be used elsewhere as they are standard Lua C modules.

# Use

You can directly use the precompiled releases or compile it yourself. To compile on Windows, run `build.bat`. On Linux, run `build.sh`. Then, copy the dynamic libraries from the `bin` directory to your project and set the Lua `package.cpath` accordingly. On Windows, it depends on `lua.dll`. Currently, only Lua 5.4 is supported.

# Libraries

## 1. Excel Reader

```lua
local excel = require "excel"

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

## 2. lpeg

## 3. [math3d](https://github.com/cloudwu/math3d)

## 4. [sproto](https://github.com/cloudwu/sproto)
