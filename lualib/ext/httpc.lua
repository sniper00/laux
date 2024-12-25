---@diagnostic disable: inject-field
local moon = require "moon"
local json = require "json"
local c = require "rust.httpc"

local protocol_type = 21

moon.register_protocol {
    name = "http",
    PTYPE = protocol_type,
    pack = function(...) return ... end,
    unpack = function (val)
        return c.decode(val)
    end
}

---@return table
local function tojson(response)
    if response.status_code ~= 200 then return {} end
    return json.decode(response.body)
end

---@class HttpRequestOptions
---@field headers? table<string,string>
---@field timeout? integer Request timeout in seconds. default 5s
---@field proxy? string

local client = {}

---@param url string
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.get(url, opts)
    opts = opts or {}
    opts.owner = moon.id
    opts.session = moon.next_sequence()
    opts.url = url
    opts.method = "GET"
    return moon.wait(c.request(opts, protocol_type))
end

local json_content_type = { ["Content-Type"] = "application/json" }

---@param url string
---@param data table
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.post_json(url, data, opts)
    opts = opts or {}
    opts.owner = moon.id
    opts.session = moon.next_sequence()
    if not opts.headers then
        opts.headers = json_content_type
    else
        if not opts.headers['Content-Type'] then
            opts.headers['Content-Type'] = "application/json"
        end
    end

    opts.url = url
    opts.method = "POST"
    opts.body = json.encode(data)

    local res = moon.wait(c.request(opts, protocol_type))

    if res.status_code == 200 then
        res.body = tojson(res)
    end
    return res
end

---@param url string
---@param data string
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.post(url, data, opts)
    opts = opts or {}
    opts.owner = moon.id
    opts.session = moon.next_sequence()
    opts.url = url
    opts.body = data
    opts.method = "POST"
    return moon.wait(c.request(opts, protocol_type))
end

local form_headers = { ["Content-Type"] = "application/x-www-form-urlencoded" }

---@param url string
---@param data table<string,string>
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.post_form(url, data, opts)
    opts = opts or {}
    opts.owner = moon.id
    opts.session = moon.next_sequence()
    if not opts.headers then
        opts.headers = form_headers
    else
        if not opts.headers['Content-Type'] then
            opts.headers['Content-Type'] = "application/x-www-form-urlencoded"
        end
    end

    opts.body = {}
    for k, v in pairs(data) do
        opts.body[k] = tostring(v)
    end

    opts.url = url
    opts.method = "POST"
    opts.body = c.form_urlencode(opts.body)

    return moon.wait(c.request(opts, protocol_type))
end

return client