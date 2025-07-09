// BoltPM Desktop Frontend
window.addEventListener('DOMContentLoaded', () => {
  refreshLogs();
  refreshTree();
  refreshCache();
  refreshConfig();
  loadPackageJson();
  showSection('logs');
  setupDarkMode();
});

function setupDarkMode() {
  const html = document.documentElement;
  const btn = document.getElementById('darkToggle');
  // Load preference
  let theme = localStorage.getItem('boltpm-theme') || 'light';
  setTheme(theme);
  btn.onclick = () => {
    theme = (theme === 'dark') ? 'light' : 'dark';
    setTheme(theme);
    localStorage.setItem('boltpm-theme', theme);
  };
}
function setTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  const btn = document.getElementById('darkToggle');
  btn.textContent = theme === 'dark' ? '☀️' : '🌙';
}

function showSection(section) {
  // Hide all sections
  document.querySelectorAll('main section').forEach(s => s.classList.remove('active'));
  // Remove active from all nav buttons
  document.querySelectorAll('nav button').forEach(b => b.classList.remove('active'));
  // Show the selected section
  document.getElementById('section-' + section).classList.add('active');
  document.getElementById('nav-' + section).classList.add('active');
  // Focus first input/textarea in section for accessibility
  const firstInput = document.querySelector(`#section-${section} input, #section-${section} textarea`);
  if (firstInput) firstInput.focus();
}

function showError(msg) {
  const el = document.getElementById('errorMsg');
  el.textContent = msg;
  el.style.display = 'block';
  setTimeout(() => { el.style.display = 'none'; }, 5000);
}

async function refreshLogs() {
  try {
    const { invoke } = window.__TAURI__;
    const logs = await invoke('get_install_logs');
    document.getElementById('logs').textContent = logs;
  } catch (e) { showError('Failed to load logs: ' + e); }
}

async function refreshTree() {
  try {
    const { invoke } = window.__TAURI__;
    const tree = await invoke('get_dependency_tree');
    document.getElementById('tree').textContent = tree;
  } catch (e) { showError('Failed to load dependency tree: ' + e); }
}

async function searchPackages() {
  try {
    const { invoke } = window.__TAURI__;
    const query = document.getElementById('searchBox').value;
    const results = await invoke('search_packages', { query });
    const el = document.getElementById('searchResults');
    el.innerHTML = '';
    results.split(',').forEach(pkg => {
      if (pkg.trim()) {
        const span = document.createElement('span');
        span.textContent = pkg.trim();
        el.appendChild(span);
      }
    });
  } catch (e) { showError('Search failed: ' + e); }
}

async function installPackage() {
  try {
    const { invoke } = window.__TAURI__;
    const name = document.getElementById('installBox').value;
    const msg = await invoke('install_package', { name });
    alert(msg);
    refreshLogs();
    refreshTree();
  } catch (e) { showError('Install failed: ' + e); }
}

async function uninstallPackage() {
  try {
    const { invoke } = window.__TAURI__;
    const name = document.getElementById('uninstallBox').value;
    const msg = await invoke('uninstall_package', { name });
    alert(msg);
    refreshLogs();
    refreshTree();
  } catch (e) { showError('Uninstall failed: ' + e); }
}

async function loadPackageJson() {
  try {
    const { invoke } = window.__TAURI__;
    const json = await invoke('get_package_json');
    document.getElementById('packageJson').value = json;
  } catch (e) { showError('Failed to load package.json: ' + e); }
}

async function savePackageJson() {
  try {
    const { invoke } = window.__TAURI__;
    const json = document.getElementById('packageJson').value;
    const msg = await invoke('set_package_json', { json });
    alert(msg);
    loadPackageJson();
    refreshTree();
  } catch (e) { showError('Failed to save package.json: ' + e); }
}

async function refreshCache() {
  try {
    const { invoke } = window.__TAURI__;
    const size = await invoke('get_cache_size');
    document.getElementById('cacheSize').textContent = size;
  } catch (e) { showError('Failed to load cache size: ' + e); }
}

async function refreshConfig() {
  try {
    const { invoke } = window.__TAURI__;
    const config = await invoke('get_config');
    document.getElementById('config').textContent = config;
  } catch (e) { showError('Failed to load config: ' + e); }
} 