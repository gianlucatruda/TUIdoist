# TUIdoist: A blazingly fast Todoist client for your terminal

## Requirements

- Vim-like keybindings: hjkl movement, G/gg to go to bottom/top, space to check/uncheck tasks, '/' to search.
- Incredibly fast and responsive.
- Beautiful UI in the terminal, using the terminal's colour scheme.
- Robust to offline use (network and sync issues), with logic to avoid duplication or data loss


## Roadmap

- [x] Pull tasks from Todoist API in structured format
- [x] Display tasks as a list in the terminal with Ratatui
- [x] Pull _today_'s tasks from Todoist API in structured format
- [ ] Display today's already-completed tasks (all tasks completed today) alongside active tasks 
- [ ] Support for markdown and URLs being rendered nicely by UI
- [ ] Ability to reorganise task order (not synced to API, local only)
- [ ] Basic task completion functionality (cached for 30 seconds before attempting to sync up to Todoist API, with easy undo)

## Dev rules

- Clean, modular, maintainable Rust. Focus on readability, simplicity, and best practices.
- Safe Rust with minimal type masturbation.
- Minimal complexity. Don't add new libraries or artefacts without direct permission. Default to implementing things from scratch.

---
