local bit = require("bit")

local function attach_to_buffer(bufnr)
        local uv = vim.uv
        local socket_path = "/tmp/editor_socket.sock"

        -- ---------- Binary helpers ----------

        local function u32_le(n)
                n = bit.band(n, 0xffffffff)

                return string.char(
                        bit.band(n, 0xff),
                        bit.band(bit.rshift(n, 8), 0xff),
                        bit.band(bit.rshift(n, 16), 0xff),
                        bit.band(bit.rshift(n, 24), 0xff)
                )
        end

        local function u16_le(n)
                n = bit.band(n, 0xffff)

                return string.char(
                        bit.band(n, 0xff),
                        bit.band(bit.rshift(n, 8), 0xff)
                )
        end
        -- delete: bit31 = 1
        local function encode_delete(start_byte, end_byte)
                local header = bit.bor(0x80000000, start_byte)
                return u32_le(header) .. u32_le(end_byte)
        end

        -- insert: bit31 = 0
        local function encode_insert(start_byte, end_byte, text)
                local header = bit.band(start_byte, 0x7fffffff)

                local len = #text
                return u32_le(header)
                    .. u32_le(end_byte)
                    .. u16_le(len)
                    .. text
        end

        -- ---------- Socket ----------

        local sock = uv.new_pipe(false)

        local function connect_socket()
                sock:connect(socket_path, function(err)
                        if err then
                                vim.notify("Socket connect error: " .. err, vim.log.levels.ERROR)
                        end
                end)
        end

        connect_socket()

        local function send(data)
                if data and not sock:is_closing() then
                        sock:write(data)
                end
        end

        -- ---------- Buffer Attach ----------

        vim.api.nvim_buf_attach(bufnr, false, {
                on_bytes = function(
                    _,
                    bufnr,
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
                        -- -------- DELETE EVENT --------
                        if old_end_byte > 0 then
                                local delete_end = start_byte + old_end_byte

                                send(encode_delete(
                                        start_byte,
                                        delete_end
                                ))
                        end

                        -- -------- INSERT EVENT --------
                        if new_end_byte > 0 then
                                local text = table.concat(
                                        vim.api.nvim_buf_get_text(
                                                bufnr,
                                                start_row,
                                                start_col,
                                                start_row + new_end_row,
                                                start_col + new_end_col,
                                                {}
                                        ),
                                        "\n"
                                )

                                local insert_end = start_byte + new_end_byte

                                send(encode_insert(
                                        start_byte,
                                        insert_end,
                                        text
                                ))
                        end
                end,

                on_detach = function(_, buf)
                        if not sock:is_closing() then
                                sock:close()
                        end

                        vim.b[buf].attached = false
                end,
        })
end



-- Auto attach on BufEnter
vim.api.nvim_create_autocmd("BufEnter", {
        callback = function(args)
                attach_to_buffer(args.buf)
        end,
})
return {}
