# Tape for VHS to record the QuotaChecker TUI demo
Output docs/demo.gif

Set FontSize 15
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 12

# Hide the command to launch the app
Hide
Type "./target/release/quotachecker-tui"
Enter
Sleep 1.5s
Show

# --- Overview Tab ---
# Show the beautiful dashboard overview with system telemetry, usage charts, and activity log.
Sleep 4.5s

# --- AI Agents Tab ---
# Switch to the AI Agents tab.
Tab
Sleep 1.5s
# Use Down/Up arrows to highlight different agents in the list.
Down
Sleep 1s
Down
Sleep 1s
Up
Sleep 1.5s

# --- Sessions Tab ---
# Switch to Sessions.
Tab
Sleep 2.5s

# --- Quotas Tab ---
# Switch to Quotas.
Tab
Sleep 2.5s

# --- Settings Tab ---
# Switch to Settings.
Tab
Sleep 1.5s
# Highlight setting options.
Down
Sleep 0.8s
Down
Sleep 0.8s
# Modify settings with +/-
Type "+"
Sleep 1s
Type "+"
Sleep 1s
Type "-"
Sleep 1.5s

# --- Add Budget Modal Demo ---
# Go back to Overview tab (shortcut '1')
Type "1"
Sleep 1.5s
# Open the budget edit modal (shortcut 's')
Type "s"
Sleep 2s
# Edit the budget value
Type "800"
Sleep 1.5s
# Confirm budget change
Enter
Sleep 3s

# Quit the application
Type "q"
Sleep 1s
