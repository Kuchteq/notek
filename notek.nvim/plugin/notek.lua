if vim.g.loaded_notek then
  return
end
vim.g.loaded_notek = true

local notek = require("notek")

local group = vim.api.nvim_create_augroup("notek", { clear = true })

-- Auto-attach to file buffers on BufEnter
vim.api.nvim_create_autocmd("BufEnter", {
  group = group,
  callback = function(args)
    local config = require("notek.config")
    if config.values.auto_attach then
      notek.attach(args.buf)
    end
  end,
})

-- Flush on save
vim.api.nvim_create_autocmd("BufWritePost", {
  group = group,
  callback = function(args)
    notek.attach(args.buf)
    notek.flush(args.buf)
  end,
})

-- Clean up all connections on exit
vim.api.nvim_create_autocmd("VimLeavePre", {
  group = group,
  callback = function()
    notek.shutdown()
  end,
})

-- User commands
vim.api.nvim_create_user_command("NotekAttach", function()
  notek.attach(vim.api.nvim_get_current_buf())
end, { desc = "Attach current buffer to notek" })

vim.api.nvim_create_user_command("NotekDetach", function()
  notek.detach(vim.api.nvim_get_current_buf())
end, { desc = "Detach current buffer from notek" })
