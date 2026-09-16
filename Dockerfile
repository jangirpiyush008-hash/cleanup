# Landing-page image for Railway. Zero dependencies — the landing page
# is served by a ~60-line static server using only Node built-ins.
#
# When a Dockerfile is present at the repo root, Railway uses it directly
# and skips Railpack auto-detection entirely.

FROM node:20-alpine

WORKDIR /app

# Copy only the landing directory. The Rust Tauri source has nothing to
# do with the web deploy.
COPY landing ./landing

# Railway sets PORT dynamically; the server reads it from env.
ENV NODE_ENV=production
EXPOSE 3000

CMD ["node", "landing/server.js"]
