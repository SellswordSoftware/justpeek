# Example Generation Plan

## Goal

Create one YAML reference file in `examples/` for every individual entry listed in [filestogenerate.md](/home/mike/sellsword/justpeek/docs/filestogenerate.md).

This plan is the tracking source for that work.

## Rules

- Every listed tool, app, platform, or reference set gets its own file.
- Each group gets its own folder under `examples/`.
- Each file lives inside its group folder and uses a lowercase kebab-case filename such as `examples/office-apps/google-docs.yaml`.
- Each file must set top-level `group` to the exact section heading from [filestogenerate.md](/home/mike/sellsword/justpeek/docs/filestogenerate.md).
- Existing examples should be reused where they fit, but normalized to the required top-level `group`.
- Slash-delimited entries are split into separate files unless the entry clearly names one product family that should stay together.

## Status Legend

- `[x]` file exists and matches the target category
- `[-]` file exists but needs normalization or expansion
- `[ ]` file does not exist yet

## Phase Order

1. Normalize existing example files that already map to target entries.
2. Add high-value core tooling examples first: source control, shells, package managers, browsers, editors.
3. Add broader app ecosystems next: infra, AI, office, creative, media, databases, API tools, communication.
4. Finish platform and long-tail reference packs.
5. Do a final validation pass against the schema in [referencefile.md](/home/mike/sellsword/justpeek/docs/referencefile.md).

## Existing File Adjustments

- `examples/source-control/git.yaml`: change top-level `group` from `CLI` to `source control`; keep as the tracked file for `git`.
- `examples/editors-and-ides/vscode.yaml`: normalized from the old root `examples/vscode.yaml`; top-level `group` is now `editors and ides`.
- `examples/people.yaml`: not part of the current generation list; leave out of completion tracking for now.
- `examples/justpeek.yaml`: not part of the current generation list; leave out of completion tracking for now.

## Tracking Checklist

### source control

- [x] `examples/source-control/git.yaml` -> `git`
- [x] `examples/source-control/github-cli.yaml` -> `github cli`
- [x] `examples/source-control/gitlab-cli.yaml` -> `gitlab cli`
- [x] `examples/source-control/jujutsu.yaml` -> `Jujutsu`
- [x] `examples/source-control/lazygit.yaml` -> `lazygit`
- [x] `examples/source-control/gitui.yaml` -> `gitui`

### shell / shell helpers

- [x] `examples/shell-shell-helpers/bash.yaml` -> `bash`
- [x] `examples/shell-shell-helpers/zsh.yaml` -> `zsh`
- [x] `examples/shell-shell-helpers/fish.yaml` -> `fish`
- [x] `examples/shell-shell-helpers/powershell.yaml` -> `powershell`
- [x] `examples/shell-shell-helpers/nushell.yaml` -> `nushell`
- [x] `examples/shell-shell-helpers/starship-prompt-config.yaml` -> `starship prompt config`
- [x] `examples/shell-shell-helpers/fzf.yaml` -> `fzf`
- [x] `examples/shell-shell-helpers/ripgrep.yaml` -> `ripgrep`
- [x] `examples/shell-shell-helpers/fd.yaml` -> `fd`
- [x] `examples/shell-shell-helpers/bat.yaml` -> `bat`
- [x] `examples/shell-shell-helpers/eza.yaml` -> `eza`
- [x] `examples/shell-shell-helpers/zoxide.yaml` -> `zoxide`
- [x] `examples/shell-shell-helpers/jq.yaml` -> `jq`
- [x] `examples/shell-shell-helpers/yq.yaml` -> `yq`
- [x] `examples/shell-shell-helpers/sed.yaml` -> `sed`
- [x] `examples/shell-shell-helpers/awk.yaml` -> `awk`
- [x] `examples/shell-shell-helpers/rsync.yaml` -> `rsync`
- [x] `examples/shell-shell-helpers/ssh.yaml` -> `ssh`
- [x] `examples/shell-shell-helpers/scp.yaml` -> `scp`
- [x] `examples/shell-shell-helpers/sftp.yaml` -> `sftp`
- [x] `examples/shell-shell-helpers/curl.yaml` -> `curl`
- [x] `examples/shell-shell-helpers/wget.yaml` -> `wget`
- [x] `examples/shell-shell-helpers/systemd.yaml` -> `systemd`
- [x] `examples/shell-shell-helpers/journalctl.yaml` -> `journalctl`
- [x] `examples/shell-shell-helpers/cron.yaml` -> `cron`
- [x] `examples/shell-shell-helpers/systemd-timers.yaml` -> `systemd timers`
- [x] `examples/shell-shell-helpers/wezterm.yaml` -> `wezterm`
- [x] `examples/shell-shell-helpers/ghostty.yaml` -> `ghostty`
- [x] `examples/shell-shell-helpers/nvim.yaml` -> `nvim`
- [x] `examples/shell-shell-helpers/tmux.yaml` -> `tmux`

### package managers

- [x] `examples/package-managers/apt.yaml` -> `apt`
- [x] `examples/package-managers/dnf.yaml` -> `dnf`
- [x] `examples/package-managers/pacman.yaml` -> `pacman`
- [x] `examples/package-managers/zypper.yaml` -> `zypper`
- [x] `examples/package-managers/brew.yaml` -> `brew`
- [x] `examples/package-managers/winget.yaml` -> `winget`
- [x] `examples/package-managers/scoop.yaml` -> `scoop`
- [x] `examples/package-managers/choco.yaml` -> `choco`
- [x] `examples/package-managers/flatpak.yaml` -> `flatpak`
- [x] `examples/package-managers/snap.yaml` -> `snap`
- [x] `examples/package-managers/nix.yaml` -> `nix`
- [x] `examples/package-managers/nixos-rebuild.yaml` -> `nixos-rebuild`
- [x] `examples/package-managers/home-manager.yaml` -> `home-manager`
- [x] `examples/package-managers/pip.yaml` -> `pip`
- [x] `examples/package-managers/pipx.yaml` -> `pipx`
- [x] `examples/package-managers/uv.yaml` -> `uv`
- [x] `examples/package-managers/poetry.yaml` -> `poetry`
- [x] `examples/package-managers/pnpm.yaml` -> `pnpm`
- [x] `examples/package-managers/yarn.yaml` -> `yarn`
- [x] `examples/package-managers/bun.yaml` -> `bun`
- [x] `examples/package-managers/cargo.yaml` -> `cargo`
- [x] `examples/package-managers/go.yaml` -> `go`
- [x] `examples/package-managers/dotnet.yaml` -> `dotnet`
- [x] `examples/package-managers/maven.yaml` -> `maven`
- [x] `examples/package-managers/gradle.yaml` -> `gradle`
- [x] `examples/package-managers/zig.yaml` -> `zig`
- [x] `examples/package-managers/npm.yaml` -> `npm`
- [x] `examples/package-managers/fnm.yaml` -> `fnm`
- [x] `examples/package-managers/nvm.yaml` -> `nvm`

### browsers

- [x] `examples/browsers/chrome.yaml` -> `chrome`
- [x] `examples/browsers/vivaldi.yaml` -> `vivaldi`
- [x] `examples/browsers/zen.yaml` -> `zen`
- [x] `examples/browsers/firefox.yaml` -> `firefox`
- [x] `examples/browsers/opera.yaml` -> `opera`
- [x] `examples/browsers/edge.yaml` -> `edge`
- [x] `examples/browsers/brave.yaml` -> `brave`
- [x] `examples/browsers/safari.yaml` -> `safari`

### container / infra

- [x] `examples/container-infra/docker.yaml` -> `docker`
- [x] `examples/container-infra/compose.yaml` -> `compose`
- [x] `examples/container-infra/podman.yaml` -> `podman`
- [x] `examples/container-infra/kubernetes.yaml` -> `kubernetes`
- [x] `examples/container-infra/kubectl.yaml` -> `kubectl`
- [x] `examples/container-infra/helm.yaml` -> `helm`
- [x] `examples/container-infra/terraform.yaml` -> `terraform`
- [x] `examples/container-infra/opentofu.yaml` -> `OpenTofu`
- [x] `examples/container-infra/ansible.yaml` -> `ansible`
- [x] `examples/container-infra/vagrant.yaml` -> `vagrant`
- [x] `examples/container-infra/qemu.yaml` -> `qemu`
- [x] `examples/container-infra/lxc.yaml` -> `lxc`

### editors and ides

- [x] `examples/editors-and-ides/visual-studio.yaml` -> `Visual Studio`
- [x] `examples/editors-and-ides/vscode.yaml` -> `VS Code`
- [x] `examples/editors-and-ides/eclipse.yaml` -> `Eclipse`
- [x] `examples/editors-and-ides/emacs.yaml` -> `Emacs`
- [x] `examples/editors-and-ides/helix.yaml` -> `Helix`
- [x] `examples/editors-and-ides/zed.yaml` -> `Zed`
- [x] `examples/editors-and-ides/sublime-text.yaml` -> `Sublime Text`
- [x] `examples/editors-and-ides/kate.yaml` -> `Kate`
- [x] `examples/editors-and-ides/notepad-plus-plus.yaml` -> `Notepad++`
- [x] `examples/editors-and-ides/cursor.yaml` -> `Cursor`
- [x] `examples/editors-and-ides/windsurf.yaml` -> `Windsurf`
- [x] `examples/editors-and-ides/android-studio.yaml` -> `Android Studio`
- [x] `examples/editors-and-ides/xcode.yaml` -> `Xcode`

### ai tools

- [x] `examples/ai-tools/claude-code.yaml` -> `Claude Code`
- [x] `examples/ai-tools/openai-codex-cli.yaml` -> `OpenAI Codex CLI`
- [x] `examples/ai-tools/gemini-cli.yaml` -> `Gemini Cli`
- [x] `examples/ai-tools/aider.yaml` -> `Aider`
- [x] `examples/ai-tools/continue.yaml` -> `Continue`
- [x] `examples/ai-tools/copilot-cli.yaml` -> `Copilot CLI`
- [x] `examples/ai-tools/ollama.yaml` -> `Ollama`
- [x] `examples/ai-tools/llama-cpp-server.yaml` -> `llama.cpp server`
- [x] `examples/ai-tools/open-webui.yaml` -> `Open WebUI`

### os shortcuts

- [x] `examples/os-shortcuts/win11.yaml` -> `Win11`
- [x] `examples/os-shortcuts/macos.yaml` -> `MacOS`
- [x] `examples/os-shortcuts/gnome.yaml` -> `GNOME`
- [x] `examples/os-shortcuts/kde-plasma.yaml` -> `KDE Plasma`
- [x] `examples/os-shortcuts/i3.yaml` -> `i3`
- [x] `examples/os-shortcuts/sway.yaml` -> `Sway`
- [x] `examples/os-shortcuts/hyprland.yaml` -> `Hyprland`
- [x] `examples/os-shortcuts/xfce.yaml` -> `XFCE`
- [x] `examples/os-shortcuts/cinnamon.yaml` -> `Cinnamon`
- [x] `examples/os-shortcuts/pop-shell.yaml` -> `Pop shell`

### launchers / productivity Tools

- [x] `examples/launchers-productivity-tools/powertoys.yaml` -> `PowerToys`
- [x] `examples/launchers-productivity-tools/autohotkey.yaml` -> `AutoHotkey`

### notetaking and knowledge tools

- [x] `examples/notetaking-and-knowledge-tools/obsidian.yaml` -> `Obsidian`
- [x] `examples/notetaking-and-knowledge-tools/logseq.yaml` -> `Logseq`
- [x] `examples/notetaking-and-knowledge-tools/notion.yaml` -> `Notion`
- [x] `examples/notetaking-and-knowledge-tools/joplin.yaml` -> `Joplin`
- [x] `examples/notetaking-and-knowledge-tools/onenote.yaml` -> `OneNote`
- [x] `examples/notetaking-and-knowledge-tools/silverbullet.yaml` -> `SilverBullet`
- [x] `examples/notetaking-and-knowledge-tools/anytype.yaml` -> `Anytype`
- [x] `examples/notetaking-and-knowledge-tools/trillium.yaml` -> `Trillium`

### office apps

- [ ] `examples/word.yaml` -> `Word`
- [ ] `examples/excel.yaml` -> `Excel`
- [ ] `examples/powerpoint.yaml` -> `Powerpoint`
- [ ] `examples/outlook.yaml` -> `Outlook`
- [ ] `examples/google-sheets.yaml` -> `Google Sheets`
- [ ] `examples/libreoffice-calc.yaml` -> `LibreOffice Calc`
- [ ] `examples/libreoffice-writer.yaml` -> `LibreOffice Writer`
- [ ] `examples/google-docs.yaml` -> `Google Docs`
- [ ] `examples/google-slides.yaml` -> `Google Slides`
- [ ] `examples/thunderbird.yaml` -> `Thunderbird`
- [ ] `examples/gmail-web.yaml` -> `Gmail Web`

### creative apps

- [ ] `examples/figma.yaml` -> `Figma`
- [ ] `examples/inkscape.yaml` -> `Inkscape`
- [ ] `examples/krita.yaml` -> `Krita`
- [ ] `examples/affinity-designer.yaml` -> `Affinity Designer`
- [ ] `examples/affinity-photo.yaml` -> `Affinity Photo`
- [ ] `examples/photoship.yaml` -> `Photoship`
- [ ] `examples/illustrator.yaml` -> `Illustrator`
- [ ] `examples/gimp.yaml` -> `Gimp`
- [ ] `examples/darktable.yaml` -> `Darktable`
- [ ] `examples/rawtherapee.yaml` -> `RawTherapee`
- [ ] `examples/photopea.yaml` -> `Photopea`

### video / audio

- [ ] `examples/davinci-resolve.yaml` -> `DaVinci Resolve`
- [ ] `examples/kdenlive.yaml` -> `Kdenlive`
- [ ] `examples/obs-studio.yaml` -> `OBS Studio`
- [ ] `examples/audacity.yaml` -> `Audacity`
- [ ] `examples/reaper.yaml` -> `Reaper`
- [ ] `examples/fl-studio.yaml` -> `FL Studio`
- [ ] `examples/ableton-live.yaml` -> `Ableton Live`
- [ ] `examples/ardour.yaml` -> `Ardour`
- [ ] `examples/lmms.yaml` -> `LMMS`
- [ ] `examples/handbrake.yaml` -> `HandBrake`
- [ ] `examples/ffmpeg.yaml` -> `FFmpeg`

### cad / 3d / maker tools

- [ ] `examples/freecad.yaml` -> `FreeCAD`
- [ ] `examples/fusion-360.yaml` -> `Fusion 360`
- [ ] `examples/openscad.yaml` -> `OpenSCAD`
- [ ] `examples/kicad.yaml` -> `KiCad`
- [ ] `examples/prusaslicer.yaml` -> `PrusaSlicer`
- [ ] `examples/orcaslicer.yaml` -> `OrcaSlicer`
- [ ] `examples/cura.yaml` -> `Cura`
- [ ] `examples/meshlab.yaml` -> `MeshLab`
- [ ] `examples/sweet-home-3d.yaml` -> `Sweet Home 3d`
- [ ] `examples/blender.yaml` -> `Blender`
- [ ] `examples/godot.yaml` -> `godot`
- [ ] `examples/unity.yaml` -> `unity`

### programming language / toolchain packs

- [ ] `examples/python.yaml` -> `python`
- [ ] `examples/go-toolchain.yaml` -> `Go`
- [ ] `examples/java-toolchain.yaml` -> `java`
- [ ] `examples/c-cpp-toolchain.yaml` -> `c/c++`
- [ ] `examples/zig-toolchain.yaml` -> `zig`
- [ ] `examples/deno.yaml` -> `deno`
- [ ] `examples/bun-toolchain.yaml` -> `bun`
- [ ] `examples/php.yaml` -> `php`
- [ ] `examples/ruby.yaml` -> `ruby`
- [ ] `examples/elixir.yaml` -> `elixir`
- [ ] `examples/lua.yaml` -> `lua`
- [ ] `examples/julia.yaml` -> `julia`
- [ ] `examples/r.yaml` -> `r`
- [ ] `examples/haskell.yaml` -> `haskell/cabal/stack`
- [ ] `examples/kotlin.yaml` -> `kotlin/gradle`
- [ ] `examples/swift.yaml` -> `swift`

### testing tools

- [ ] `examples/pytest.yaml` -> `pytest`
- [ ] `examples/unittest.yaml` -> `unittest`
- [ ] `examples/jest.yaml` -> `jest`
- [ ] `examples/vitest.yaml` -> `vitest`
- [ ] `examples/mocha.yaml` -> `mocha`
- [ ] `examples/playwright.yaml` -> `playwright`
- [ ] `examples/cypress.yaml` -> `cypress`
- [ ] `examples/cargo-test.yaml` -> `cargo test`
- [ ] `examples/go-test.yaml` -> `go test`
- [ ] `examples/junit.yaml` -> `junit`
- [ ] `examples/nunint.yaml` -> `nunint`
- [ ] `examples/postman-cli.yaml` -> `postman cli`

### build tools

- [ ] `examples/make.yaml` -> `make`
- [ ] `examples/cmake.yaml` -> `cmake`
- [ ] `examples/meson.yaml` -> `meson`
- [ ] `examples/ninja.yaml` -> `ninja`
- [ ] `examples/just.yaml` -> `just`
- [ ] `examples/task.yaml` -> `task`
- [ ] `examples/bazel.yaml` -> `bazel`
- [ ] `examples/gradle-build.yaml` -> `gradle`
- [ ] `examples/maven-build.yaml` -> `maven`

### web / devops / app tools

- [ ] `examples/vite.yaml` -> `vite`
- [ ] `examples/nextjs.yaml` -> `next.js`
- [ ] `examples/react.yaml` -> `react`
- [ ] `examples/vue.yaml` -> `vue`
- [ ] `examples/svelte.yaml` -> `svelte`
- [ ] `examples/sveltekit.yaml` -> `sveltekit`
- [ ] `examples/astro.yaml` -> `astro`
- [ ] `examples/angular.yaml` -> `angular`
- [ ] `examples/express.yaml` -> `express`
- [ ] `examples/fastify.yaml` -> `fastify`
- [ ] `examples/django.yaml` -> `django`
- [ ] `examples/flask.yaml` -> `flask`
- [ ] `examples/fastapi.yaml` -> `fastAPI`
- [ ] `examples/rails.yaml` -> `rails`
- [ ] `examples/laravel.yaml` -> `laravel`
- [ ] `examples/tauri.yaml` -> `tauri`
- [ ] `examples/wails.yaml` -> `wails`
- [ ] `examples/electron.yaml` -> `electron`
- [ ] `examples/capacitor.yaml` -> `capacitor`

### database

- [ ] `examples/postgres.yaml` -> `postgres`
- [ ] `examples/sqlite.yaml` -> `sqlite`
- [ ] `examples/mysql.yaml` -> `mysql`
- [ ] `examples/redis.yaml` -> `redis`
- [ ] `examples/mongodb.yaml` -> `mongodb`
- [ ] `examples/duckdb.yaml` -> `duckdb`
- [ ] `examples/clickhouse.yaml` -> `clickhouse`
- [ ] `examples/dbeaver.yaml` -> `dbeaver`
- [ ] `examples/pgadmin.yaml` -> `pgadmin`
- [ ] `examples/beekeeper-studio.yaml` -> `beekeeper studio`

### api tools

- [ ] `examples/postman.yaml` -> `Postman`
- [ ] `examples/insomnia.yaml` -> `Insomnia`
- [ ] `examples/bruno.yaml` -> `Bruno`
- [ ] `examples/httpie.yaml` -> `HTTPie`
- [ ] `examples/curl-api.yaml` -> `Curl`
- [ ] `examples/grpcurl.yaml` -> `grpcurl`
- [ ] `examples/openapi-generator.yaml` -> `openapi generator`

### communication apps

- [ ] `examples/slack.yaml` -> `Slack`
- [ ] `examples/discord.yaml` -> `Discord`
- [ ] `examples/teams.yaml` -> `Teams`
- [ ] `examples/zoom.yaml` -> `Zoom`
- [ ] `examples/google-meet.yaml` -> `Google Meet`
- [ ] `examples/signal-desktop.yaml` -> `Signal Desktop`
- [ ] `examples/telegram-desktop.yaml` -> `Telegram Desktop`
- [ ] `examples/mattermost.yaml` -> `Mattermost`
- [ ] `examples/element.yaml` -> `Element`
- [ ] `examples/matrix.yaml` -> `Matrix`
- [ ] `examples/linear.yaml` -> `Linear`
- [ ] `examples/jira.yaml` -> `Jira`
- [ ] `examples/trello.yaml` -> `Trello`
- [ ] `examples/github-projects.yaml` -> `Github Projects`

### gaming / streaming

- [ ] `examples/steam.yaml` -> `Steam`
- [ ] `examples/discord-gaming.yaml` -> `discord`
- [ ] `examples/obs-studio-streaming.yaml` -> `obs studio`
- [ ] `examples/prism-launcher.yaml` -> `prism launcher`
- [ ] `examples/minecraft-launchers.yaml` -> `minecraft launchers`
- [ ] `examples/retroarch.yaml` -> `retroarch`
- [ ] `examples/lutris.yaml` -> `lutris`
- [ ] `examples/heroic-games-launcher.yaml` -> `heroic games launcher`
- [ ] `examples/mangohud.yaml` -> `mangohud`
- [ ] `examples/gamemode.yaml` -> `gamemode`

## Completion Criteria

- Every checklist item above has a corresponding file in `examples/`.
- Every generated file validates against the schema described in [referencefile.md](/home/mike/sellsword/justpeek/docs/referencefile.md).
- Every generated file uses the exact intended top-level `group`.
- The two pre-existing tracked files, `git.yaml` and `vscode.yaml`, are normalized before marking their items complete.
