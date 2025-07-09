// pnpm_to_boltpm.js
// Usage: node pnpm_to_boltpm.js
const fs = require('fs');
const yaml = require('js-yaml');
const lock = yaml.load(fs.readFileSync('pnpm-lock.yaml', 'utf8'));
let boltLock = {};
for (const [name, version] of Object.entries(lock.dependencies || {})) {
  boltLock[name] = version;
}
fs.writeFileSync('bolt.lock', JSON.stringify(boltLock, null, 2));
console.log('bolt.lock generated from pnpm-lock.yaml'); 