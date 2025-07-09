#!/usr/bin/env node
const fs = require('fs');
console.log('[npm_to_boltpm] running');
fs.writeFileSync('bolt.lock', JSON.stringify({ bolt: true }, null, 2));
process.exit(0); 