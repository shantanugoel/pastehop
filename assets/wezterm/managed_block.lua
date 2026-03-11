-- BEGIN PASTEHOP MANAGED BLOCK
config = config or {}
config.keys = config.keys or {}

table.insert(config.keys, {
  key = "v",
  mods = "CTRL",
  action = wezterm.action_callback(function(window, pane)
    local success, stdout, stderr = wezterm.run_child_process({
      "__PH_BINARY__",
      "hook",
      "wezterm",
      "--key",
      "CTRL+V",
      "--domain",
      tostring(pane:get_domain_name() or ""),
      "--foreground-process",
      tostring(pane:get_foreground_process_name() or ""),
      "--cwd",
      tostring(pane:get_current_working_dir() or ""),
    })

    if not success then
      window:toast_notification("pastehop", stderr or "hook failed", nil, 3000)
      window:perform_action(wezterm.action.SendKey({ key = "v", mods = "CTRL" }), pane)
      return
    end

    local response = wezterm.json_parse(stdout)
    if response.action == "inject_text" and response.text then
      pane:send_paste(response.text)
    elseif response.action == "passthrough_key" then
      window:perform_action(wezterm.action.SendKey({ key = "v", mods = "CTRL" }), pane)
    elseif response.action == "error" and response.message then
      window:toast_notification("pastehop", response.message, nil, 3000)
    end
  end),
})
-- END PASTEHOP MANAGED BLOCK
