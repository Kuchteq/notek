local M = {}

---@class notek.Config
---@field socket_path string Path to the headless client Unix socket
---@field debug boolean Enable debug notifications
---@field auto_attach boolean Automatically attach to buffers on BufEnter

---@type notek.Config
local defaults = {
  socket_path = "/tmp/editor_socket.sock",
  debug = false,
  auto_attach = true,
}

---@type notek.Config
M.values = vim.deepcopy(defaults)

---@param opts? notek.Config
function M.setup(opts)
  M.values = vim.tbl_deep_extend("force", defaults, opts or {})
end

return M
