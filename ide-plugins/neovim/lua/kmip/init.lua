local lsp = require("kmip.lsp")
local ttlv = require("kmip.ttlv")

local M = {}

M.config = {
  binary_path = "kmip-lsp",
  filetypes = { "kmip-json", "json", "rust", "java", "python", "c" },
}

function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})
  lsp.setup(M.config)
  ttlv.setup(M.config)
end

return M
