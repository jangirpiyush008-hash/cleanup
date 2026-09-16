# Landing-page image for Railway. Bundles installers into the image at
# build time so downloads never leave our own domain.
#
# When a Dockerfile is present at the repo root, Railway uses it directly
# and skips Railpack auto-detection entirely.

FROM node:20-alpine

WORKDIR /app

# curl is used only during build to fetch installers from GitHub Releases.
RUN apk add --no-cache curl

COPY landing ./landing

# ─── Installer manifest ─────────────────────────────────────────
# Bump these when a new release ships. Missing files are ignored — the
# landing page's /api/downloads endpoint only surfaces what actually
# landed in landing/downloads/, so the corresponding button on the site
# lights up automatically the moment the file is present.

ARG MAC_URL=https://github.com/jangirpiyush008-hash/cleanup/releases/download/v0.1.0/MacCleanup-0.1.0-universal.dmg
ARG WIN_URL=https://github.com/jangirpiyush008-hash/cleanup/releases/download/v0.1.0/MacCleanup-v0.1.0-x64-setup.exe
ARG APK_URL=https://github.com/jangirpiyush008-hash/cleanup/releases/download/v0.1.0/MacCleanup-v0.1.0-universal.apk

RUN mkdir -p landing/downloads && \
    ( curl -fsSL "$MAC_URL" -o "landing/downloads/$(basename "$MAC_URL")" && echo "✓ Mac"        ) || echo "· Mac asset not yet published" && \
    ( curl -fsSL "$WIN_URL" -o "landing/downloads/$(basename "$WIN_URL")" && echo "✓ Windows"    ) || echo "· Windows asset not yet published" && \
    ( curl -fsSL "$APK_URL" -o "landing/downloads/$(basename "$APK_URL")" && echo "✓ Android"    ) || echo "· Android asset not yet published" && \
    ls -lh landing/downloads/

ENV NODE_ENV=production
EXPOSE 3000

CMD ["node", "landing/server.js"]
