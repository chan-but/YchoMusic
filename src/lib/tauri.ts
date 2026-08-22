import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// Silence Tauri "Couldn't find callback id" warnings (app reload during Rust async ops)
if (typeof window !== 'undefined') {
  const origWarn = console.warn.bind(console);
  console.warn = function (...args: unknown[]) {
    const first = typeof args[0] === 'string' ? args[0] : '';
    if (first.includes('[TAURI]') && first.includes('Couldn\'t find callback id')) {
      return;
    }
    origWarn(...args);
  };
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return null as T;
  }
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (e) {
    const errMsg = e instanceof Error ? e.message : String(e);
    if (!errMsg.includes('Couldn\'t find callback id') && !errMsg.includes('callback id')) {
      console.warn(`[Tauri invoke] ${cmd} failed:`, errMsg);
    }
    throw e;
  }
}
