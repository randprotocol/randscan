/* tslint:disable */
/* eslint-disable */

/**
 * Interpret a pasted key as `kind` ("viewing", "spend", "tx", "call") and describe it.
 */
export function key_info(input: string, kind: string): string;

/**
 * The nullifier a viewing key's note with commitment `cm` publishes when spent.
 */
export function nullifier_of(key_kind: string, key: string, cm_hex: string): string;

/**
 * Open a call's input transcript with the caller's viewing key ("viewing"/"spend"/"file"),
 * the auditor's viewing key ("auditor", same formats as viewing) or the per-call key ("call").
 */
export function open_call(h_in_hex: string, envelope_json: string, key_kind: string, key: string): string;

/**
 * Try to open one envelope with a key. `key_kind` is "viewing" (64-hex `nk`), "spend"
 * (64-hex spend key, derived locally), "file" (a key file JSON) or "tx" (32-byte transaction
 * key). Returns the opened note as JSON, or `null` when the key does not open it.
 */
export function open_note(cm_hex: string, envelope_json: string, key_kind: string, key: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly key_info: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly nullifier_of: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly open_call: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly open_note: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
