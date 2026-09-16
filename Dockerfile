# Landing-page image for Railway. Bundles the .dmg into the image at
# build time so the download link never leaves our own domain.
#
# When a Dockerfile is present at the repo root, Railway uses it directly
# and skips Railpack auto-detection entirely.

FROM node:20-alpine

WORKDIR /app

# curl is used only during build to fetch the Mac installer from
# GitHub Releases. Not present in the runtime image after this line.
RUN apk add --no-cache curl

COPY landing ./landing

# Pull the current .dmg from the GitHub release. The .dmg lives on GitHub
# Releases as the canonical source; we mirror it into our container so
# visitors download from our own domain. Bump the release URL whenever
# a new version ships.
ARG DMG_URL=https://github.com/jangirpiyush008-hash/cleanup/releases/download/v0.1.0/MacCleanup-0.1.0-universal.dmg
ARG DMG_NAME=MacCleanup-0.1.0-universal.dmg
RUN mkdir -p landing/downloads && \
    curl -fsSL "$DMG_URL" -o "landing/downloads/$DMG_NAME" && \
    ls -lh "landing/downloads/$DMG_NAME"

ENV NODE_ENV=production
EXPOSE 3000

CMD ["node", "landing/server.js"]
