---@diagnostic disable: inject-field
local moon = require "moon"
local c = require "rust.sqlx"

local protocol_type = 23
local callback = _G['send_message']

moon.register_protocol {
    name = "database",
    PTYPE = protocol_type,
    pack = function(...) return ... end,
    unpack = function(sz, len)
        return c.decode(sz, len)
    end
}

local M = {}

---@nodiscard
---@param database_url string Database url e. "postgres://postgres:123456@localhost/postgres"
---@param name string Connection name for find by other services
---@param timeout? integer Connect timeout. Default 5000ms
function M.connect(database_url, name, timeout)
    local res = moon.wait(c.connect(protocol_type, moon.id, callback, moon.next_sequence(), database_url, name, timeout))
    if res.kind then
        error(string.format("connect database failed: %s", res.message))
    end
    local o = {
        obj = c.find_connection(name)
    }
    return setmetatable(o, { __index = M })
end

---@nodiscard
---@param name string Connection name
function M.find_connection(name)
    return c.find_connection(name)
end

function M:close()
    self.obj:close()
end

---@param sql string
---@vararg any
function M:execute(sql, ...)
    self.obj:query(0, sql, ...)
end

---@nodiscard
---@param sql string
---@vararg any
---@return table
function M:query(sql, ...)
    local session = self.obj:query(moon.next_sequence(), sql, ...)
    if type(session) == "table" then
        return session
    end
    return moon.wait(session)
end

return M
