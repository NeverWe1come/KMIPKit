local M = {}

function M.setup(config)
  local cmd = { config.binary_path or "kmip-lsp", "--stdio" }
  if vim.lsp and vim.lsp.config then
    vim.lsp.config("kmip", {
      cmd = cmd,
      filetypes = config.filetypes or { "json", "rust", "java", "python", "c" },
      root_markers = { "Cargo.toml", ".git", "kmip-config.json" },
    })
    vim.lsp.enable("kmip")
  end
end

function M.get_client()
  if not vim.lsp or not vim.lsp.get_clients then
    return nil
  end
  local clients = vim.lsp.get_clients({ name = "kmip" })
  return clients[1]
end

return M
