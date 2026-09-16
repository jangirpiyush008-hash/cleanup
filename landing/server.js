// Minimal static file server for the landing page. Zero dependencies,
// runs on Node 20+. Railway will `node landing/server.js`.

import { createServer } from 'node:http';
import { readFile, readdir, stat } from 'node:fs/promises';
import { extname, join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

// Directory of THIS file. `fileURLToPath` handles spaces / unicode in the
// path correctly (import.meta.url returns URL-encoded characters otherwise).
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)));
const PORT = Number(process.env.PORT) || 3000;

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.css':  'text/css; charset=utf-8',
  '.js':   'application/javascript; charset=utf-8',
  '.svg':  'image/svg+xml',
  '.png':  'image/png',
  '.jpg':  'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.ico':  'image/x-icon',
  '.txt':  'text/plain; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.dmg':  'application/x-apple-diskimage',
  '.exe':  'application/vnd.microsoft.portable-executable',
  '.msi':  'application/x-msi',
  '.zip':  'application/zip',
};

// Extensions that should always download (never render inline).
const DOWNLOAD_EXTS = new Set(['.dmg', '.exe', '.msi', '.zip']);

const server = createServer(async (req, res) => {
  try {
    // Strip query string, normalize, and prevent path escapes.
    const urlPath = decodeURIComponent((req.url || '/').split('?')[0]);

    // Manifest of installer files currently available in landing/downloads/.
    // The landing page uses this to know which platform buttons to enable —
    // Mac / Windows / Android all light up automatically the moment the
    // matching file is dropped into that directory by the Docker build.
    if (urlPath === '/api/downloads') {
      const dir = join(ROOT, 'downloads');
      const entries = await readdir(dir).catch(() => []);
      const list = [];
      for (const name of entries) {
        try {
          const s = await stat(join(dir, name));
          if (s.isFile()) list.push({ name, size: s.size });
        } catch {}
      }
      res.writeHead(200, {
        'Content-Type': 'application/json; charset=utf-8',
        'Cache-Control': 'no-cache',
      });
      res.end(JSON.stringify({ files: list }));
      return;
    }

    const rel = urlPath === '/' ? 'index.html' : urlPath.replace(/^\/+/, '');
    const filePath = resolve(join(ROOT, rel));

    // Sandbox: refuse anything outside ROOT.
    if (!filePath.startsWith(ROOT)) {
      res.writeHead(403); res.end('Forbidden'); return;
    }

    // Fall back to index.html for unknown routes (SPA-friendly).
    let target = filePath;
    try {
      const s = await stat(target);
      if (s.isDirectory()) target = join(target, 'index.html');
    } catch {
      target = join(ROOT, 'index.html');
    }

    const buf = await readFile(target);
    const ext = extname(target).toLowerCase();
    const filename = target.split('/').pop() || 'download';

    const headers = {
      'Content-Type': MIME[ext] || 'application/octet-stream',
      'Cache-Control': ext === '.html' ? 'no-cache' : 'public, max-age=3600',
      'X-Content-Type-Options': 'nosniff',
      'Referrer-Policy': 'strict-origin-when-cross-origin',
    };
    if (DOWNLOAD_EXTS.has(ext)) {
      // Force download regardless of browser previewing behavior.
      headers['Content-Disposition'] = `attachment; filename="${filename}"`;
    }

    res.writeHead(200, headers);
    res.end(buf);
  } catch (err) {
    res.writeHead(500);
    res.end('Internal error');
    console.error(err);
  }
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`Mac Cleanup landing → http://0.0.0.0:${PORT}`);
});
