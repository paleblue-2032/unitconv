let wasm = null;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

// Rust コア（unitconv_core.wasm）を読み込む。
export async function init(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`wasm を読めません: ${response.status}`);
  }
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  wasm = instance.exports;
}

function writeStr(text) {
  const bytes = encoder.encode(text);
  const pointer = wasm.unitconv_alloc(bytes.length);
  const memory = new Uint8Array(wasm.memory.buffer);
  memory.set(bytes, pointer);
  memory[pointer + bytes.length] = 0;
  return [pointer, bytes.length];
}

function readStr(pointer) {
  const memory = new Uint8Array(wasm.memory.buffer);
  let end = pointer;
  while (memory[end] !== 0) end += 1;
  return decoder.decode(memory.subarray(pointer, end));
}

function call(fn, args) {
  const buffers = args.map(writeStr);
  const resultPointer = fn(...buffers.map(([pointer]) => pointer));
  const json = readStr(resultPointer);
  wasm.unitconv_free(resultPointer);
  buffers.forEach(([pointer, length]) => wasm.unitconv_dealloc(pointer, length));
  return json;
}

export function convert(value, from, to) {
  if (!wasm) return '{"ok":false,"error":"wasm 未初期化"}';
  return call(wasm.unitconv_convert, [value ?? '', from ?? '', to ?? '']);
}

export function scale(value, from, to) {
  if (!wasm) return '{"ok":false,"error":"wasm 未初期化"}';
  return call(wasm.unitconv_scale, [value ?? '', from ?? '', to ?? '']);
}

export function prefixes() {
  if (!wasm) return '[]';
  const pointer = wasm.unitconv_prefixes();
  const json = readStr(pointer);
  wasm.unitconv_free(pointer);
  return json;
}

export function units() {
  if (!wasm) return '[]';
  const pointer = wasm.unitconv_units();
  const json = readStr(pointer);
  wasm.unitconv_free(pointer);
  return json;
}
