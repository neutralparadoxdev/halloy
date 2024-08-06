# Sidebar

## `[sidebar]` Section

```toml
[sidebar]
default_action = "new-pane" | "replace-pane"
width = <integer>
show_unread_indicators = true | false
```

| Key                      | Description                                                                                              | Default      |
| ------------------------ | -------------------------------------------------------------------------------------------------------- | ------------ |
| `default_action`         | Action when selecting buffers in the sidebar. Can be `"new-pane"`, `"replace-pane"`, or `"toggle-pane"`. | `"new-pane"` |
| `width`                  | Specify sidebar width in pixels.                                                                         | `120`        |
| `show_unread_indicators` | Unread buffer indicators                                                                                 | `true`       |

## `[sidebar.buttons]` Section

```toml
[sidebar.buttons]
file_transfer = true | false
command_bar = true | false
reload_config = true | false
```

| Key             | Description                      | Default |
| --------------- | -------------------------------- | ------- |
| `file_transfer` | File transfer button in sidebar. | `true`  |
| `command_bar`   | Command bar button in sidebar.   | `true`  |
| `reload_config` | Reload config button in sidebar. | `true`  |
