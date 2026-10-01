import init, {
  Mandelbrot, Life, Field,
  bench_matmul, bench_sieve, bench_escape, bench_mix,
  bench_radix, bench_crc, bench_queens, bench_trees,
  heap_bytes,
} from './pkg/wasm_demo.js';

/* ── helpers ─────────────────────────────────────────────────────────── */

const $ = (id) => document.getElementById(id);
const fmt = (n) => Math.round(n).toLocaleString('en-US');

function bytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1048576) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1048576).toFixed(2)} MB`;
}

/** Views must be rebuilt each frame: growing linear memory detaches the old buffer. */
function imageFrom(memory, ptr, len, w, h) {
  return new ImageData(new Uint8ClampedArray(memory.buffer, ptr, len), w, h);
}

let wasm;
try {
  wasm = await init();
} catch (err) {
  $('stage').innerHTML = `
    <div class="fault">
      <h2>تعذّر تحميل الوحدة</h2>
      <p>لا تُحمَّل وحدات WebAssembly من <code>file://</code>. شغّل خادماً محلياً من مجلد المشروع:</p>
      <p><code>python3 -m http.server 8000</code></p>
      <p>ثم افتح <code>http://localhost:8000</code>.</p>
    </div>`;
  $('telemetry').remove();
  document.querySelectorAll('.controls').forEach((el) => el.remove());
  throw err;
}

/* ── telemetry ───────────────────────────────────────────────────────── */

const telemetry = $('telemetry');
const spark = document.createElement('canvas');
spark.width = 132;
spark.height = 30;
const sparkCtx = spark.getContext('2d');
const timings = new Float32Array(66);
let timingAt = 0;

function buildReadouts(fields) {
  telemetry.innerHTML = '';
  const cells = {};
  for (const f of fields) {
    const box = document.createElement('div');
    box.className = 'readout';
    box.innerHTML = `<div class="label"></div><div class="value${f.text ? ' text' : ''}">—</div>`;
    box.firstChild.textContent = f.label;
    telemetry.appendChild(box);
    cells[f.key] = box.lastChild;
  }
  const holder = document.createElement('div');
  holder.className = 'spark';
  holder.appendChild(spark);
  telemetry.appendChild(holder);
  return cells;
}

function setReadout(el, value, unit) {
  el.innerHTML = unit ? `${value}<span class="unit">${unit}</span>` : value;
}

function pushTiming(ms) {
  timings[timingAt] = ms;
  timingAt = (timingAt + 1) % timings.length;

  const w = spark.width, h = spark.height;
  sparkCtx.clearRect(0, 0, w, h);
  let peak = 8;
  for (const t of timings) if (t > peak) peak = t;

  const bw = w / timings.length;
  for (let i = 0; i < timings.length; i++) {
    const t = timings[(timingAt + i) % timings.length];
    const bh = Math.max(1, (t / peak) * (h - 2));
    // Amber past the 16.7 ms budget: the frame missed its slot.
    sparkCtx.fillStyle = t > 16.7 ? '#f0b429' : '#7c5cff';
    sparkCtx.fillRect(i * bw, h - bh, Math.max(1, bw - 0.6), bh);
  }
}

/* ── linear-memory map ───────────────────────────────────────────────── */

const strip = $('strip');
const legend = $('legend');
const REGION_TINT = { fractal: '#7c5cff', life: '#5b6bd6', field: '#9a63ff' };
let stripReady = false;

function paintMemory(active) {
  const heap = heap_bytes();
  const regions = [
    { key: 'fractal', label: 'بكسلات ماندلبرو', ptr: fractal.mb.pixels_ptr(), len: fractal.mb.pixels_len() },
    { key: 'life', label: 'بكسلات لعبة الحياة', ptr: life.board.pixels_ptr(), len: life.board.pixels_len() },
    { key: 'field', label: 'بكسلات حقل الجسيمات', ptr: field.sim.pixels_ptr(), len: field.sim.pixels_len() },
  ].sort((a, b) => a.ptr - b.ptr);

  let cursor = 0;
  let html = '';
  let named = 0;

  for (const r of regions) {
    if (r.ptr > cursor) {
      html += `<div class="seg seg-free" style="width:${((r.ptr - cursor) / heap) * 100}%"></div>`;
    }
    const live = r.key === active ? '1' : '0';
    html += `<div class="seg" data-live="${live}" title="${r.label} @ 0x${r.ptr.toString(16)}"
                  style="width:${(r.len / heap) * 100}%;background:${REGION_TINT[r.key]}"></div>`;
    cursor = r.ptr + r.len;
    named += r.len;
  }
  html += `<div class="seg seg-free" style="flex:1 1 auto"></div>`;

  // Widths animate from zero once, on the first paint only.
  if (!stripReady) {
    strip.innerHTML = html.replace(/width:[\d.]+%/g, 'width:0%');
    requestAnimationFrame(() => { strip.innerHTML = html; });
    stripReady = true;
  } else {
    strip.innerHTML = html;
  }

  legend.innerHTML = regions.map((r) =>
    `<span class="legend-item"><i class="chip" style="background:${REGION_TINT[r.key]}"></i>
       ${r.label} <span class="bytes">${bytes(r.len)}</span></span>`
  ).join('') +
    `<span class="legend-item"><i class="chip seg-free" style="border:1px solid var(--rule)"></i>
       بقية الكومة <span class="bytes">${bytes(heap - named)}</span></span>`;

  setReadoutText('mem-used', bytes(named));
  setReadoutText('mem-total', bytes(heap));
}

function setReadoutText(id, text) { $(id).textContent = text; }

/* ── module: Mandelbrot ──────────────────────────────────────────────── */

const fractal = (() => {
  const canvas = $('fractal-canvas');
  const ctx = canvas.getContext('2d');
  const HOME = { re: -0.65, im: 0.0, scale: 3.4 };
  const DEEP = { re: -0.743643887037151, im: 0.13182590420533, scale: 0.00008 };

  let view = { ...HOME };
  let mb = new Mandelbrot(2, 2);
  let w = 0, h = 0;
  let iter = 512;
  let out = null;
  let lastMs = 0;
  let dragging = false;
  let drag = null;

  function fit() {
    const box = $('stage').getBoundingClientRect();
    const nw = Math.max(320, Math.min(1000, Math.round(box.width)));
    const nh = Math.max(240, Math.round(box.height));
    if (nw === w && nh === h) return;
    w = nw; h = nh;
    canvas.width = w;
    canvas.height = h;
    mb.resize(w, h);
    draw();
  }

  function draw(quality = 1) {
    const budget = quality === 1 ? iter : Math.min(iter, 110);
    const t0 = performance.now();
    mb.render(view.re, view.im, view.scale, budget);
    lastMs = performance.now() - t0;
    ctx.putImageData(imageFrom(wasm.memory, mb.pixels_ptr(), mb.pixels_len(), w, h), 0, 0);
    report();
  }

  function report() {
    if (!out) return;
    setReadout(out.render, lastMs.toFixed(1), 'ms');
    setReadout(out.work, fmt(mb.last_work()));
    setReadout(out.zoom, `${(HOME.scale / view.scale).toExponential(1)}`, '×');
    setReadout(out.re, view.re.toFixed(9));
    setReadout(out.im, view.im.toFixed(9));
    setReadout(out.iter, fmt(iter));
  }

  canvas.addEventListener('pointerdown', (e) => {
    dragging = true;
    canvas.classList.add('dragging');
    canvas.setPointerCapture(e.pointerId);
    drag = { x: e.clientX, y: e.clientY, re: view.re, im: view.im };
  });

  canvas.addEventListener('pointermove', (e) => {
    if (!dragging) return;
    const perPx = view.scale / w;
    view.re = drag.re - (e.clientX - drag.x) * perPx;
    view.im = drag.im - (e.clientY - drag.y) * perPx;
    draw(0);
  });

  function endDrag(e) {
    if (!dragging) return;
    dragging = false;
    canvas.classList.remove('dragging');
    canvas.releasePointerCapture?.(e.pointerId);
    draw(1);
  }
  canvas.addEventListener('pointerup', endDrag);
  canvas.addEventListener('pointercancel', endDrag);

  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    const box = canvas.getBoundingClientRect();
    // Keep the point under the cursor fixed while the scale changes.
    const fx = (e.clientX - box.left) / box.width - 0.5;
    const fy = (e.clientY - box.top) / box.height - 0.5;
    const aspect = h / w;
    const before = { re: view.re + fx * view.scale, im: view.im + fy * view.scale * aspect };
    const factor = Math.exp(Math.sign(e.deltaY) * 0.22);
    view.scale = Math.min(6, Math.max(2e-13, view.scale * factor));
    view.re = before.re - fx * view.scale;
    view.im = before.im - fy * view.scale * aspect;
    draw(0);
    clearTimeout(canvas._settle);
    canvas._settle = setTimeout(() => draw(1), 130);
  }, { passive: false });

  $('f-iter').addEventListener('input', (e) => {
    iter = +e.target.value;
    $('f-iter-out').textContent = fmt(iter);
    draw();
  });
  $('f-reset').addEventListener('click', () => { view = { ...HOME }; draw(); });
  $('f-deep').addEventListener('click', () => {
    view = { ...DEEP };
    iter = 1600;
    $('f-iter').value = 1600;
    $('f-iter-out').textContent = '1,600';
    draw();
  });

  return {
    get mb() { return mb; },
    hint: 'اسحب للتحريك، وعجلة الفأرة للتكبير',
    enter() {
      out = buildReadouts([
        { key: 'render', label: 'زمن الرسم' },
        { key: 'work', label: 'تكرارات الإطار' },
        { key: 'zoom', label: 'التكبير' },
        { key: 'iter', label: 'الحد الأقصى' },
        { key: 're', label: 'المركز ‎Re' },
        { key: 'im', label: 'المركز ‎Im' },
      ]);
      fit();
      draw();
    },
    leave() { out = null; },
    frame() {},          // renders on interaction, not on a clock
    resize: fit,
  };
})();

/* ── module: Game of Life ────────────────────────────────────────────── */

const life = (() => {
  const COLS = 320, ROWS = 200, CELL = 3;
  const canvas = $('life-canvas');
  const ctx = canvas.getContext('2d');
  const board = new Life(COLS, ROWS, CELL);

  let running = true;
  let perFrame = 1;
  let out = null;
  let painting = 0;
  let lastMs = 0;

  function cellAt(e) {
    const box = canvas.getBoundingClientRect();
    return {
      x: Math.floor(((e.clientX - box.left) / box.width) * COLS),
      y: Math.floor(((e.clientY - box.top) / box.height) * ROWS),
    };
  }

  canvas.addEventListener('pointerdown', (e) => {
    painting = 1;
    canvas.setPointerCapture(e.pointerId);
    const c = cellAt(e);
    board.toggle(c.x, c.y);
    blit();
  });
  canvas.addEventListener('pointermove', (e) => {
    if (!painting) return;
    const c = cellAt(e);
    board.toggle(c.x, c.y);
    blit();
  });
  canvas.addEventListener('pointerup', () => { painting = 0; });

  function blit() {
    board.render();
    ctx.putImageData(
      imageFrom(wasm.memory, board.pixels_ptr(), board.pixels_len(), COLS * CELL, ROWS * CELL), 0, 0
    );
  }

  const playBtn = $('l-play');
  playBtn.addEventListener('click', () => {
    running = !running;
    playBtn.textContent = running ? 'إيقاف' : 'تشغيل';
  });
  $('l-step').addEventListener('click', () => { board.tick(); blit(); });
  $('l-random').addEventListener('click', () => {
    board.randomize(0.28, (Math.random() * 4294967295) >>> 0);
    blit();
  });
  $('l-clear').addEventListener('click', () => { board.clear(); blit(); });
  $('l-gun').addEventListener('click', () => { board.stamp_gun(12, 12); blit(); });
  $('l-speed').addEventListener('input', (e) => {
    perFrame = +e.target.value;
    $('l-speed-out').textContent = perFrame;
  });

  return {
    get board() { return board; },
    hint: 'ارسم على اللوح بالضغط والسحب',
    enter() {
      out = buildReadouts([
        { key: 'gen', label: 'الجيل' },
        { key: 'pop', label: 'الأحياء' },
        { key: 'step', label: 'زمن الخطوة' },
        { key: 'cells', label: 'الخلايا' },
      ]);
      setReadout(out.cells, fmt(COLS * ROWS));
      blit();
    },
    leave() { out = null; },
    frame() {
      const t0 = performance.now();
      if (running) for (let i = 0; i < perFrame; i++) board.tick();
      blit();
      lastMs = performance.now() - t0;
      if (out) {
        setReadout(out.gen, fmt(board.generation()));
        setReadout(out.pop, fmt(board.population()));
        setReadout(out.step, lastMs.toFixed(2), 'ms');
      }
      return lastMs;
    },
  };
})();

/* ── module: particle field ──────────────────────────────────────────── */

const field = (() => {
  const W = 960, H = 600;
  const canvas = $('field-canvas');
  const ctx = canvas.getContext('2d');
  let sim = new Field(W, H, 30000);

  let pointer = null;
  const wells = [];
  let out = null;

  function toField(e) {
    const box = canvas.getBoundingClientRect();
    return {
      x: ((e.clientX - box.left) / box.width) * W,
      y: ((e.clientY - box.top) / box.height) * H,
    };
  }

  canvas.addEventListener('pointermove', (e) => { pointer = toField(e); });
  canvas.addEventListener('pointerleave', () => { pointer = null; });
  canvas.addEventListener('pointerdown', (e) => {
    const p = toField(e);
    if (wells.length >= 3) wells.shift();
    wells.push(p);
  });

  $('p-count').addEventListener('input', (e) => {
    $('p-count-out').textContent = fmt(+e.target.value);
  });
  $('p-count').addEventListener('change', (e) => {
    sim.set_count(+e.target.value);
    paintMemory('field');
  });
  $('p-trail').addEventListener('input', (e) => {
    sim.set_trail(+e.target.value);
    $('p-trail-out').textContent = e.target.value;
  });
  $('p-scatter').addEventListener('click', () => sim.scatter());
  $('p-clear').addEventListener('click', () => { wells.length = 0; });

  return {
    get sim() { return sim; },
    hint: 'حرّك الفأرة لجذب الجسيمات، واضغط لتثبيت بئر',
    enter() {
      out = buildReadouts([
        { key: 'count', label: 'الجسيمات' },
        { key: 'step', label: 'التكامل' },
        { key: 'draw', label: 'الرسم' },
        { key: 'speed', label: 'متوسط السرعة' },
        { key: 'state', label: 'حالة الجسيمات' },
      ]);
      sim.set_trail(+$('p-trail').value);
    },
    leave() { out = null; },
    frame() {
      sim.clear_attractors();
      // The core the seeded velocities were balanced against; without it the
      // disc has nothing to orbit and just sits there.
      sim.add_attractor(W / 2, H / 2, sim.core_mass());
      for (const wl of wells) sim.add_attractor(wl.x, wl.y, 900000);
      if (pointer) sim.add_attractor(pointer.x, pointer.y, 1400000);

      const t0 = performance.now();
      sim.step(1 / 60);
      const t1 = performance.now();
      sim.render();
      const t2 = performance.now();

      ctx.putImageData(imageFrom(wasm.memory, sim.pixels_ptr(), sim.pixels_len(), W, H), 0, 0);

      if (out) {
        setReadout(out.count, fmt(sim.count()));
        setReadout(out.step, (t1 - t0).toFixed(2), 'ms');
        setReadout(out.draw, (t2 - t1).toFixed(2), 'ms');
        setReadout(out.speed, sim.mean_speed().toFixed(1), 'px/s');
        setReadout(out.state, bytes(sim.state_bytes()));
      }
      return t2 - t0;
    },
  };
})();

/* ── module: benchmark ───────────────────────────────────────────────── */

const bench = (() => {
  // Each JS routine mirrors its Rust and Go counterparts statement for statement.
  const js = {
    matmul(n) {
      const a = new Float64Array(n * n);
      const b = new Float64Array(n * n);
      for (let i = 0; i < n * n; i++) {
        a[i] = (i % 17) * 0.125 - 1.0;
        b[i] = (i % 23) * 0.0625 - 0.5;
      }
      const c = new Float64Array(n * n);
      for (let i = 0; i < n; i++) {
        for (let k = 0; k < n; k++) {
          const aik = a[i * n + k];
          if (aik === 0) continue;
          for (let j = 0; j < n; j++) c[i * n + j] += aik * b[k * n + j];
        }
      }
      let sum = 0;
      for (let i = 0; i < c.length; i++) sum += c[i];
      return sum;
    },
    sieve(limit) {
      if (limit < 2) return 0;
      const composite = new Uint8Array(limit);
      let count = 0;
      for (let i = 2; i < limit; i++) {
        if (!composite[i]) {
          count++;
          for (let j = i * i; j < limit; j += i) composite[j] = 1;
        }
      }
      return count;
    },
    escape(side, maxIter) {
      let total = 0;
      for (let y = 0; y < side; y++) {
        const ci = -1.25 + 2.5 * (y / side);
        for (let x = 0; x < side; x++) {
          const cr = -2.0 + 3.0 * (x / side);
          let zr = 0, zi = 0, i = 0;
          while (i < maxIter && zr * zr + zi * zi <= 4.0) {
            const t = zr * zr - zi * zi + cr;
            zi = 2 * zr * zi + ci;
            zr = t;
            i++;
          }
          total += i;
        }
      }
      return total;
    },
    mix(rounds) {
      let h = 0x811c9dc5 >>> 0;
      for (let i = 0; i < rounds; i++) {
        h = (h ^ i) >>> 0;
        h = Math.imul(h, 0x01000193) >>> 0;
        h = (h ^ (h >>> 15)) >>> 0;
        h = (h + ((h << 7) >>> 0)) >>> 0;
      }
      return h >>> 0;
    },
    radix(n) {
      let src = new Uint32Array(n);
      let x = 0x92961e37 >>> 0;
      for (let i = 0; i < n; i++) {
        x = (x ^ (x << 13)) >>> 0;
        x = (x ^ (x >>> 17)) >>> 0;
        x = (x ^ (x << 5)) >>> 0;
        src[i] = x;
      }
      let dst = new Uint32Array(n);
      const count = new Uint32Array(256);
      for (const shift of [0, 8, 16, 24]) {
        count.fill(0);
        for (let i = 0; i < n; i++) count[(src[i] >>> shift) & 0xff]++;
        let sum = 0;
        for (let i = 0; i < 256; i++) { const c = count[i]; count[i] = sum; sum += c; }
        for (let i = 0; i < n; i++) {
          const v = src[i];
          const bucket = (v >>> shift) & 0xff;
          dst[count[bucket]] = v;
          count[bucket]++;
        }
        const swap = src; src = dst; dst = swap;
      }
      let h = 0x811c9dc5 >>> 0;
      for (let i = 0; i < n; i++) {
        h = (h ^ src[i]) >>> 0;
        h = Math.imul(h, 0x01000193) >>> 0;
      }
      return h >>> 0;
    },
    crc(len, rounds) {
      const table = new Uint32Array(256);
      for (let i = 0; i < 256; i++) {
        let c = i;
        for (let k = 0; k < 8; k++) c = (c & 1) ? ((0xedb88320 ^ (c >>> 1)) >>> 0) : (c >>> 1);
        table[i] = c;
      }
      const data = new Uint8Array(len);
      for (let i = 0; i < len; i++) data[i] = ((i * 167) ^ (i >> 3)) & 0xff;
      let crc = 0xffffffff >>> 0;
      for (let r = 0; r < rounds; r++) {
        for (let i = 0; i < len; i++) {
          crc = ((crc >>> 8) ^ table[(crc ^ data[i]) & 0xff]) >>> 0;
        }
      }
      return (crc ^ 0xffffffff) >>> 0;
    },
    queens(n) {
      const search = (cols, ld, rd, all) => {
        if (cols === all) return 1;
        let count = 0;
        let open = ~(cols | ld | rd) & all;
        while (open !== 0) {
          const bit = open & -open;
          open -= bit;
          count += search(cols | bit, ((ld | bit) << 1) & all, (rd | bit) >>> 1, all);
        }
        return count;
      };
      return search(0, 0, 0, (1 << n) - 1);
    },
    trees(maxDepth) {
      const build = (d) => (d === 0
        ? { left: null, right: null }
        : { left: build(d - 1), right: build(d - 1) });
      const check = (node) => (node.left === null ? 1 : 1 + check(node.left) + check(node.right));
      let total = 0;
      for (let depth = 4; depth <= maxDepth; depth += 2) {
        const iterations = 1 << (maxDepth - depth + 4);
        for (let i = 0; i < iterations; i++) total += check(build(depth));
      }
      return total;
    },
  };

  /* ── the Go module, fetched only when this panel is opened ──────────── */

  // Go ships its whole runtime and garbage collector in the binary, so it is
  // some thirty times the size of the Rust one and not worth paying for on a
  // page load that may never reach this panel.
  let go = null;

  function loadGo() {
    if (go) return go;
    go = new Promise((resolve, reject) => {
      const tag = document.createElement('script');
      tag.src = './pkg/wasm_exec.js';
      tag.onerror = () => reject(new Error('تعذّر تحميل pkg/wasm_exec.js'));
      tag.onload = async () => {
        try {
          globalThis.__goBenchReady = () => resolve(globalThis.goBench);
          const runtime = new globalThis.Go();
          const src = await WebAssembly.instantiateStreaming(
            fetch('./pkg/go_bench.wasm'), runtime.importObject);
          // `main` parks on `select {}` so the exported functions stay live;
          // this promise is not meant to settle.
          runtime.run(src.instance).catch(reject);
        } catch (err) {
          reject(err);
        }
      };
      document.head.appendChild(tag);
    });
    return go;
  }

  async function moduleSize(url) {
    const res = await fetch(url, { method: 'HEAD' });
    const len = Number(res.headers.get('content-length'));
    return Number.isFinite(len) && len > 0 ? bytes(len) : '—';
  }

  /* ── tasks ──────────────────────────────────────────────────────────── */

  const RUNTIMES = [
    { key: 'rust', label: 'Rust' },
    { key: 'go', label: 'Go' },
    { key: 'js', label: 'JS' },
  ];

  const TASKS = [
    {
      id: 'matmul', name: 'ضرب مصفوفتين', note: '‎420×420‎ بدقة مضاعفة، ‎O(n³)‎',
      warm: (n, g) => {
        for (let i = 0; i < n; i++) {
          bench_matmul(46); js.matmul(46);
          if (g) { g.matmul(46); g.matmulHinted(46); }
        }
      },
      rust: () => bench_matmul(420), js: () => js.matmul(420), go: (g) => g.matmul(420),
      variant: {
        key: 'goBce',
        label: 'Go مقطّعة',
        run: (g) => g.matmulHinted(420),
        note: 'نفس الحساب بنفس الترتيب، لكن بتقطيع الصفوف إلى شرائح فرعية — '
            + 'فيثبت المترجم أن الفهرس داخل المدى ويحذف فحص الحدود. الفارق هنا '
            + 'هو ثمن تلك الفحوص وحدها.',
      },
    },
    {
      id: 'sieve', name: 'غربال إراتوستينس', note: 'الأعداد الأولية دون عشرين مليوناً',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_sieve(60000); js.sieve(60000); if (g) g.sieve(60000); } },
      rust: () => bench_sieve(20000000), js: () => js.sieve(20000000), go: (g) => g.sieve(20000000),
    },
    {
      id: 'escape', name: 'حلقة الإفلات', note: 'شبكة ‎800×800‎ بحد ‎450‎ تكرار',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_escape(46, 70); js.escape(46, 70); if (g) g.escape(46, 70); } },
      rust: () => bench_escape(800, 450), js: () => js.escape(800, 450), go: (g) => g.escape(800, 450),
    },
    {
      id: 'mix', name: 'خلط عددي متسلسل', note: 'ستون مليون جولة، بلا تسريع شعاعي',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_mix(200000); js.mix(200000); if (g) g.mix(200000); } },
      rust: () => bench_mix(60000000), js: () => js.mix(60000000), go: (g) => g.mix(60000000),
    },
    {
      id: 'radix', name: 'فرز جذري', note: 'ستة ملايين كلمة، أربع مرات على ‎8‎ بتات',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_radix(20000); js.radix(20000); if (g) g.radix(20000); } },
      rust: () => bench_radix(6000000), js: () => js.radix(6000000), go: (g) => g.radix(6000000),
    },
    {
      id: 'crc', name: 'مجموع تحقق ‎CRC-32‎', note: 'ثمانية ميغابايت، مطوية أربع مرات',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_crc(20000, 1); js.crc(20000, 1); if (g) g.crc(20000, 1); } },
      rust: () => bench_crc(8000000, 4), js: () => js.crc(8000000, 4), go: (g) => g.crc(8000000, 4),
    },
    {
      id: 'queens', name: 'مسألة الوزيرات', note: 'لوح ‎14×14‎، بحث تراجعي بأقنعة بتّية',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_queens(8); js.queens(8); if (g) g.queens(8); } },
      rust: () => bench_queens(14), js: () => js.queens(14), go: (g) => g.queens(14),
    },
    {
      id: 'trees', name: 'أشجار ثنائية', note: 'ثلاثة ملايين عقدة قصيرة العمر',
      warm: (n, g) => { for (let i = 0; i < n; i++) { bench_trees(8); js.trees(8); if (g) g.trees(8); } },
      rust: () => bench_trees(14), js: () => js.trees(14), go: (g) => g.trees(14),
    },
  ];

  // A variant is a second way of writing the same task in a language already
  // on the board. It sits beside its own runtime and is excluded from the
  // verdict — it is evidence about that runtime, not a fourth contestant.
  const lanesOf = (task) => (task.variant
    ? [RUNTIMES[0], RUNTIMES[1], task.variant, RUNTIMES[2]]
    : RUNTIMES);

  const host = $('bench-rows');
  host.innerHTML = TASKS.map((t) => `
    <div class="row" id="row-${t.id}">
      <div class="row-head">
        <span class="task">${t.name}<small>${t.note}</small></span>
        <span class="verdict" id="verdict-${t.id}">لم تُشغَّل</span>
      </div>
      <div class="bars">
        ${lanesOf(t).map((r) => `
          <div class="bar-line"><span class="who">${r.label}</span>
            <span class="bar-track"><span class="bar-fill ${r.key}" id="fill-${r.key}-${t.id}"></span></span>
            <span class="ms" id="ms-${r.key}-${t.id}">—</span></div>`).join('')}
      </div>
      ${t.variant ? `<p class="variant-note">${t.variant.note}</p>` : ''}
    </div>`).join('');

  const WARM_PASSES = 30;
  const MEASURED_PASSES = 3;
  let status = null;

  const yieldFrame = () => new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));

  async function run() {
    const btn = $('b-run');
    btn.disabled = true;
    $('bench-checks').textContent = '';

    let goApi = null;
    try {
      btn.textContent = 'جارٍ تحميل Go…';
      goApi = await loadGo();
    } catch (err) {
      $('bench-checks').innerHTML = `<b>لم تُحمَّل وحدة Go:</b> ${err.message}. القياس يمضي بمتسابقَين.`;
    }

    const running = RUNTIMES.filter((r) => r.key !== 'go' || goApi);
    btn.textContent = 'جارٍ القياس…';
    const marks = [];
    let done = 0;
    if (status) setReadout(status.state, `0 من ${TASKS.length}`);

    for (const t of TASKS) {
      const verdict = $(`verdict-${t.id}`);
      verdict.className = 'verdict';
      verdict.textContent = 'جارٍ…';
      await yieldFrame();

      // Warm every runtime, then take the best of three passes each. Without
      // this V8 is still running Wasm in its baseline tier and the numbers
      // describe the compilers rather than the code.
      t.warm(WARM_PASSES, goApi);
      await yieldFrame();

      // The variant runs on the Go module, so it is only measurable when Go is.
      const measured = t.variant && goApi ? [...running, t.variant] : running;

      const best = {};
      const sums = {};
      for (const r of measured) best[r.key] = Infinity;

      for (let pass = 0; pass < MEASURED_PASSES; pass++) {
        for (const r of measured) {
          const t0 = performance.now();
          sums[r.key] = r.run ? r.run(goApi) : r.key === 'go' ? t.go(goApi) : t[r.key]();
          best[r.key] = Math.min(best[r.key], performance.now() - t0);
          await yieldFrame();
        }
      }

      const scale = Math.max(...measured.map((r) => best[r.key]));
      const fastest = running.reduce((a, r) => (best[r.key] < best[a.key] ? r : a), running[0]);
      const slowest = Math.max(...running.map((r) => best[r.key]));

      for (const r of lanesOf(t)) {
        const fill = $(`fill-${r.key}-${t.id}`);
        const ms = $(`ms-${r.key}-${t.id}`);
        if (!measured.includes(r)) {
          fill.style.width = '0%';
          ms.textContent = '—';
          continue;
        }
        fill.style.width = `${(best[r.key] / scale) * 100}%`;
        ms.textContent = `${best[r.key].toFixed(1)} ms`;
      }

      verdict.innerHTML =
        `الأسرع ${fastest.label}، بفارق <span class="num">${(slowest / best[fastest.key]).toFixed(2)}×</span>`;
      verdict.className = `verdict win-${fastest.key}`;

      const values = measured.map((r) => sums[r.key]);
      marks.push({ name: t.name, ok: values.every((v) => Object.is(v, values[0])) });

      done++;
      if (status) setReadout(status.state, `${done} من ${TASKS.length}`);
      await yieldFrame();
    }

    const bad = marks.filter((m) => !m.ok);
    const line = bad.length === 0
      ? `<b>البصمات متطابقة في المهام كلها</b> — المتسابقون أدّوا العمل نفسه.`
      : `<b>بصمات غير متطابقة:</b> ${bad.map((m) => m.name).join('، ')}`;
    $('bench-checks').innerHTML = $('bench-checks').innerHTML
      ? `${$('bench-checks').innerHTML}<br>${line}` : line;

    btn.disabled = false;
    btn.textContent = 'إعادة التشغيل';
  }

  $('b-run').addEventListener('click', run);

  return {
    hint: '',
    enter() {
      status = buildReadouts([
        { key: 'tasks', label: 'المهام' },
        { key: 'state', label: 'المكتمل', text: true },
        { key: 'rustSize', label: 'وحدة Rust' },
        { key: 'goSize', label: 'وحدة Go' },
      ]);
      setReadout(status.tasks, String(TASKS.length));
      setReadout(status.state, `0 من ${TASKS.length}`);
      moduleSize('./pkg/wasm_demo_bg.wasm').then((s) => setReadout(status.rustSize, s));
      moduleSize('./pkg/go_bench.wasm').then((s) => setReadout(status.goSize, s));
      // Fetching 1.9 MB now means the run button does not stall on it later.
      loadGo().catch(() => {});
    },
    leave() {},
    frame() {},
  };
})();

/* ── shell wiring ────────────────────────────────────────────────────── */

const MODULES = { fractal, life, field, bench };
let activeKey = 'fractal';

function activate(key) {
  if (key === activeKey) return;
  MODULES[activeKey].leave();
  activeKey = key;

  for (const btn of $('rail').children) {
    btn.setAttribute('aria-selected', String(btn.dataset.panel === key));
  }
  for (const name of Object.keys(MODULES)) {
    $(`view-${name}`).dataset.active = name === key ? '1' : '0';
    $(`ctl-${name}`).dataset.active = name === key ? '1' : '0';
  }

  $('stage').dataset.panel = key;
  $('hint').textContent = MODULES[key].hint;
  $('hint').style.display = MODULES[key].hint ? '' : 'none';
  timings.fill(0);

  MODULES[key].enter();
  paintMemory(key);
}

for (const btn of $('rail').children) {
  btn.addEventListener('click', () => { location.hash = btn.dataset.panel; });
}

function routeFromHash() {
  const key = location.hash.slice(1);
  if (key in MODULES) activate(key);
}
addEventListener('hashchange', routeFromHash);

let memoryAt = 0;

function loop(now) {
  const ms = MODULES[activeKey].frame();
  if (ms !== undefined) pushTiming(ms);

  // The heap only ever grows, so a poll a few times a second is enough.
  if (now - memoryAt > 400) {
    memoryAt = now;
    paintMemory(activeKey);
  }
  requestAnimationFrame(loop);
}

// The stage settles after fonts and layout, so watch the box itself rather
// than the window — a window `resize` never fires for that first reflow.
new ResizeObserver(() => {
  if (activeKey === 'fractal') fractal.resize();
}).observe($('stage'));

$('stage').dataset.panel = 'fractal';
$('hint').textContent = fractal.hint;
fractal.enter();
paintMemory('fractal');
routeFromHash();
requestAnimationFrame(loop);
