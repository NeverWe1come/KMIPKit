if vim.g.loaded_kmipkit then
  return
end
vim.g.loaded_kmipkit = true

local ttlv = require("kmip.ttlv")

vim.api.nvim_create_user_command("KmipDecode", function(opts)
  local text = opts.args
  if text == "" and opts.range > 0 then
    local lines = vim.api.nvim_buf_get_lines(0, opts.line1 - 1, opts.line2, false)
    text = table.concat(lines, "")
  end
  if text == "" then
    vim.notify("Usage: :'<,'>KmipDecode or :KmipDecode <hex>", vim.log.levels.INFO)
    return
  end
  ttlv.decode_selection(text)
end, { nargs = "?", range = true })

vim.api.nvim_create_user_command("KmipLookup", function(opts)
  ttlv.lookup_spec(opts.args)
end, { nargs = 1 })

vim.api.nvim_create_user_command("KmipPing", function()
  local client = require("kmip.lsp").get_client()
  if client then
    vim.notify("KMIPKit: kmip-lsp active (id=" .. client.id .. ")", vim.log.levels.INFO)
  else
    vim.notify("KMIPKit: kmip-lsp not active", vim.log.levels.WARN)
  end
end, {})
