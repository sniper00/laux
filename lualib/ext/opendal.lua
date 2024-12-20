local c = require "rust.opendal"
local moon = require "moon"
local protocol_type = 22
local callback = _G['send_message']

moon.register_protocol {
    name = "opendal",
    PTYPE = protocol_type,
    pack = function(...) return ... end,
    unpack = moon.tostring,
}

local M = {}

function M.new(schema, options)
    local o = {
        obj = assert(c.new(schema, options))
    }
    return setmetatable(o, { __index = M })
end

local function operators(self, ...)
    return moon.wait(self.obj:operators(protocol_type, callback, moon.next_sequence(), moon.id, ...))
end

function M:read(path)
    return operators(self, "read", path)
end

function M:write(path, data)
    return operators(self, "write", path, data)
end

function M:delete(path)
    return operators(self, "delete", path)
end

function M:exists(path)
    return operators(self, "exists", path)
end

function M:create_dir(path)
    return operators(self, "create_dir", path)
end

function M:rename(path, to_name)
    return operators(self, "rename", path, to_name)
end

function M:stat(path)
    return operators(self, "stat", path)
end

return M
