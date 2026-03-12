-- BEGIN PASTEHOP MANAGED BLOCK
config = config or {}
config.keys = config.keys or {}

local ph_toast_timeout_ms = 10000

table.insert(config.keys, {
  key = "v",
  mods = "CTRL",
  action = wezterm.action_callback(function(window, pane)
    local fg_process = tostring(pane:get_foreground_process_name() or "")
    local ok, info = pcall(pane.get_foreground_process_info, pane)
    if ok and info and info.argv then
      fg_process = table.concat(info.argv, " ")
    end

    local success, stdout, stderr = wezterm.run_child_process({
      "__PH_BINARY__",
      "hook",
      "wezterm",
      "--key",
      "CTRL+V",
      "--domain",
      tostring(pane:get_domain_name() or ""),
      "--foreground-process",
      fg_process,
      "--cwd",
      tostring(pane:get_current_working_dir() or ""),
    })

    if not success then
      window:toast_notification("pastehop", stderr or "hook failed", nil, ph_toast_timeout_ms)
      window:perform_action(wezterm.action.PasteFrom("Clipboard"), pane)
      return
    end

    local response = wezterm.json_parse(stdout)
    if response.action == "inject_text" and response.text then
      pane:send_paste(response.text)
    elseif response.action == "passthrough_key" then
      window:perform_action(wezterm.action.PasteFrom("Clipboard"), pane)
    elseif response.action == "error" and response.message then
      window:toast_notification("pastehop", response.message, nil, ph_toast_timeout_ms)
    end
  end),
})
-- END PASTEHOP MANAGED BLOCK
