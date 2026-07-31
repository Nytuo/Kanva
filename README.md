<h1 align="center">
  <a href="https://github.com/Nytuo/Kanva">
    <img src="assets/logo.png" alt="Logo" width="auto" height="200">
  </a>
</h1>
<div align="center">
<h2>Kanva</h2>
Self-hostable Kanban boards with Markdown notes, calendar, and team collaboration
  <br />
  <br />
  <a href="https://github.com/Nytuo/Kanva/issues/new?assignees=&labels=bug&title=bug%3A+">Report a Bug</a>
  ·
  <a href="https://github.com/Nytuo/Kanva/issues/new?assignees=&labels=enhancement&title=feat%3A+">Request a Feature</a>
    · <a href="https://github.com/Nytuo/Kanva/discussions">Ask a Question</a>

</div>

<div align="center">
<br />

[![Project license](https://img.shields.io/github/license/Nytuo/Kanva.svg?style=flat-square)](LICENSE)

[![code with love by Nytuo](https://img.shields.io/badge/%3C%2F%3E%20with%20%E2%99%A5%20by-Nytuo-ff1414.svg?style=flat-square)](https://github.com/Nytuo)

</div>

<details open="open">
<summary>Table of Contents</summary>

- [About](#about)
- [What Kanva Can Do](#what-kanva-can-do)
- [Architecture](#architecture)
- [Technologies](#technologies)
- [Getting Started](#getting-started)
  - [Self-hosted (Docker Compose)](#self-hosted-docker-compose)
  - [Desktop / Mobile (standalone)](#desktop--mobile-standalone)
  - [Local development](#local-development)
- [Authors \& contributors](#authors--contributors)
- [License](#license)

</details>

---

## About

Kanva is a Trello/Notion-flavored project management app: Kanban boards for planning work, and Markdown notes — per-project or global — for everything that doesn't fit on a card. It runs however you want it to: as a self-hosted server for a team, or as a fully local, no-server-required desktop and mobile app.

## What Kanva Can Do

- **Kanban boards**
  - Lists and cards with drag-and-drop, priorities, due/start dates, cover colors, labels, and custom fields
  - Checklists, comments, file attachments, and per-board activity logs
  - Board templates, starring, archiving, and private / team / public visibility
  - Real-time updates over WebSocket when a board is shared with a team

- **Notes, Notion/Obsidian-style**
  - Global notes that are yours alone, and per-project notes shared with everyone on that board
  - Everything is Markdown, with a live write/preview split

- **Calendar** — due dates and custom events across your boards in one timeline

- **Teams** — invite members, assign roles (owner/admin/member/viewer), and scope boards to a team (optional; disabled entirely in standalone mode)

- **Integrations** — import issues from GitHub, GitLab, or Atlassian (Jira) straight onto a board

- **Runs anywhere**
  - **Web**: point it at any Kanva server
  - **Desktop & Mobile** (Tauri): an embedded, standalone Kanva server with a local SQLite database — no external server needed
  - **Self-hosted**: Docker Compose stack with Postgres + Redis for teams at scale

## Architecture

```
Kanva/
├── server/    # Rust (Axum) API — Postgres or SQLite via a single AnyPool backend
├── web/       # React + Vite + TypeScript SPA (shared by browser, desktop, and mobile)
├── desktop/   # Tauri v2 desktop shell, embeds the server for standalone use
├── mobile/    # Tauri v2 mobile shell (remote-server only)
└── migrations/ # SQL schema, split into postgres/ and sqlite/ variants
```

## Technologies
<div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
  <img src="https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust"/>
  <img src="https://img.shields.io/badge/Axum-black?style=for-the-badge"/>
  <img src="https://img.shields.io/badge/PostgreSQL-black?style=for-the-badge&logo=postgresql"/>
  <img src="https://img.shields.io/badge/SQLite-black?style=for-the-badge&logo=sqlite"/>
  <img src="https://img.shields.io/badge/Redis-black?style=for-the-badge&logo=redis"/>
  <img src="https://img.shields.io/badge/React-black?style=for-the-badge&logo=React"/>
  <img src="https://img.shields.io/badge/vite-black?style=for-the-badge&logo=vite"/>
  <img src="https://img.shields.io/badge/typeScript-black?style=for-the-badge&logo=typescript"/>
  <img src="https://img.shields.io/badge/TailwindCSS-black?style=for-the-badge&logo=tailwindcss"/>
  <img src="https://img.shields.io/badge/TAURI-black?style=for-the-badge&logo=tauri"/>
  <img src="https://img.shields.io/badge/Docker-black?style=for-the-badge&logo=docker"/>
</div>

## Getting Started

### Self-hosted (Docker Compose)

```bash
cp .env.example .env   # fill in JWT_SECRET and any OAuth credentials you use
docker compose up -d
```

This starts Postgres, Redis, the Kanva server (`:8080`), and the web app (`:3000`).

> **Before going to production**, set a strong, unique `JWT_SECRET` and a proper `CORS_ORIGIN` — the values in `docker-compose.yml`/`.env.example` are development placeholders only.

### Desktop / Mobile (standalone)

The desktop and mobile apps embed the Kanva server directly and store data locally in SQLite — no `docker-compose` or separate server required. Build them from `desktop/` or `mobile/` with the Tauri CLI:

```bash
cd desktop && cargo tauri dev     # or: cargo tauri build
```

### Local development

```bash
# Server
cd server && cargo run

# Web
cd web && npm install && npm run dev
```

## Authors & contributors

The original setup of this repository is by [Arnaud BEUX](https://github.com/Nytuo).

For a full list of all authors and contributors, see [the contributors page](https://github.com/Nytuo/Kanva/contributors).

## License

Kanva is licensed under the **GNU General Public License v3**.
Kanva is provided **"as is"** without any **warranty**. Use at your own risk.
See [LICENSE](LICENSE) for more information.
