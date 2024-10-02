---@diagnostic disable: inject-field
local moon = require "moon"
local json = require "json"
local httpc = require "httpc"

local coroutine = coroutine
local debug = debug

local requests = {}
moon.async(function()
    while true do
        local session, data = httpc.poll()
        if not session then
            moon.sleep(10)
        else
            local co = requests[session]
            requests[session] = nil
            moon.timeout(0, function()
                local ok, err = coroutine.resume(co, data)
                if not ok then
                    err = debug.traceback(co, tostring(err))
                    coroutine.close(co)
                    moon.error(err)
                end
            end)
        end
    end
end)

local function request(opts)
    local session, err = httpc.request(opts)
    local co = coroutine.running()
    requests[session] = co
    return coroutine.yield()
end

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
    opts.url = url
    opts.method = "GET"
    return request(opts)
end

local json_content_type = { ["Content-Type"] = "application/json" }

---@param url string
---@param data table
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.post_json(url, data, opts)
    opts = opts or {}
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

    local res = request(opts)

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
    opts.url = url
    opts.body = data
    opts.method = "POST"
    return request(opts)
end

local form_headers = { ["Content-Type"] = "application/x-www-form-urlencoded" }

---@param url string
---@param data table<string,string>
---@param opts? HttpRequestOptions
---@return HttpResponse
function client.post_form(url, data, opts)
    opts = opts or {}
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
    opts.body = httpc.form_urlencode(opts.body)

    return request(opts)
end

return client
