# CI templates

The GitHub Actions build workflow lives here rather than in
`.github/workflows/` because our current push token doesn't carry the
`workflow` scope. GitHub refuses to accept modifications to files under
`.github/workflows/` without that scope.

## Activate the workflow (60 seconds, one of two ways)

### Option A · Via the GitHub website (no new token needed)

1. Open the repo on the web:
   <https://github.com/jangirpiyush008-hash/cleanup>
2. Click **Add file → Create new file**.
3. In the filename field, type: `.github/workflows/build.yml`
4. Copy the contents of `ci-templates/build.yml` (this directory) and paste them in.
5. Scroll down, click **Commit changes** → **Commit directly to `main`**.

Done. The workflow will run on the next push to `main` or any pushed tag.

### Option B · Regenerate a PAT with `workflow` scope

1. <https://github.com/settings/tokens> → **Generate new token (classic)**.
2. Tick both `repo` and `workflow`.
3. Locally:
   ```bash
   cd MacCleanup
   git mv ci-templates/build.yml .github/workflows/build.yml
   git commit -m "Add cross-platform build workflow"
   git push https://jangirpiyush008-hash:NEW_TOKEN@github.com/jangirpiyush008-hash/cleanup.git main
   ```

Then rotate the token when you're done.

## What the workflow does

- Triggers on push to `main`, on any `v*` tag, and on manual dispatch.
- Runs a matrix: `macos-latest` (builds universal `.dmg`) and `windows-latest`
  (builds `.msi` + `.exe` NSIS installer).
- Uses `tauri-apps/tauri-action` to build the app and — on tags — attach
  the installers to a draft release.
- Free tier of Actions covers this comfortably; expect ~15 min per platform.
