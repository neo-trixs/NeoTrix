# Upgrading NeoTrix

This page documents steps required when upgrading between NeoTrix versions and
records known breaking changes. Add a new section below whenever a release
introduces a behavior change, config rename, or removed feature (release-checklist 5.4).

## Upgrade procedure

1. **Read the changelog**: `CHANGELOG.md` at the repo root (or
   `git cliff -u` output) lists feature/fix/chore groups per release.
2. **Check the breaking changes below** for your target version.
3. **Back up your state** before upgrading:
   ```bash
   cp -r ~/.neotrix ~/.neotrix.bak
   ```
   State locations: agent definitions in `~/.neotrix/agents`, config in
   `~/.neotrix/config.toml`, KB at `~/.neotrix/knowledge.db` (also
   `neotrix_knowledge.db` in older installs).
4. **Reinstall the binary**:
   ```bash
   curl -fsSL https://neotrix.ai/install | bash
   ```
   Or update the Desktop app via its built-in updater / cask.
5. **Verify**:
   ```bash
   neotrix --version
   neotrix doctor
   ```

## Known breaking changes

### Unreleased

- **Agent file definitions moved**: file-based agent definitions are now sourced
  from `~/.neotrix/agents` (single source of truth), not the repository's
  `.opencode/agents`. Copy any custom agent `.md` files you maintain into
  `~/.neotrix/agents/` and remove the repo-local copies.
- **`frontend-v1` / `session-log` / `anchor` shells archived**: moved to
  `_archive/`. Import paths referencing them no longer resolve.
- **Removed orphan modules**: `nt_world_video` and `nt_world_resource_discovery`
  were deleted (zero consumers, Dark Forest rule). Use `nt_world_crawl` /
  the unified crawl pipeline instead.

### v0.18.x

- **macOS sysctl FFI extracted**: platform FFI now lives in the
  `neotrix-sysctl` crate. Internal only — no user-facing config change.

## Template for future entries

```markdown
### vX.Y.Z

- **Short description of the change** — what to do if you were relying on the
  old behavior (config rename, file path move, removed command, default change).
```
