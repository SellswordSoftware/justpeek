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

- [x] `examples/office-apps/word.yaml` -> `Word`
- [x] `examples/office-apps/excel.yaml` -> `Excel`
- [x] `examples/office-apps/powerpoint.yaml` -> `Powerpoint`
- [x] `examples/office-apps/outlook.yaml` -> `Outlook`
- [x] `examples/office-apps/google-sheets.yaml` -> `Google Sheets`
- [x] `examples/office-apps/libreoffice-calc.yaml` -> `LibreOffice Calc`
- [x] `examples/office-apps/libreoffice-writer.yaml` -> `LibreOffice Writer`
- [x] `examples/office-apps/google-docs.yaml` -> `Google Docs`
- [x] `examples/office-apps/google-slides.yaml` -> `Google Slides`
- [x] `examples/office-apps/thunderbird.yaml` -> `Thunderbird`
- [x] `examples/office-apps/gmail-web.yaml` -> `Gmail Web`

### creative apps

- [x] `examples/creative-apps/figma.yaml` -> `Figma`
- [x] `examples/creative-apps/inkscape.yaml` -> `Inkscape`
- [x] `examples/creative-apps/krita.yaml` -> `Krita`
- [x] `examples/creative-apps/affinity-designer.yaml` -> `Affinity Designer`
- [x] `examples/creative-apps/affinity-photo.yaml` -> `Affinity Photo`
- [x] `examples/creative-apps/photoship.yaml` -> `Photoship`
- [x] `examples/creative-apps/illustrator.yaml` -> `Illustrator`
- [x] `examples/creative-apps/gimp.yaml` -> `Gimp`
- [x] `examples/creative-apps/darktable.yaml` -> `Darktable`
- [x] `examples/creative-apps/rawtherapee.yaml` -> `RawTherapee`
- [x] `examples/creative-apps/photopea.yaml` -> `Photopea`

### video / audio

- [x] `examples/video-audio/davinci-resolve.yaml` -> `DaVinci Resolve`
- [x] `examples/video-audio/kdenlive.yaml` -> `Kdenlive`
- [x] `examples/video-audio/obs-studio.yaml` -> `OBS Studio`
- [x] `examples/video-audio/audacity.yaml` -> `Audacity`
- [x] `examples/video-audio/reaper.yaml` -> `Reaper`
- [x] `examples/video-audio/fl-studio.yaml` -> `FL Studio`
- [x] `examples/video-audio/ableton-live.yaml` -> `Ableton Live`
- [x] `examples/video-audio/ardour.yaml` -> `Ardour`
- [x] `examples/video-audio/lmms.yaml` -> `LMMS`
- [x] `examples/video-audio/handbrake.yaml` -> `HandBrake`
- [x] `examples/video-audio/ffmpeg.yaml` -> `FFmpeg`

### cad / 3d / maker tools

- [x] `examples/cad-3d-maker-tools/freecad.yaml` -> `FreeCAD`
- [x] `examples/cad-3d-maker-tools/fusion-360.yaml` -> `Fusion 360`
- [x] `examples/cad-3d-maker-tools/openscad.yaml` -> `OpenSCAD`
- [x] `examples/cad-3d-maker-tools/kicad.yaml` -> `KiCad`
- [x] `examples/cad-3d-maker-tools/prusaslicer.yaml` -> `PrusaSlicer`
- [x] `examples/cad-3d-maker-tools/orcaslicer.yaml` -> `OrcaSlicer`
- [x] `examples/cad-3d-maker-tools/cura.yaml` -> `Cura`
- [x] `examples/cad-3d-maker-tools/meshlab.yaml` -> `MeshLab`
- [x] `examples/cad-3d-maker-tools/sweet-home-3d.yaml` -> `Sweet Home 3d`
- [x] `examples/cad-3d-maker-tools/blender.yaml` -> `Blender`
- [x] `examples/cad-3d-maker-tools/godot.yaml` -> `godot`
- [x] `examples/cad-3d-maker-tools/unity.yaml` -> `unity`

### programming language / toolchain packs

- [x] `examples/programming-language-toolchain-packs/python.yaml` -> `python`
- [x] `examples/programming-language-toolchain-packs/go-toolchain.yaml` -> `Go`
- [x] `examples/programming-language-toolchain-packs/java-toolchain.yaml` -> `java`
- [x] `examples/programming-language-toolchain-packs/c-cpp-toolchain.yaml` -> `c/c++`
- [x] `examples/programming-language-toolchain-packs/zig-toolchain.yaml` -> `zig`
- [x] `examples/programming-language-toolchain-packs/deno.yaml` -> `deno`
- [x] `examples/programming-language-toolchain-packs/bun-toolchain.yaml` -> `bun`
- [x] `examples/programming-language-toolchain-packs/php.yaml` -> `php`
- [x] `examples/programming-language-toolchain-packs/ruby.yaml` -> `ruby`
- [x] `examples/programming-language-toolchain-packs/elixir.yaml` -> `elixir`
- [x] `examples/programming-language-toolchain-packs/lua.yaml` -> `lua`
- [x] `examples/programming-language-toolchain-packs/julia.yaml` -> `julia`
- [x] `examples/programming-language-toolchain-packs/r.yaml` -> `r`
- [x] `examples/programming-language-toolchain-packs/haskell.yaml` -> `haskell/cabal/stack`
- [x] `examples/programming-language-toolchain-packs/kotlin.yaml` -> `kotlin/gradle`
- [x] `examples/programming-language-toolchain-packs/swift.yaml` -> `swift`

### testing tools

- [x] `examples/testing-tools/pytest.yaml` -> `pytest`
- [x] `examples/testing-tools/unittest.yaml` -> `unittest`
- [x] `examples/testing-tools/jest.yaml` -> `jest`
- [x] `examples/testing-tools/vitest.yaml` -> `vitest`
- [x] `examples/testing-tools/mocha.yaml` -> `mocha`
- [x] `examples/testing-tools/playwright.yaml` -> `playwright`
- [x] `examples/testing-tools/cypress.yaml` -> `cypress`
- [x] `examples/testing-tools/cargo-test.yaml` -> `cargo test`
- [x] `examples/testing-tools/go-test.yaml` -> `go test`
- [x] `examples/testing-tools/junit.yaml` -> `junit`
- [x] `examples/testing-tools/nunint.yaml` -> `nunint`

### build tools

- [x] `examples/build-tools/make.yaml` -> `make`
- [x] `examples/build-tools/cmake.yaml` -> `cmake`
- [x] `examples/build-tools/meson.yaml` -> `meson`
- [x] `examples/build-tools/ninja.yaml` -> `ninja`
- [x] `examples/build-tools/just.yaml` -> `just`
- [x] `examples/build-tools/task.yaml` -> `task`
- [x] `examples/build-tools/bazel.yaml` -> `bazel`
- [x] `examples/build-tools/gradle-build.yaml` -> `gradle`
- [x] `examples/build-tools/maven-build.yaml` -> `maven`

### web / devops / app tools

- [x] `examples/web-devops-app-tools/vite.yaml` -> `vite`
- [x] `examples/web-devops-app-tools/nextjs.yaml` -> `next.js`
- [x] `examples/web-devops-app-tools/react.yaml` -> `react`
- [x] `examples/web-devops-app-tools/vue.yaml` -> `vue`
- [x] `examples/web-devops-app-tools/svelte.yaml` -> `svelte`
- [x] `examples/web-devops-app-tools/sveltekit.yaml` -> `sveltekit`
- [x] `examples/web-devops-app-tools/astro.yaml` -> `astro`
- [x] `examples/web-devops-app-tools/angular.yaml` -> `angular`
- [x] `examples/web-devops-app-tools/express.yaml` -> `express`
- [x] `examples/web-devops-app-tools/fastify.yaml` -> `fastify`
- [x] `examples/web-devops-app-tools/tauri.yaml` -> `tauri`
- [x] `examples/web-devops-app-tools/wails.yaml` -> `wails`
- [x] `examples/web-devops-app-tools/electron.yaml` -> `electron`

### database

- [x] `examples/database/postgres.yaml` -> `postgres`
- [x] `examples/database/sqlite.yaml` -> `sqlite`
- [x] `examples/database/mysql.yaml` -> `mysql`
- [x] `examples/database/redis.yaml` -> `redis`
- [x] `examples/database/mongodb.yaml` -> `mongodb`
- [x] `examples/database/duckdb.yaml` -> `duckdb`
- [x] `examples/database/clickhouse.yaml` -> `clickhouse`
- [x] `examples/database/dbeaver.yaml` -> `dbeaver`
- [x] `examples/database/pgadmin.yaml` -> `pgadmin`
- [x] `examples/database/beekeeper-studio.yaml` -> `beekeeper studio`

### communication apps

- [x] `examples/communication-apps/slack.yaml` -> `Slack`
- [x] `examples/communication-apps/discord.yaml` -> `Discord`
- [x] `examples/communication-apps/teams.yaml` -> `Teams`
- [x] `examples/communication-apps/zoom.yaml` -> `Zoom`
- [x] `examples/communication-apps/telegram-desktop.yaml` -> `Telegram Desktop`
- [x] `examples/communication-apps/jira.yaml` -> `Jira`
- [x] `examples/communication-apps/trello.yaml` -> `Trello`
- [x] `examples/communication-apps/github-projects.yaml` -> `Github Projects`

### gaming / streaming

- [x] `examples/gaming-streaming/steam.yaml` -> `Steam`
- [x] `examples/gaming-streaming/discord-gaming.yaml` -> `discord`
- [x] `examples/gaming-streaming/obs-studio-streaming.yaml` -> `obs studio`
- [x] `examples/gaming-streaming/retroarch.yaml` -> `retroarch`
- [x] `examples/gaming-streaming/lutris.yaml` -> `lutris`
- [x] `examples/gaming-streaming/heroic-games-launcher.yaml` -> `heroic games launcher`
- [x] `examples/gaming-streaming/mangohud.yaml` -> `mangohud`
- [x] `examples/gaming-streaming/gamemode.yaml` -> `gamemode`

## Completion Criteria

- Every checklist item above has a corresponding file in `examples/`.
- Every generated file validates against the schema described in [referencefile.md](/home/mike/sellsword/justpeek/docs/referencefile.md).
- Every generated file uses the exact intended top-level `group`.
- The two pre-existing tracked files, `git.yaml` and `vscode.yaml`, are normalized before marking their items complete.
