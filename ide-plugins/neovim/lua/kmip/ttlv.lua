local lsp = require("kmip.lsp")

local M = {}
local current_config = {}

function M.setup(config)
  current_config = config or {}
end

local function format_node(node, indent, lines)
  local pad = string.rep("  ", indent)
  local line = string.format(
    "%s%s %s [%s, %dB] = %s",
    pad,
    node.tagHex or "",
    node.tagName or "",
    node.itemType or "",
    node.valueLength or 0,
    node.displayValue or ""
  )
  table.insert(lines, line)
  for _, child in ipairs(node.children or {}) do
    format_node(child, indent + 1, lines)
  end
end

local function open_floating_lines(title, lines)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].modifiable = false
  vim.bo[buf].filetype = "markdown"

  local width = math.min(100, math.floor(vim.o.columns * 0.8))
  local height = math.min(#lines + 2, math.floor(vim.o.lines * 0.7))
  vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    row = math.floor((vim.o.lines - height) / 2),
    col = math.floor((vim.o.columns - width) / 2),
    style = "minimal",
    border = "rounded",
    title = " " .. title .. " ",
    title_pos = "center",
  })
end

function M.decode_selection(input_text)
  local client = lsp.get_client()
  if not client then
    vim.notify("KMIPKit: kmip-lsp is not attached to current buffer", vim.log.levels.WARN)
    return
  end

  client:request("kmip/decodeTtlv", { input = input_text, format = "auto" }, function(err, result)
    if err then
      vim.notify("KMIPKit decode error: " .. (err.message or vim.inspect(err)), vim.log.levels.ERROR)
      return
    end
    local lines = { "# KMIP TTLV Inspector (" .. tostring(result.byteLength) .. " bytes)", "" }
    if result.alerts and #result.alerts > 0 then
      table.insert(lines, "## Alerts")
      for _, alert in ipairs(result.alerts) do
        table.insert(lines, "- [" .. alert.code .. "] " .. alert.message)
      end
      table.insert(lines, "")
    end
    format_node(result.root, 0, lines)
    open_floating_lines("KMIP TTLV Inspector", lines)
  end, 0)
end

function M.lookup_spec(query)
  local client = lsp.get_client()
  if not client then
    vim.notify("KMIPKit: kmip-lsp is not attached", vim.log.levels.WARN)
    return
  end
  client:request("kmip/lookupSpec", { query = query }, function(err, result)
    if err then
      vim.notify("KMIPKit lookup error: " .. (err.message or ""), vim.log.levels.ERROR)
      return
    end
    local lines = { "# KMIP OASIS Lookup: " .. query, "" }
    for _, entry in ipairs(result or {}) do
      table.insert(lines, string.format("- **%s** (`%s`) [%s] — %s", entry.oasisName, entry.tagHex, entry.versions, entry.summary))
    end
    open_floating_lines("KMIP OASIS Spec", lines)
  end, 0)
end

return M
