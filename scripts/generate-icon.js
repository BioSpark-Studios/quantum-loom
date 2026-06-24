const fs = require('fs');
const path = require('path');
const pngToIco = require('png-to-ico');

(async () => {
  try {
    const repoRoot = path.resolve(__dirname, '..');
    const candidates = [
      path.join(repoRoot, 'assets', 'icon-1024.png'),
      path.join(repoRoot, 'assets', 'icon-512.png'),
      path.join(repoRoot, 'assets', 'icon.png')
    ];

    const src = candidates.find(p => fs.existsSync(p));
    if (!src) {
      console.error('No source PNG found. Please add one of: assets/icon-1024.png, assets/icon-512.png, or assets/icon.png');
      process.exit(1);
    }

    const dest = path.join(repoRoot, 'assets', 'icon.ico');
    const buf = await pngToIco(src);
    fs.writeFileSync(dest, buf);
    console.log('Wrote', dest);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
