# App Icons - Placeholder Notice

The icon files in this directory are **temporary placeholders** and should be replaced before building production releases.

## Current Placeholder Files

| File | Size | Purpose |
|---|---|---|
| `32x32.png` | 32×32 px | Taskbar/dock small icon |
| `128x128.png` | 128×128 px | Desktop/taskbar large icon |
| `icon.ico` | ICO format (multi-size) | Windows executable icon |

## How to Replace

### Option 1: Generate from a single PNG with Tauri CLI

If you have a single high-resolution PNG (1024×1024 recommended), run:

```bash
npm run tauri icon path/to/your/icon.png
```

Tauri CLI will auto-generate all required sizes and formats in this directory.

### Option 2: Manual replacement

Ensure each file matches the dimensions listed above:
- **32x32.png** – PNG, 32×32, 32-bit RGBA recommended
- **128x128.png** – PNG, 128×128, 32-bit RGBA recommended
- **icon.ico** – Windows ICO format, should contain 16×16, 32×32, 48×48, 64×64, 128×128 sizes
- *(Optional)* Add macOS `icon.icns` and Linux `512x512.png` / `32x32.png` variants for cross-platform builds

## Design Guidelines for Production Icons

- Use a square canvas (1:1 aspect ratio)
- Leave ~10% safe padding around the logo to avoid clipping in rounded OS masks
- Export as PNG-24 / PNG-32 with transparency
- Test at small sizes (16px, 32px) — thin lines and fine details may become unreadable
- Consider Windows 11 rounded corners and macOS Big Sur+ squircle shapes when designing

## Build Note

Until replaced, Tauri builds will use these placeholder icons and may show a compiler warning. The app will still run normally in development mode.
