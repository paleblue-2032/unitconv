'use strict';

const $ = (sel) => document.querySelector(sel);

function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (c) => (
    { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]
  ));
}

function fmtNum(x) {
  if (x === 0) return '0';
  const a = Math.abs(x);
  if (a >= 1e-4 && a < 1e7) return String(Number(x.toFixed(10)));
  return x.toExponential(6).replace(/\.?0+e/, 'e');
}

async function jget(url) {
  const response = await fetch(url);
  return response.json();
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

async function runConvert() {
  const value = $('#value').value.trim();
  const from = $('#from').value.trim();
  const to = $('#to').value.trim();
  const out = $('#result');
  const error = $('#error');
  error.textContent = '';
  if (!value || !from || !to) {
    out.textContent = '—';
    return;
  }
  const data = await jget(
    `/api/convert?value=${encodeURIComponent(value)}&from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`,
  );
  if (data.ok) {
    out.innerHTML =
      `${escapeHtml(fmtNum(data.value))} <span class="u">${escapeHtml(data.unit)}</span>` +
      (data.pretty ? `<span class="pretty">≈ ${escapeHtml(data.pretty)}</span>` : '');
  } else {
    out.textContent = '—';
    error.textContent = data.error || '変換できません';
  }
}

async function runScale() {
  const value = $('#svalue').value.trim();
  const from = $('#sfrom').value.trim();
  const to = $('#sto').value.trim();
  const out = $('#sresult');
  const error = $('#serror');
  error.textContent = '';
  if (!value || !from || !to) {
    out.textContent = '—';
    return;
  }
  const data = await jget(
    `/api/scale?value=${encodeURIComponent(value)}&from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`,
  );
  if (data.ok) {
    out.textContent = fmtNum(data.value);
  } else {
    out.textContent = '—';
    error.textContent = data.error || '';
  }
}

function powerOfTen(x) {
  if (x <= 0) return null;
  const exp = Math.round(Math.log10(x));
  return Math.abs(10 ** exp - x) / x < 1e-9 ? exp : null;
}

function buildPrefixTable(prefixes) {
  const wrap = $('#preftable');
  wrap.innerHTML = '';
  prefixes.forEach((p) => {
    const exp = powerOfTen(p.mult);
    const mult = exp === null ? String(p.mult) : `10^${exp}`;
    const row = document.createElement('div');
    row.className = 'row';
    row.innerHTML = `<span class="name">${escapeHtml(p.name)}</span><span class="sym">${escapeHtml(p.sym)}</span><span>${mult}</span>`;
    wrap.appendChild(row);
  });
}

function buildUnitTable(units) {
  const wrap = $('#unittable');
  wrap.innerHTML = '';
  const groups = new Map();
  units.forEach((u) => {
    if (!groups.has(u.group)) groups.set(u.group, []);
    groups.get(u.group).push(u);
  });
  groups.forEach((list, group) => {
    const block = document.createElement('div');
    const heading = document.createElement('h3');
    heading.textContent = group;
    block.appendChild(heading);
    const chips = document.createElement('div');
    chips.className = 'chips';
    list.forEach((u) => {
      const chip = document.createElement('span');
      chip.className = 'u';
      chip.textContent = u.prefixable ? `${u.name}*（接頭辞OK）` : u.name;
      if (u.aliases.length) chip.title = u.aliases.join(', ');
      chips.appendChild(chip);
    });
    block.appendChild(chips);
    wrap.appendChild(block);
  });
}

async function init() {
  const [units, prefixes] = await Promise.all([jget('/api/units'), jget('/api/prefixes')]);

  const unitNames = new Set();
  units.forEach((u) => {
    unitNames.add(u.name);
    u.aliases.forEach((a) => unitNames.add(a));
  });
  ['km', 'cm', 'mm', 'µm', 'nm', 'ms', 'µs', 'ns', 'GHz', 'MHz', 'kHz',
    'kB', 'MB', 'GB', 'TB', 'KiB', 'MiB', 'GiB', 'kJ', 'kcal', 'keV', 'MeV',
    'mV', 'kPa', 'degC', 'degF', 'K'].forEach((n) => unitNames.add(n));
  const unitList = $('#units');
  [...unitNames].sort().forEach((n) => unitList.appendChild(new Option(n)));

  const prefixNames = new Set();
  prefixes.forEach((p) => {
    prefixNames.add(p.name);
    p.sym.split('/').forEach((s) => prefixNames.add(s));
  });
  const prefixList = $('#prefixes');
  [...prefixNames].sort().forEach((n) => prefixList.appendChild(new Option(n)));

  buildPrefixTable(prefixes);
  buildUnitTable(units);

  const onChange = debounce(runConvert, 120);
  ['#value', '#from', '#to'].forEach((sel) => $(sel).addEventListener('input', onChange));
  const onScale = debounce(runScale, 120);
  ['#svalue', '#sfrom', '#sto'].forEach((sel) => $(sel).addEventListener('input', onScale));

  document.querySelectorAll('.chip').forEach((chip) =>
    chip.addEventListener('click', () => {
      const [value, from, to] = chip.dataset.ex.split('|');
      $('#value').value = value;
      $('#from').value = from;
      $('#to').value = to;
      runConvert();
    }));

  runConvert();
  runScale();
}

init();
