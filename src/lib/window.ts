import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { isTauri } from '@/lib/tauri';

function getWin() {
  if (!isTauri()) return null;
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

const NORMAL_WIDTH = 1200;
const NORMAL_HEIGHT = 800;
const MINI_WIDTH = 380;
const MINI_HEIGHT = 152;

export async function setNormalMode() {
  const win = getWin();
  if (!win) return;
  try {
    const isFs = await win.isFullscreen();
    if (isFs) {
      await win.setFullscreen(false);
      await new Promise(r => setTimeout(r, 350));
    }
    await win.setAlwaysOnTop(false);
    await win.setSize(new LogicalSize(NORMAL_WIDTH, NORMAL_HEIGHT));
    await win.center();
    await win.setDecorations(true);
  } catch (e) {
    console.error('[diag] setNormalMode failed:', e);
  }
}

export async function setMiniMode() {
  const win = getWin();
  if (!win) return;
  try {
    const isFs = await win.isFullscreen();
    if (isFs) {
      await win.setFullscreen(false);
      await new Promise(r => setTimeout(r, 350));
    }
    await win.setDecorations(false);
    await win.setAlwaysOnTop(true);
    await win.setSize(new LogicalSize(MINI_WIDTH, MINI_HEIGHT));
    await win.center();
  } catch (e) {
    console.error('[diag] setMiniMode failed:', e);
  }
}

export async function setFullscreenMode() {
  const win = getWin();
  if (!win) return;
  try {
    await win.setDecorations(false);
    await win.setAlwaysOnTop(false);
    await win.setFullscreen(true);
  } catch (e) {
    console.error('[diag] setFullscreenMode failed:', e);
  }
}

export async function minimizeWindow() {
  const win = getWin();
  if (!win) return;
  try {
    await win.minimize();
  } catch (e) {
    console.error('[diag] minimizeWindow failed:', e);
  }
}

export async function closeWindow() {
  const win = getWin();
  if (!win) return;
  try {
    await win.close();
  } catch (e) {
    console.error('[diag] closeWindow failed:', e);
  }
}

export async function startDragging() {
  const win = getWin();
  if (!win) return;
  try {
    await win.startDragging();
  } catch (e) {
    console.error('[diag] startDragging failed:', e);
  }
}

export async function toggleAlwaysOnTop(): Promise<boolean> {
  const win = getWin();
  if (!win) return false;
  try {
    const current = await win.isAlwaysOnTop();
    await win.setAlwaysOnTop(!current);
    return !current;
  } catch (e) {
    console.error('[diag] toggleAlwaysOnTop failed:', e);
    return false;
  }
}

export async function setAlwaysOnTop(value: boolean): Promise<void> {
  const win = getWin();
  if (!win) return;
  try {
    await win.setAlwaysOnTop(value);
  } catch (e) {
    console.error('[diag] setAlwaysOnTop failed:', e);
  }
}
