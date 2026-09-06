-- balatro-vs: Lua side of the native bridge
--
-- Loaded by the native code right after it registered `bvs_native_dispatch(name, args_json)`.
-- Defines every native function as a global Lua wrapper.
-- Arguments and results travel as JSON arrays

if bvs_lua_dispatch then return end -- already loaded

local native = bvs_native_dispatch
assert(type(native) == "function", "bvs_native_dispatch is missing")

local json
local function Json()
    if not json then json = require("json") end
    return json
end

local function pack(...)
    local n = select("#", ...)
    local t = {}
    for i = 1, n do t[i] = select(i, ...) end
    return t
end

local function call_native(name, ...)
    local reply = Json().decode(native(name, Json().encode(pack(...))))
    if reply.error then error(reply.error, 2) end
    return unpack(reply.ok)
end

for name in native("__functions", ""):gmatch("[^,]+") do
    _G[name] = function(...) return call_native(name, ...) end
end

function bvs_lua_dispatch(name, args_json)
    local fn = _G[name]
    if type(fn) ~= "function" then
        return Json().encode({ error = "no Lua function named " .. tostring(name) })
    end
    local args = Json().decode(args_json)
    local ok, result = pcall(fn, unpack(args))
    if not ok then
        return Json().encode({ error = tostring(result) })
    end
    return Json().encode({ ok = { result } })
end
