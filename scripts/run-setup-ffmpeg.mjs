import { execFileSync } from 'node:child_process';
import { platform } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

if (platform() === 'win32') {
  execFileSync(
    'powershell',
    ['-ExecutionPolicy', 'Bypass', '-File', join(__dirname, 'setup-ffmpeg.ps1')],
    { stdio: 'inherit' }
  );
} else {
  execFileSync('bash', [join(__dirname, 'setup-ffmpeg.sh')], { stdio: 'inherit' });
}
