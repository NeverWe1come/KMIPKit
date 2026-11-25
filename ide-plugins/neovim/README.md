# KMIPKit Neovim Plugin (`ide-plugins/neovim/`)

Lightweight Lua plugin for **Neovim 0.11+** using native `vim.lsp.config` and `kmip-lsp`.

## Installation (`lazy.nvim`)
```lua
{
  dir = "/path/to/KMIPKit/ide-plugins/neovim",
  config = function()
    require("kmip").setup({
      binary_path = "kmip-lsp",
    })
  end,
}
```

## Commands
- `:'<,'>KmipDecode` / `:KmipDecode <hex>` — Decode TTLV into a floating Markdown buffer (sensitive fields redacted by default).
- `:KmipLookup <tag-or-name>` — Query the OASIS KMIP 1.0–3.0 specification catalog.
- `:KmipPing` — Verify `kmip-lsp` attachment status.
