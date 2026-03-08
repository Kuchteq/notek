--- Socket connection management for communicating with the notek headless client.
--- A single shared connection is used for all buffers, since the headless client
--- can only service one connection at a time.
local config = require("notek.config")
local protocol = require("notek.protocol")

local M = {}

---@class notek.Connection
---@field pipe uv_pipe_t
---@field connected boolean
---@field pending string[] Messages queued before the connection is ready
local Connection = {}
Connection.__index = Connection

---Create a new connection to the headless client.
---@param on_connect? fun(err?: string) Optional callback after connect completes
---@return notek.Connection
function Connection.new(on_connect)
  local self = setmetatable({}, Connection)
  self.pipe = vim.uv.new_pipe(false)
  self.connected = false
  self.pending = {}

  self.pipe:connect(config.values.socket_path, function(err)
    if err then
      vim.schedule(function()
        vim.notify("[notek] socket connect error: " .. err, vim.log.levels.ERROR)
      end)
      if on_connect then on_connect(err) end
      return
    end
    self.connected = true
    -- Drain any messages that were queued while connecting
    for _, data in ipairs(self.pending) do
      self.pipe:write(data, function(write_err)
        if write_err then
          vim.schedule(function()
            vim.notify("[notek] socket write error: " .. write_err, vim.log.levels.WARN)
          end)
        end
      end)
    end
    self.pending = {}
    if on_connect then on_connect(nil) end
  end)

  return self
end

---Send data over the connection. Queues the message if not yet connected.
---@param data string
function Connection:send(data)
  if not data then return end
  if self.pipe:is_closing() then return end

  if not self.connected then
    table.insert(self.pending, data)
    return
  end

  self.pipe:write(data, function(err)
    if err then
      vim.schedule(function()
        vim.notify("[notek] socket write error: " .. err, vim.log.levels.WARN)
      end)
    end
  end)
end

---Flush and close the connection.
function Connection:close()
  if self.pipe and not self.pipe:is_closing() then
    self:send(protocol.encode_flush())
    self.pipe:close()
  end
  self.connected = false
  self.pending = {}
end

---Check if the connection is still alive.
---@return boolean
function Connection:is_alive()
  return self.pipe ~= nil and not self.pipe:is_closing()
end

M.Connection = Connection

--- The single shared connection instance.
---@type notek.Connection?
M.shared = nil

---Get or create the shared connection.
---@return notek.Connection
function M.get()
  if not M.shared or not M.shared:is_alive() then
    M.shared = Connection.new()
  end
  return M.shared
end

---Close the shared connection.
function M.close()
  if M.shared then
    M.shared:close()
    M.shared = nil
  end
end

return M
