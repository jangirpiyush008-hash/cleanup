# CI templates

Contains the GitHub Actions workflow that builds **macOS + Windows + Android**
in parallel and attaches the artifacts to a GitHub Release on any `v*` tag push.

**Why it's here, not in `.github/workflows/`**: our current push token
doesn't carry the `workflow` scope. GitHub refuses direct pushes that
touch `.github/workflows/*` without that scope. The workflow content is
identical either way — GitHub simply won't accept it via git without a
scoped token.

## Activate in 60 seconds via the GitHub website (no new token needed)

1. Open the repo on the web: <https://github.com/jangirpiyush008-hash/cleanup>
2. Click **Add file → Create new file**.
3. In the filename field, type exactly: `.github/workflows/build.yml`
4. Copy the contents of `ci-templates/build.yml` (this directory) and paste in.
5. Scroll down → **Commit changes** → *Commit directly to `main`*.

That's it. Once committed, the workflow runs on every push to `main`
and on any `v*` tag. To trigger a full release build:

```bash
git tag v0.1.1
git push origin v0.1.1
```

GitHub Actions then produces:
- **macOS**: `Mac Cleanup_0.1.1_universal.dmg` (~4 MB)
- **Windows**: `Mac Cleanup_0.1.1_x64-setup.exe` (~4 MB, NSIS installer)
- **Android**: `MacCleanup-v0.1.1-universal.apk` (~15 MB, debug-signed)

All three are attached to a draft release. Publish the release → the
landing page's download buttons for each OS start working automatically
(Docker rebuilds pull them into the container).

## What Android currently is

The APK is built from the same Tauri codebase as the Mac / Windows app.
On a phone it opens and shows the UI, but the current scanners target
Mac / Windows paths (`~/Library/Caches`, `AppData\Local`, etc.), so it
finds nothing on a real Android device.

The **infrastructure** (build + sign + release + host) is fully wired.
The **Android-specific scanners** (Downloads, WhatsApp Media, DCIM,
duplicate photos, unused apps) are the next 1-week body of work — worth
doing only if Android is a real product commitment.

For Play Store distribution the debug-signed APK is not sufficient —
you'll need to configure a release keystore + Play Store upload key.
See <https://tauri.app/develop/sign/android/> for the walkthrough.

## Alternative: regenerate a PAT with `workflow` scope

If you'd rather push the file from your terminal instead of the web UI:

1. <https://github.com/settings/tokens> → **Generate new token (classic)**
2. Tick both `repo` and `workflow`
3. Locally:
   ```bash
   cd MacCleanup
   git mv ci-templates/build.yml .github/workflows/build.yml
   git commit -m "Enable cross-platform build workflow"
   git push https://jangirpiyush008-hash:NEW_TOKEN@github.com/jangirpiyush008-hash/cleanup.git main
   ```
4. Rotate the token when you're done.
