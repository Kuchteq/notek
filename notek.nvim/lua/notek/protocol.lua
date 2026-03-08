--- Binary protocol encoding for the notek headless client.
---
--- Wire format (all integers are unsigned little-endian):
---   Insert:  opcode=0  | u32 start_byte | u32 text_len | text
---   Delete:  opcode=1  | u32 start_byte | u32 len
---   Start:   opcode=2  | u32 name_len   | document_name
---   Flush:   opcode=3
local bit = require("bit")

local M = {}

---Encode an unsigned 8-bit integer.
---@param n integer
---@return string
function M.u8(n)
  return string.char(n)
end

---Encode an unsigned 32-bit little-endian integer.
---@param n integer
---@return string
function M.u32(n)
  n = bit.band(n, 0xffffffff)
  return string.char(
    bit.band(n, 0xff),
    bit.band(bit.rshift(n, 8), 0xff),
    bit.band(bit.rshift(n, 16), 0xff),
    bit.band(bit.rshift(n, 24), 0xff)
  )
end

---Encode a delete operation.
---@param start_byte integer
---@param len integer
---@return string
function M.encode_delete(start_byte, len)
  return M.u8(1) .. M.u32(start_byte) .. M.u32(len)
end

---Encode an insert operation.
---@param start_byte integer
---@param text string
---@return string
function M.encode_insert(start_byte, text)
  return M.u8(0) .. M.u32(start_byte) .. M.u32(#text) .. text
end

---Encode a start-session message with the document name.
---@param document string
---@return string
function M.encode_document_select(document)
  return M.u8(2) .. M.u32(#document) .. document
end

---Encode a flush message.
---@return string
function M.encode_flush()
  return M.u8(3)
end

return M
