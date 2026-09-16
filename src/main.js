// Entry point. Loads app state, mounts the phase-driven router.
// Uses Tauri's IPC — accessible via `window.__TAURI__.core.invoke`.

const { invoke } = window.__TAURI__.core;

// ─── State ────────────────────────────────────────────────────────
const state = {
  phase: 'onboarding', // onboarding | idle | scanning | results | review | confirming | cleaning | done
  scanProgress: 'Preparing…',
  report: null,        // ScanReport
  selection: new Set(),
  plan: null,          // { items: ScanItem[] }
  cleanupSummary: null,
  volume: { total: 0, free: 0, used: 0 },
  meta: null,
};

const KEY_ONBOARDED = 'mc.onboarding.seen';
try {
  if (localStorage.getItem(KEY_ONBOARDED) === '1') state.phase = 'idle';
} catch {}

// ─── Utilities ────────────────────────────────────────────────────
function fmtBytes(n) {
  if (n == null || n < 0) return '—';
  const abs = Math.abs(n);
  if (abs < 1024) return `${n} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let v = n / 1024, i = 0;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2)} ${units[i]}`;
}
function html(strings, ...values) {
  let out = '';
  strings.forEach((s, i) => {
    out += s;
    if (i < values.length) {
      const v = values[i];
      if (Array.isArray(v)) out += v.join('');
      else if (v == null) out += '';
      else out += String(v);
    }
  });
  return out;
}
function escapeHTML(str) {
  return String(str ?? '').replace(/[&<>"']/g, m => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
  }[m]));
}

// ─── Router ───────────────────────────────────────────────────────
function render() {
  const root = document.getElementById('app');
  root.innerHTML = `<div class="titlebar"></div>` + viewForPhase();
  wireHandlers();
}

function viewForPhase() {
  switch (state.phase) {
    case 'onboarding':  return viewOnboarding();
    case 'idle':        return viewHome();
    case 'scanning':    return viewScanning();
    case 'results':     return viewResults();
    case 'review':      return viewReview();
    case 'confirming':  return viewConfirm();
    case 'cleaning':    return viewCleaning();
    case 'done':        return viewDone();
    default:            return `<p>Unknown phase.</p>`;
  }
}

// ─── Screens ──────────────────────────────────────────────────────
function viewOnboarding() {
  return html`
    <section class="screen onboarding col gap-3">
      <div class="mark">✦</div>
      <h1 class="serif">Your privacy matters.</h1>
      <p>Mac Cleanup runs entirely on this computer.
         Your files and scan results never leave this Mac.</p>
      <button class="btn btn-primary" data-action="onboard">Continue</button>
      <div class="tiny">Nothing is deleted during scanning. You approve every cleanup.</div>
    </section>`;
}

function viewHome() {
  const v = state.volume;
  const pct = v.total > 0 ? Math.round((v.used / v.total) * 100) : 0;
  return html`
    <section class="screen home col gap-3">
      <h1 class="serif">Mac Cleanup</h1>
      <div class="sub">Find unnecessary files and safely free up space.</div>
      <div class="ring" style="--pct:${pct}">
        <div class="ring-inner">
          <div class="ring-used">${escapeHTML(fmtBytes(v.used))}</div>
          <div class="ring-label">of ${escapeHTML(fmtBytes(v.total))} used</div>
          <div class="ring-free">${escapeHTML(fmtBytes(v.free))} free</div>
        </div>
      </div>
      <button class="btn btn-primary" data-action="scan">Scan my Mac</button>
      <div class="disclaimer">Nothing will be deleted during the scan.</div>
    </section>`;
}

function viewScanning() {
  return html`
    <section class="screen scanning col gap-3">
      <div class="spinner"></div>
      <h2 class="serif" style="font-size:22px;">Scanning your Mac…</h2>
      <div class="mono muted">${escapeHTML(state.scanProgress)}</div>
      <div class="disclaimer" style="margin-top:12px">Read-only. Nothing is being deleted.</div>
    </section>`;
}

function viewResults() {
  const r = state.report;
  const safeItems = r.items.filter(i => i.risk === 'safe-to-clean');
  const reviewItems = r.items.filter(i => i.risk === 'review');
  const selectedTotal = r.items
    .filter(i => state.selection.has(i.id))
    .reduce((a, i) => a + i.size, 0);

  const categoryOrder = [
    'browser-cache', 'application-cache', 'developer-cache',
    'package-manager', 'logs', 'temp', 'large-file',
    'duplicate', 'xcode', 'homebrew', 'docker',
    'snapshots', 'system-managed', 'other',
  ];
  const categoryLabels = {
    'browser-cache': 'Browser caches',
    'application-cache': 'Application caches',
    'developer-cache': 'Developer caches',
    'package-manager': 'Package managers',
    'logs': 'Logs',
    'temp': 'Temporary files',
    'large-file': 'Large files',
    'duplicate': 'Duplicates',
    'xcode': 'Xcode',
    'homebrew': 'Homebrew',
    'docker': 'Docker',
    'snapshots': 'System snapshots',
    'system-managed': 'System managed',
    'other': 'Other',
  };
  const byCat = {};
  r.items.forEach(i => { (byCat[i.category] ||= []).push(i); });

  const catSections = categoryOrder
    .filter(c => byCat[c] && byCat[c].length)
    .map(c => html`
      <div class="cat-title">${escapeHTML(categoryLabels[c])}</div>
      ${byCat[c].map(itemRow).join('')}
    `).join('');

  const errorsBlock = (r.errors && r.errors.length) ? html`
    <details style="margin-top:18px;font-size:12px;color:var(--ink-muted)">
      <summary>${r.errors.length} locations were skipped (details)</summary>
      <div style="margin-top:8px">${r.errors.map(e => html`
        <div class="mono" style="font-size:11px">• ${escapeHTML(e.path)} — ${escapeHTML(e.reason)}</div>
      `).join('')}</div>
    </details>` : '';

  const empty = r.items.length === 0
    ? `<p class="muted" style="margin-top:24px">Nothing to clean right now — your Mac is tidy.</p>`
    : '';

  return html`
    <div class="results-head">
      <button class="btn btn-ghost" data-action="home">← Home</button>
      <div class="grow" style="text-align:center">
        <span class="title">Scan complete</span>
      </div>
      <span style="width:70px"></span>
    </div>
    <div class="results-body">
      <div class="summary-row">
        <div class="summary-tile safe">
          <div class="label">Safe to clean</div>
          <div class="value">${escapeHTML(fmtBytes(sumBy(r.items, 'safe-to-clean')))}</div>
        </div>
        <div class="summary-tile review">
          <div class="label">Review</div>
          <div class="value">${escapeHTML(fmtBytes(sumBy(r.items, 'review')))}</div>
        </div>
        <div class="summary-tile protect">
          <div class="label">Protected</div>
          <div class="value">${escapeHTML(fmtBytes(sumBy(r.items, 'protected')))}</div>
        </div>
      </div>
      <div class="select-actions">
        <button data-action="select-safe">Select all safe</button>
        <button data-action="clear-selection">Clear selection</button>
        ${state.selection.size ? html`<div class="sel-total">Selected: ${escapeHTML(fmtBytes(selectedTotal))}</div>` : ''}
      </div>
      ${catSections}
      ${empty}
      ${errorsBlock}
    </div>
    <div class="footer-bar">
      <div class="disclaimer grow">Reviewing then approving is the only way anything is removed.</div>
      <button class="btn btn-primary" data-action="review" ${state.selection.size === 0 ? 'disabled' : ''}>
        Review ${state.selection.size} item${state.selection.size === 1 ? '' : 's'}
      </button>
    </div>`;
}

function itemRow(item) {
  const selectable = item.risk !== 'protected';
  const isSelected = state.selection.has(item.id);
  const badge = item.risk === 'safe-to-clean' ? 'safe' : item.risk === 'review' ? 'review' : 'protect';
  const badgeLabel = item.risk === 'safe-to-clean' ? 'Safe to clean' : item.risk === 'review' ? 'Review' : 'Protected';
  return html`
    <div class="item ${isSelected ? 'selected' : ''}">
      <input type="checkbox" data-toggle="${escapeHTML(item.id)}"
             ${isSelected ? 'checked' : ''} ${selectable ? '' : 'disabled'} />
      <div class="body">
        <div class="head">
          <div class="name">${escapeHTML(item.display_name)}</div>
          <span class="badge ${badge}">${escapeHTML(badgeLabel)}</span>
          ${item.requires_app_closed ? html`
            <span class="badge warn" title="Close ${escapeHTML(item.requires_app_closed)} for best results">
              ${escapeHTML(item.requires_app_closed)} should be closed
            </span>` : ''}
          <div class="size">${escapeHTML(fmtBytes(item.size))}</div>
        </div>
        <div class="desc">${escapeHTML(item.explanation)}</div>
        <details>
          <summary>Details</summary>
          <div class="details-body">
            <div><span class="quiet">Location:</span> <span class="path">${escapeHTML(item.path)}</span></div>
            <div style="margin-top:4px"><span class="quiet">Recovery:</span> ${escapeHTML(item.recovery)}</div>
          </div>
        </details>
      </div>
    </div>`;
}

function sumBy(items, risk) {
  return items.filter(i => i.risk === risk).reduce((a, i) => a + i.size, 0);
}

function viewReview() {
  const plan = state.plan;
  return html`
    <div class="results-head">
      <button class="btn btn-ghost" data-action="back-to-results">← Go back</button>
      <div class="grow" style="text-align:center"><span class="title">Review Cleanup</span></div>
      <button class="btn btn-ghost" data-action="home">Cancel</button>
    </div>
    <div class="results-body">
      <div class="cat-title">You are about to remove</div>
      <div class="review-list">
        ${plan.items.map(i => html`
          <div class="item">
            <div class="body">
              <div class="head">
                <div class="name">${escapeHTML(i.display_name)}</div>
                <div class="size">${escapeHTML(fmtBytes(i.size))}</div>
              </div>
              <div class="path mono muted" style="font-size:11px;margin-top:4px">${escapeHTML(i.path)}</div>
            </div>
          </div>`).join('')}
      </div>
      <div class="review-total">
        <div class="label">Total</div>
        <div class="value">${escapeHTML(fmtBytes(plan.items.reduce((a, i) => a + i.size, 0)))}</div>
      </div>
      <p class="muted" style="font-size:12.5px;margin-top:8px">
        Only the items listed above will be moved to Trash. Nothing else will be touched.
      </p>
    </div>
    <div class="footer-bar">
      <button class="btn btn-secondary" data-action="home">Cancel</button>
      <button class="btn btn-secondary" data-action="back-to-results">Go Back</button>
      <div class="grow"></div>
      <button class="btn btn-danger" data-action="request-confirm">Move Selected Items to Trash</button>
    </div>`;
}

function viewConfirm() {
  const plan = state.plan;
  return html`
    ${viewReview()}
    <div class="overlay">
      <div class="modal">
        <div class="icon">🗑</div>
        <h2>Move ${plan.items.length} selected item${plan.items.length === 1 ? '' : 's'} to Trash?</h2>
        <div class="total">Total: ${escapeHTML(fmtBytes(plan.items.reduce((a, i) => a + i.size, 0)))}</div>
        <div class="subtle">Items go to Trash. You can restore them until you empty Trash.</div>
        <div class="actions">
          <button class="btn btn-secondary" data-action="cancel-confirm">Cancel</button>
          <button class="btn btn-danger"    data-action="do-cleanup">Move to Trash</button>
        </div>
      </div>
    </div>`;
}

function viewCleaning() {
  return html`
    <section class="screen scanning col gap-3">
      <div class="spinner"></div>
      <h2 class="serif" style="font-size:20px;">Moving items to Trash…</h2>
    </section>`;
}

function viewDone() {
  const s = state.cleanupSummary;
  const moved   = s.results.filter(r => r.outcome.outcome === 'moved-to-trash');
  const skipped = s.results.filter(r => r.outcome.outcome === 'skipped');
  const failed  = s.results.filter(r => r.outcome.outcome === 'failed');

  return html`
    <section class="screen done col gap-3">
      <h1 class="serif" style="font-size:26px">Cleanup complete</h1>
      <div class="mono quiet" style="font-size:11px;letter-spacing:0.14em;text-transform:uppercase;font-weight:800;color:var(--ink-muted)">
        You recovered
      </div>
      <div class="big">${escapeHTML(fmtBytes(s.total_reclaimed))}</div>
      <div class="caption">Mac now has ${escapeHTML(fmtBytes(s.volume_free_after))} free</div>

      <div class="result-list">
        ${moved.length ? html`
          <div class="result-section">Moved to Trash</div>
          ${moved.map(r => resultRow('moved', r.item.display_name, fmtBytes(r.bytes_reclaimed)))}
        ` : ''}
        ${skipped.length ? html`
          <div class="result-section">Skipped for safety</div>
          ${skipped.map(r => resultRow('skipped', r.item.display_name, r.outcome.detail?.reason || ''))}
        ` : ''}
        ${failed.length ? html`
          <div class="result-section">Failed</div>
          ${failed.map(r => resultRow('failed', r.item.display_name, r.outcome.detail?.reason || ''))}
        ` : ''}
      </div>

      <div class="row gap-3" style="margin-top:22px;justify-content:center">
        <button class="btn btn-secondary" data-action="open-trash">Open Trash</button>
        <button class="btn btn-primary"   data-action="home">Done</button>
      </div>
    </section>`;
}

function resultRow(kind, name, meta) {
  return html`
    <div class="result-row ${kind}">
      <div class="icon-dot"></div>
      <div class="name">${escapeHTML(name)}</div>
      <div class="meta">${escapeHTML(meta)}</div>
    </div>`;
}

// ─── Handlers ─────────────────────────────────────────────────────
function wireHandlers() {
  document.querySelectorAll('[data-action]').forEach(el => {
    el.addEventListener('click', () => onAction(el.dataset.action));
  });
  document.querySelectorAll('[data-toggle]').forEach(el => {
    el.addEventListener('change', () => {
      const id = el.dataset.toggle;
      if (state.selection.has(id)) state.selection.delete(id);
      else state.selection.add(id);
      render();
    });
  });
}

async function onAction(action) {
  switch (action) {
    case 'onboard':
      try { localStorage.setItem(KEY_ONBOARDED, '1'); } catch {}
      state.phase = 'idle';
      await refreshVolume();
      return render();

    case 'scan':
      state.phase = 'scanning';
      state.scanProgress = 'Preparing…';
      render();
      try {
        const report = await invoke('scan');
        state.report = report;
        state.selection = new Set();
        state.volume = { total: report.volume_total, free: report.volume_free, used: report.volume_used };
        state.phase = 'results';
      } catch (e) {
        alert('Scan failed: ' + e);
        state.phase = 'idle';
      }
      return render();

    case 'home':
      state.phase = 'idle';
      state.selection = new Set();
      await refreshVolume();
      return render();

    case 'select-safe':
      state.report.items
        .filter(i => i.risk === 'safe-to-clean')
        .forEach(i => state.selection.add(i.id));
      return render();

    case 'clear-selection':
      state.selection = new Set();
      return render();

    case 'review': {
      const items = state.report.items.filter(i => state.selection.has(i.id));
      if (!items.length) return;
      state.plan = { items };
      state.phase = 'review';
      return render();
    }

    case 'back-to-results':
      state.phase = 'results';
      return render();

    case 'request-confirm':
      state.phase = 'confirming';
      return render();

    case 'cancel-confirm':
      state.phase = 'review';
      return render();

    case 'do-cleanup':
      state.phase = 'cleaning';
      render();
      try {
        const summary = await invoke('cleanup', {
          req: { item_ids: state.plan.items.map(i => i.id) },
        });
        state.cleanupSummary = summary;
        state.volume = {
          total: state.volume.total,
          free: summary.volume_free_after,
          used: summary.volume_used_after,
        };
        state.phase = 'done';
      } catch (e) {
        alert('Cleanup failed: ' + e);
        state.phase = 'review';
      }
      return render();

    case 'open-trash':
      try { await invoke('open_trash'); } catch {}
      return;
  }
}

async function refreshVolume() {
  try { state.volume = await invoke('volume_stats'); } catch {}
}

// ─── Boot ─────────────────────────────────────────────────────────
(async function boot() {
  try { state.meta = await invoke('app_meta'); } catch {}
  await refreshVolume();
  render();
})();
