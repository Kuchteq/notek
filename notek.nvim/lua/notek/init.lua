local config = require("notek.config")
local protocol = require("notek.protocol")
local socket = require("notek.socket")

local M = {}

--- Set of buffer numbers that have on_bytes attached.
---@type table<integer, boolean>
local attached = {}

---@param bufnr integer
---@return boolean
local function is_trackable(bufnr)
  -- Skip special buffer types
  local buftype = vim.bo[bufnr].buftype
  if buftype ~= "" then return false end

  -- Must have a file name
  local name = vim.api.nvim_buf_get_name(bufnr)
  if name == "" then return false end

  local stat = vim.uv.fs_stat(name)
  if not stat or stat.type ~= "file" then return false end

  return true
end

---Select a document on the shared connection, sending the select (start) command.
---Called on every BufEnter for trackable buffers.
---@param bufnr integer
local function select_document(bufnr)
  local conn = socket.get()
  local buf_name = vim.api.nvim_buf_get_name(bufnr)
  conn:send(protocol.encode_document_select(buf_name))
end

---Attach on_bytes to a buffer if not already attached, and select it.
---@param bufnr integer
function M.attach(bufnr)
  if not is_trackable(bufnr) then return end

  select_document(bufnr)

  -- on_bytes already hooked for this buffer
  if attached[bufnr] then return end
  attached[bufnr] = true

  vim.api.nvim_buf_attach(bufnr, false, {
    on_bytes = function(
      _,
      buf,
      _changedtick,
      start_row,
      start_col,
      start_byte,
      _old_end_row,
      _old_end_col,
      old_end_byte,
      new_end_row,
      new_end_col,
      new_end_byte
    )
      if not attached[buf] then return true end -- returning true detaches

      local conn = socket.get()

      -- Delete event
      if old_end_byte > 0 then
        conn:send(protocol.encode_delete(start_byte, old_end_byte))
      end

      -- Insert event
      if new_end_byte > 0 then
        local lines = vim.api.nvim_buf_get_text(
          buf,
          start_row,
          start_col,
          start_row + new_end_row,
          start_col + new_end_col,
          {}
        )
        local text = table.concat(lines, "\n")
        conn:send(protocol.encode_insert(start_byte, text))
      end
    end,

    on_detach = function(_, buf)
      attached[buf] = nil
    end,
  })
end

---Detach from a buffer.
---@param bufnr integer
function M.detach(bufnr)
  attached[bufnr] = nil
  -- on_bytes will return true on the next callback, completing the detach.
end

---Flush the current document (e.g., on save).
---@param bufnr integer
function M.flush(bufnr)
  if not attached[bufnr] then return end
  local conn = socket.get()
  conn:send(protocol.encode_flush())
end

---Flush and close the shared connection. Called on VimLeave.
function M.shutdown()
  attached = {}
  socket.close()
end

---Plugin setup. Call this from your Neovim config.
---@param opts? notek.Config
function M.setup(opts)
  config.setup(opts)
end

return M
