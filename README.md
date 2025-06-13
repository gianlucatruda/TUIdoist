# TUIdoist: A blazingly fast Todoist client for your terminal

TUI 🤝 Todoist

<img width="600" alt="SCR-20250613-sgdj" src="https://github.com/user-attachments/assets/a147d180-d4a1-453c-8b48-51b6aaa0db5c" />

> [!WARNING]
> Highly-experimental alpha code that's mainly a learning exercise and should not yet be trusted with your real Todoist account.

## Goals

- Vim-like keybindings: hjkl movement, G/gg to go to bottom/top, space to check/uncheck tasks.
- Incredibly fast and responsive.
- Beautiful UI in the terminal.
- Robust to offline use.
- Tinker on a Rust project that's actually useful to me.


## Roadmap

- [x] Pull tasks from Todoist API in structured format
- [x] Display tasks as a list in the terminal with Ratatui
- [x] Pull _today_'s tasks from Todoist API in structured format
- [x] Display today's already-completed tasks (all tasks completed today) alongside active tasks 
- [x] Support for task descriptions (as 100char-truncated text)
- [x] Hit `r` to refresh (pull tasks from Todoist, update status)
- [ ] Ability to reorder tasks (not synced to API, local only) with `shift+j` and `shift+k` to move currently selected task down/up.
- [ ] Support for markdown URLs being rendered as rich hyperlinks
- [ ] Support for basic markdown being rendered as corresponding rich text
- [ ] Basic task completion functionality with spacebar (cached for 30 seconds before attempting to sync up to Todoist API, with easy undo)

---
