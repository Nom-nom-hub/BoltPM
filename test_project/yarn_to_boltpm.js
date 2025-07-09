// yarn_to_boltpm.js
// Usage: node yarn_to_boltpm.js
const fs = require('fs');
const { parse } = require('@yarnpkg/lockfile');
const lock = parse(fs.readFileSync('yarn.lock', 'utf8'));
let boltLock = {};
for (const [key, entry] of Object.entries(lock.object)) {
  // key is like 'left-pad@^1.3.0', entry.version is the resolved version
  const name = key.split('@')[0];
  boltLock[name] = entry.version;
}
fs.writeFileSync('bolt.lock', JSON.stringify(boltLock, null, 2));
console.log('bolt.lock generated from yarn.lock'); 