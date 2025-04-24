function pretty_print(center)
    local seen = {}
    local function serialize(value)
        if type(value) == "table" then
            if seen[value] then
                return "(circular reference)"
            end
            seen[value] = true
            local result = {}
            for k, v in pairs(value) do
                table.insert(result, "[" .. serialize(k) .. "]=" .. serialize(v))
            end
            return "{" .. table.concat(result, ",") .. "}"
        elseif type(value) == "userdata" then
            return "(userdata)"
        elseif type(value) == "function" then
            return "(function)"
        end
        return lume.serialize(value)
    end

    local serialized = serialize(center)
    local indent = 0
    local function addIndentation(str)
        return string.rep("  ", indent) .. str
    end

    local formatted = serialized:gsub("([{}])", function(c)
        if c == "{" then
            indent = indent + 1
            return "{\n" .. addIndentation("")
        elseif c == "}" then
            indent = indent - 1
            return "\n" .. addIndentation("") .. "}"
        end
    end)

    formatted = formatted:gsub(",([^}])", ",\n" .. addIndentation("%1"))
    print(formatted)
end

-- VecDeque implementation
local VecDeque = {}
VecDeque.__index = VecDeque

function VecDeque:new()
    return setmetatable({ first = 0, last = -1, items = {} }, self)
end

function VecDeque:push_front(value)
    self.first = self.first - 1
    self.items[self.first] = value
end

function VecDeque:push_back(value)
    self.last = self.last + 1
    self.items[self.last] = value
end

function VecDeque:pop_front()
    if self.first > self.last then
        error("Deque is empty")
    end
    local value = self.items[self.first]
    self.items[self.first] = nil
    self.first = self.first + 1
    return value
end

function VecDeque:pop_back()
    if self.first > self.last then
        error("Deque is empty")
    end
    local value = self.items[self.last]
    self.items[self.last] = nil
    self.last = self.last - 1
    return value
end

function VecDeque:is_empty()
    return self.first > self.last
end

function VecDeque:size()
    return self.last - self.first + 1
end

function VecDeque:peek_front()
    if self:is_empty() then
        error("Deque is empty")
    end
    return self.items[self.first]
end

function VecDeque:peek_back()
    if self:is_empty() then
        error("Deque is empty")
    end
    return self.items[self.last]
end

-- Timer implementation
local EventTimer = {}
EventTimer.__index = EventTimer

function EventTimer:new(duration, callback)
    local obj = setmetatable({}, EventTimer)
    obj.duration = duration
    obj.callback = callback
    obj.time_left = duration
    obj.active = false
    obj.paused = true
    obj.pretty_time_left = math.floor(duration)
    return obj
end

function EventTimer:start()
    self.active = true
    self.paused = false
end

function EventTimer:start_paused()
    self.active = true
    self.paused = true
end

function EventTimer:modify(new_duration_modifier)
    self.time_left = self.duration - (self.duration * new_duration_modifier)
end

function EventTimer:update(dt)
    if not G.SETTINGS.paused and self.active and not self.paused then
        self.time_left = self.time_left - dt
        if self.time_left <= 0 then
            self.active = false
            if self.callback then
                self.callback()
                self:stop()
            end
        end
    end
end

function EventTimer:pause()
    if not self.active then
        print("Timer is not active")
        return
    end

    if self.paused then
        print("Timer is already paused")
        return
    end

    self.paused = true
end

function EventTimer:resume()
    print("Resuming timer")
    if not self.active then
        print("Timer is not active")
        return
    end

    if not self.paused then
        print("Timer is not paused")
        return
    end

    self.paused = false
end

function EventTimer:stop()
    if not self.active then
        print("Timer is not active")
        return
    end

    self.active = false
    self.paused = false
end

function EventTimer:update_and_get_pretty_time_left()
    local pretty_time_left = math.floor(self.time_left)
    if pretty_time_left ~= self.pretty_time_left then
        self.pretty_time_left = pretty_time_left
        return tostring(pretty_time_left)
    end

    return nil
end

function EventTimer:is_active()
    return self.active
end

function EventTimer:is_paused()
    return self.paused
end

return {
    VecDeque = VecDeque,
    EventTimer = EventTimer
}
