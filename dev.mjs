import { spawn } from 'node:child_process';

const tailwind = spawn('npx', ['tailwindcss', '-i', 'src/styles/global.css', '-o', 'public/tailwind.css', '--watch'], {
  stdio: 'inherit',
  shell: true,
});

const vite = spawn('npx', ['vite'], {
  stdio: 'inherit',
  shell: true,
});

tailwind.on('error', (e) => console.error('Tailwind error:', e));
vite.on('error', (e) => console.error('Vite error:', e));

process.on('SIGTERM', () => {
  tailwind.kill();
  vite.kill();
  process.exit(0);
});
