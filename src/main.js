const API_URL = 'http://localhost:4000';

// Global state management
let appState = {
    currentPageHTML: '',
    isProcessing: false,
    activeTab: 'dashboard',
    bitwardenStatus: 'logged-out', // logged-out, locked, unlocked
    sessionId: null,
    credentials: [],
    logs: [],
    logFilters: {
        level: 'all',
        component: 'all',
        search: ''
    },
    settings: {
        theme: 'dark',
        autoSave: true,
        notifications: true,
        logRetention: 30
    }
};

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    initializeApp();
});

async function initializeApp() {
    try {
        showStatus('🚀 Inicjalizowanie aplikacji...', 'info');
        
        // Initialize UI components
        initializeEventListeners();
        initializeSidebar();
        initializeTabSystem();
        
        // Check backend connection
        await checkBackendConnection();
        
        // Load user session if exists
        await loadUserSession();
        
        // Load settings
        loadSettings();
        
        // Set initial tab
        showTab('dashboard');
        
        showStatus('✅ Aplikacja gotowa do użycia', 'success');
    } catch (error) {
        console.error('Initialization error:', error);
        showStatus('❌ Błąd inicjalizacji aplikacji', 'error');
    }
}

function initializeEventListeners() {
    // Dashboard form elements
    const cvFile = document.getElementById('cv-file');
    const analyzeBtn = document.getElementById('analyze-btn');
    const generateBtn = document.getElementById('generate-btn');
    const runBtn = document.getElementById('run-btn');
    const clearBtn = document.getElementById('clear-btn');
    const targetUrl = document.getElementById('target-url');
    const email = document.getElementById('email');
    
    if (cvFile) cvFile.addEventListener('change', handleFileUpload);
    if (analyzeBtn) analyzeBtn.addEventListener('click', analyzePage);
    if (generateBtn) generateBtn.addEventListener('click', generateDSL);
    if (runBtn) runBtn.addEventListener('click', runAutomation);
    if (clearBtn) clearBtn.addEventListener('click', clearForm);
    if (targetUrl) targetUrl.addEventListener('input', validateURL);
    if (email) email.addEventListener('input', validateEmail);
    
    // Template buttons
    document.querySelectorAll('.template-btn').forEach(btn => {
        btn.addEventListener('click', (e) => loadTemplate(e.target.dataset.template));
    });
    
    // Bitwarden form elements
    const bwLoginBtn = document.getElementById('bw-login-btn');
    const bwUnlockBtn = document.getElementById('bw-unlock-btn');
    const bwRefreshBtn = document.getElementById('bw-refresh-btn');
    const bwLogoutBtn = document.getElementById('bw-logout-btn');
    const bwSearchInput = document.getElementById('bw-search');
    
    if (bwLoginBtn) bwLoginBtn.addEventListener('click', handleBitwardenLogin);
    if (bwUnlockBtn) bwUnlockBtn.addEventListener('click', handleBitwardenUnlock);
    if (bwRefreshBtn) bwRefreshBtn.addEventListener('click', refreshBitwardenCredentials);
    if (bwLogoutBtn) bwLogoutBtn.addEventListener('click', handleBitwardenLogout);
    if (bwSearchInput) bwSearchInput.addEventListener('input', filterCredentials);
    
    // Logging panel elements
    const logRefreshBtn = document.getElementById('log-refresh-btn');
    const logClearBtn = document.getElementById('log-clear-btn');
    const logLevelFilter = document.getElementById('log-level-filter');
    const logComponentFilter = document.getElementById('log-component-filter');
    const logSearchInput = document.getElementById('log-search');
    
    if (logRefreshBtn) logRefreshBtn.addEventListener('click', refreshLogs);
    if (logClearBtn) logClearBtn.addEventListener('click', clearLogs);
    if (logLevelFilter) logLevelFilter.addEventListener('change', updateLogFilters);
    if (logComponentFilter) logComponentFilter.addEventListener('change', updateLogFilters);
    if (logSearchInput) logSearchInput.addEventListener('input', updateLogFilters);
    
    // Settings elements
    const themeSelect = document.getElementById('theme-select');
    const autoSaveCheck = document.getElementById('auto-save-check');
    const notificationsCheck = document.getElementById('notifications-check');
    
    if (themeSelect) themeSelect.addEventListener('change', updateSettings);
    if (autoSaveCheck) autoSaveCheck.addEventListener('change', updateSettings);
    if (notificationsCheck) notificationsCheck.addEventListener('change', updateSettings);
}

// Initialize sidebar navigation
function initializeSidebar() {
    const sidebarItems = document.querySelectorAll('.sidebar-item');
    sidebarItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const tabId = item.dataset.tab;
            if (tabId) {
                showTab(tabId);
            }
        });
    });
}

// Initialize tab system
function initializeTabSystem() {
    // Hide all tabs initially except dashboard
    const tabs = document.querySelectorAll('.tab-content');
    tabs.forEach(tab => {
        if (tab.id !== 'dashboard-tab') {
            tab.style.display = 'none';
        }
    });
}

// Show specific tab and update sidebar
function showTab(tabId) {
    // Hide all tabs
    const tabs = document.querySelectorAll('.tab-content');
    tabs.forEach(tab => tab.style.display = 'none');
    
    // Show selected tab
    const selectedTab = document.getElementById(`${tabId}-tab`);
    if (selectedTab) {
        selectedTab.style.display = 'block';
    }
    
    // Update sidebar active state
    const sidebarItems = document.querySelectorAll('.sidebar-item');
    sidebarItems.forEach(item => {
        item.classList.remove('active');
        if (item.dataset.tab === tabId) {
            item.classList.add('active');
        }
    });
    
    // Update app state
    appState.activeTab = tabId;
    
    // Initialize tab-specific functionality
    switch (tabId) {
        case 'bitwarden':
            updateBitwardenUI();
            break;
        case 'logs':
            refreshLogs();
            break;
        case 'settings':
            loadSettingsUI();
            break;
    }
}

// Check if backend is running
async function checkBackendConnection() {
    try {
        const response = await fetch(`${API_URL}/health`);
        if (response.ok) {
            showStatus('✅ Połączenie z backendem nawiązane', 'success');
        } else {
            showStatus('⚠️ Problem z backendem', 'warning');
        }
    } catch (error) {
        showStatus('❌ Brak połączenia z backendem', 'error');
    }
}

// Session Management Functions
async function loadUserSession() {
    try {
        const response = await fetch(`${API_URL}/session/get`, {
            credentials: 'include'
        });
        
        if (response.ok) {
            const sessionData = await response.json();
            appState.sessionId = sessionData.session_id;
            
            // Restore form data if available
            if (sessionData.user_data) {
                restoreFormData(sessionData.user_data);
            }
            
            showNotification('✅ Sesja użytkownika wczytana', 'success');
        }
    } catch (error) {
        console.error('Session load error:', error);
    }
}

async function saveUserSession() {
    if (!appState.settings.autoSave) return;
    
    try {
        const userData = collectUserData();
        const response = await fetch(`${API_URL}/session/create`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({
                user_data: userData,
                form_data: userData
            })
        });
        
        if (response.ok) {
            const sessionData = await response.json();
            appState.sessionId = sessionData.session_id;
        }
    } catch (error) {
        console.error('Session save error:', error);
    }
}

// Bitwarden Integration Functions
async function handleBitwardenLogin() {
    const email = document.getElementById('bw-email').value.trim();
    const password = document.getElementById('bw-password').value.trim();
    
    if (!email || !password) {
        showStatus('❌ Podaj email i hasło do Bitwarden', 'error');
        return;
    }
    
    setProcessing(true);
    showStatus('🔑 Logowanie do Bitwarden...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/bitwarden/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        
        if (response.ok) {
            const data = await response.json();
            appState.bitwardenStatus = 'locked';
            updateBitwardenUI();
            showStatus('✅ Zalogowano do Bitwarden', 'success');
        } else {
            const error = await response.json();
            showStatus(`❌ Błąd logowania: ${error.error}`, 'error');
        }
    } catch (error) {
        console.error('Bitwarden login error:', error);
        showStatus('❌ Błąd połączenia z Bitwarden', 'error');
    } finally {
        setProcessing(false);
    }
}

async function handleBitwardenUnlock() {
    const masterPassword = document.getElementById('bw-master-password').value.trim();
    
    if (!masterPassword) {
        showStatus('❌ Podaj hasło główne', 'error');
        return;
    }
    
    setProcessing(true);
    showStatus('🔓 Odblokowywanie sejfu...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/bitwarden/unlock`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ master_password: masterPassword })
        });
        
        if (response.ok) {
            appState.bitwardenStatus = 'unlocked';
            updateBitwardenUI();
            await refreshBitwardenCredentials();
            showStatus('✅ Sejf odblokowany', 'success');
            
            // Clear master password field
            document.getElementById('bw-master-password').value = '';
        } else {
            const error = await response.json();
            showStatus(`❌ Błąd odblokowywania: ${error.error}`, 'error');
        }
    } catch (error) {
        console.error('Bitwarden unlock error:', error);
        showStatus('❌ Błąd odblokowywania sejfu', 'error');
    } finally {
        setProcessing(false);
    }
}

// Handle file upload
function handleFileUpload(event) {
    const file = event.target.files[0];
    if (file) {
        // Validate file type
        const allowedTypes = ['.pdf', '.doc', '.docx'];
        const fileExtension = '.' + file.name.split('.').pop().toLowerCase();
        
        if (!allowedTypes.includes(fileExtension)) {
            showStatus('❌ Niewspierany format pliku. Użyj PDF, DOC lub DOCX', 'error');
            return;
        }
        
        // Display file path (Tauri provides file path)
        const path = file.path || `${getDefaultDocumentsPath()}/${file.name}`;
        const cvPathElement = document.getElementById('cv-path');
        if (cvPathElement) {
            cvPathElement.textContent = `📄 Wybrany plik: ${file.name}`;
            cvPathElement.dataset.path = path;
        }
        
        // Update file label
        const label = document.querySelector('.file-label');
        if (label) {
            label.textContent = `📄 ${file.name}`;
            label.classList.add('file-selected');
        }
        
        showStatus('✅ Plik CV został wybrany', 'success');
        
        // Auto-save session
        saveUserSession();
    }
}

async function refreshBitwardenCredentials() {
    if (appState.bitwardenStatus !== 'unlocked') {
        showStatus('❌ Najpierw odblokuj sejf Bitwarden', 'error');
        return;
    }
    
    setProcessing(true);
    showStatus('🔄 Pobieranie danych uwierzytelniających...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/bitwarden/credentials`);
        
        if (response.ok) {
            const data = await response.json();
            appState.credentials = data.credentials || [];
            displayCredentials(appState.credentials);
            showStatus('✅ Dane uwierzytelniające pobrane', 'success');
        } else {
            const error = await response.json();
            showStatus(`❌ Błąd pobierania: ${error.error}`, 'error');
        }
    } catch (error) {
        console.error('Credentials fetch error:', error);
        showStatus('❌ Błąd pobierania danych', 'error');
    } finally {
        setProcessing(false);
    }
}

async function handleBitwardenLogout() {
    if (confirm('🚪 Czy na pewno chcesz się wylogować z Bitwarden?')) {
        appState.bitwardenStatus = 'logged-out';
        appState.credentials = [];
        updateBitwardenUI();
        displayCredentials([]);
        showStatus('👋 Wylogowano z Bitwarden', 'info');
    }
}

function updateBitwardenUI() {
    const loginForm = document.getElementById('bw-login-form');
    const unlockForm = document.getElementById('bw-unlock-form');
    const vaultContent = document.getElementById('bw-vault-content');
    const statusBadge = document.getElementById('bw-status');
    
    if (!loginForm || !unlockForm || !vaultContent || !statusBadge) return;
    
    // Update status badge
    statusBadge.className = `status-badge ${appState.bitwardenStatus}`;
    
    switch (appState.bitwardenStatus) {
        case 'logged-out':
            statusBadge.textContent = 'Wylogowany';
            loginForm.style.display = 'block';
            unlockForm.style.display = 'none';
            vaultContent.style.display = 'none';
            break;
            
        case 'locked':
            statusBadge.textContent = 'Zablokowany';
            loginForm.style.display = 'none';
            unlockForm.style.display = 'block';
            vaultContent.style.display = 'none';
            break;
            
        case 'unlocked':
            statusBadge.textContent = 'Odblokowany';
            loginForm.style.display = 'none';
            unlockForm.style.display = 'none';
            vaultContent.style.display = 'block';
            break;
    }
}

function getDefaultDocumentsPath() {
    // Platform-specific document paths
    const platform = window.navigator.platform.toLowerCase();
    if (platform.includes('win')) {
        return 'C:\\Users\\User\\Documents';
    } else if (platform.includes('mac')) {
        return '/Users/User/Documents';
    } else {
        return '/home/user/Documents';
    }
}

// Display credentials in the vault
function displayCredentials(credentials) {
    const container = document.getElementById('bw-credentials-list');
    if (!container) return;
    
    container.innerHTML = '';
    
    if (credentials.length === 0) {
        container.innerHTML = '<div class="empty-state">Brak danych uwierzytelniających</div>';
        return;
    }
    
    credentials.forEach(credential => {
        const credentialCard = document.createElement('div');
        credentialCard.className = 'credential-card';
        credentialCard.innerHTML = `
            <div class="credential-header">
                <h4>${escapeHtml(credential.name || 'Bez nazwy')}</h4>
                <div class="credential-actions">
                    <button class="btn-icon" onclick="useCredential('${credential.id}')" title="Użyj danych">
                        <span>📋</span>
                    </button>
                    <button class="btn-icon" onclick="copyToClipboard('${credential.password || ''}')" title="Kopiuj hasło">
                        <span>🔑</span>
                    </button>
                </div>
            </div>
            <div class="credential-details">
                <div class="credential-field">
                    <label>Email/Login:</label>
                    <span>${escapeHtml(credential.login || 'Brak')}</span>
                </div>
                <div class="credential-field">
                    <label>URL:</label>
                    <span>${escapeHtml(credential.url || 'Brak')}</span>
                </div>
                <div class="credential-field">
                    <label>Notatki:</label>
                    <span>${escapeHtml(credential.notes || 'Brak')}</span>
                </div>
            </div>
        `;
        container.appendChild(credentialCard);
    });
}

// Filter credentials based on search input
function filterCredentials() {
    const searchTerm = document.getElementById('bw-search').value.toLowerCase().trim();
    
    if (!searchTerm) {
        displayCredentials(appState.credentials);
        return;
    }
    
    const filtered = appState.credentials.filter(credential => 
        (credential.name && credential.name.toLowerCase().includes(searchTerm)) ||
        (credential.login && credential.login.toLowerCase().includes(searchTerm)) ||
        (credential.url && credential.url.toLowerCase().includes(searchTerm))
    );
    
    displayCredentials(filtered);
}

// Use credential to fill form
async function useCredential(credentialId) {
    const credential = appState.credentials.find(c => c.id === credentialId);
    if (!credential) return;
    
    // Fill dashboard form if on dashboard tab
    if (appState.activeTab === 'dashboard') {
        const emailField = document.getElementById('email');
        const usernameField = document.getElementById('username');
        const passwordField = document.getElementById('password');
        
        if (emailField && credential.login) {
            emailField.value = credential.login;
        }
        if (usernameField && credential.login) {
            usernameField.value = credential.login;
        }
        if (passwordField && credential.password) {
            passwordField.value = credential.password;
        }
        
        // Auto-save session
        saveUserSession();
        
        showNotification(`✅ Użyto danych: ${credential.name}`, 'success');
        
        // Switch to dashboard tab
        showTab('dashboard');
    }
}

// Analyze page with CDP
async function analyzePage() {
    const url = document.getElementById('target-url').value.trim();
    
    if (!url) {
        showStatus('❌ Podaj URL strony do analizy', 'error');
        return;
    }
    
    if (!isValidURL(url)) {
        showStatus('❌ Podaj prawidłowy URL (z http:// lub https://)', 'error');
        return;
    }
    
    setProcessing(true);
    updateProgress(25);
    showStatus('🔍 Analizuję stronę...', 'info');
    
    try {
        // Load URL in Tauri webview (if available)
        if (window.__TAURI__ && window.__TAURI__.invoke) {
            await window.__TAURI__.invoke('load_url', { url });
        }
        
        updateProgress(50);
        
        // Fetch HTML through CDP
        const response = await fetch(`${API_URL}/page/analyze`);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        appState.currentPageHTML = data.html || '';
        
        updateProgress(100);
        showStatus('✅ Strona przeanalizowana pomyślnie', 'success');
        
        // Enable generate button
        const generateBtn = document.getElementById('generate-btn');
        if (generateBtn) generateBtn.disabled = false;
        
        // Auto-save session
        saveUserSession();
        
    } catch (error) {
        console.error('Analysis error:', error);
        showStatus(`❌ Błąd analizy: ${error.message}`, 'error');
        updateProgress(0);
    } finally {
        setProcessing(false);
    }
}

// Logging Panel Functions
async function refreshLogs() {
    setProcessing(true);
    showStatus('📋 Pobieranie logów...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/logs/get`);
        
        if (response.ok) {
            const data = await response.json();
            appState.logs = data.logs || [];
            displayLogs(appState.logs);
            updateLogStatistics();
            showStatus('✅ Logi odświeżone', 'success');
        } else {
            showStatus('❌ Błąd pobierania logów', 'error');
        }
    } catch (error) {
        console.error('Logs fetch error:', error);
        showStatus('❌ Błąd połączenia z serwerem logów', 'error');
    } finally {
        setProcessing(false);
    }
}

async function clearLogs() {
    if (confirm('🗑️ Czy na pewno chcesz wyczyścić wszystkie logi?')) {
        try {
            const response = await fetch(`${API_URL}/logs/clear`, { method: 'POST' });
            
            if (response.ok) {
                appState.logs = [];
                displayLogs([]);
                updateLogStatistics();
                showStatus('✅ Logi wyczyszczone', 'success');
            } else {
                showStatus('❌ Błąd czyszczenia logów', 'error');
            }
        } catch (error) {
            console.error('Clear logs error:', error);
            showStatus('❌ Błąd czyszczenia logów', 'error');
        }
    }
}

function updateLogFilters() {
    const levelFilter = document.getElementById('log-level-filter');
    const componentFilter = document.getElementById('log-component-filter');
    const searchInput = document.getElementById('log-search');
    
    appState.logFilters = {
        level: levelFilter?.value || 'all',
        component: componentFilter?.value || 'all',
        search: searchInput?.value?.toLowerCase().trim() || ''
    };
    
    const filteredLogs = filterLogs(appState.logs);
    displayLogs(filteredLogs);
}

function filterLogs(logs) {
    return logs.filter(log => {
        // Filter by level
        if (appState.logFilters.level !== 'all' && log.level !== appState.logFilters.level) {
            return false;
        }
        
        // Filter by component
        if (appState.logFilters.component !== 'all' && log.component !== appState.logFilters.component) {
            return false;
        }
        
        // Filter by search term
        if (appState.logFilters.search) {
            const searchTerm = appState.logFilters.search;
            const searchableText = `${log.message} ${log.component}`.toLowerCase();
            if (!searchableText.includes(searchTerm)) {
                return false;
            }
        }
        
        return true;
    });
}

// Display logs in the logging panel
function displayLogs(logs) {
    const container = document.getElementById('log-entries');
    if (!container) return;
    
    container.innerHTML = '';
    
    if (logs.length === 0) {
        container.innerHTML = '<div class="empty-state">Brak logów do wyświetlenia</div>';
        return;
    }
    
    logs.reverse().forEach(log => {
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry log-${log.level}`;
        
        const timestamp = new Date(log.timestamp).toLocaleString('pl-PL');
        
        logEntry.innerHTML = `
            <div class="log-header">
                <span class="log-level">${log.level.toUpperCase()}</span>
                <span class="log-component">${escapeHtml(log.component)}</span>
                <span class="log-timestamp">${timestamp}</span>
            </div>
            <div class="log-message">${escapeHtml(log.message)}</div>
        `;
        
        container.appendChild(logEntry);
    });
    
    // Auto-scroll to bottom
    container.scrollTop = container.scrollHeight;
}

function updateLogStatistics() {
    const stats = {
        total: appState.logs.length,
        error: appState.logs.filter(l => l.level === 'error').length,
        warn: appState.logs.filter(l => l.level === 'warn').length,
        info: appState.logs.filter(l => l.level === 'info').length,
        debug: appState.logs.filter(l => l.level === 'debug').length
    };
    
    const statsContainer = document.getElementById('log-stats');
    if (statsContainer) {
        statsContainer.innerHTML = `
            <div class="stat-item">
                <span class="stat-label">Łącznie:</span>
                <span class="stat-value">${stats.total}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Błędy:</span>
                <span class="stat-value stat-error">${stats.error}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Ostrzeżenia:</span>
                <span class="stat-value stat-warn">${stats.warn}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Info:</span>
                <span class="stat-value stat-info">${stats.info}</span>
            </div>
        `;
    }
}

// Generate DSL script
async function generateDSL() {
    if (!appState.currentPageHTML) {
        showStatus('❌ Najpierw przeanalizuj stronę', 'error');
        return;
    }
    
    const userData = collectUserData();
    
    if (!validateUserData(userData)) {
        return;
    }
    
    setProcessing(true);
    updateProgress(25);
    showStatus('⚡ Generuję skrypt DSL...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/dsl/generate`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json' 
            },
            body: JSON.stringify({
                html: appState.currentPageHTML,
                user_data: userData
            })
        });
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        updateProgress(75);
        
        const data = await response.json();
        const script = data.script || '';
        
        const dslScript = document.getElementById('dsl-script');
        if (dslScript) dslScript.value = script;
        
        updateProgress(100);
        showStatus('✅ Skrypt DSL wygenerowany pomyślnie', 'success');
        
        // Enable run button
        const runBtn = document.getElementById('run-btn');
        if (runBtn) runBtn.disabled = false;
        
        // Auto-save session
        saveUserSession();
        
    } catch (error) {
        console.error('Generation error:', error);
        showStatus(`❌ Błąd generowania: ${error.message}`, 'error');
        updateProgress(0);
    } finally {
        setProcessing(false);
    }
}

// Run automation
async function runAutomation() {
    const dslScript = document.getElementById('dsl-script');
    const script = dslScript?.value?.trim();
    
    if (!script) {
        showStatus('❌ Najpierw wygeneruj skrypt DSL', 'error');
        return;
    }
    
    // Confirm automation execution
    if (!confirm('🤖 Czy na pewno chcesz uruchomić automatyzację?\n\nSkrypt zostanie wykonany w przeglądarce.')) {
        return;
    }
    
    setProcessing(true);
    updateProgress(25);
    showStatus('🚀 Uruchamiam automatyzację...', 'info');
    
    try {
        const response = await fetch(`${API_URL}/rpa/run`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json' 
            },
            body: JSON.stringify({ 
                script 
            })
        });
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        updateProgress(75);
        
        const data = await response.json();
        
        updateProgress(100);
        
        if (data.success) {
            showStatus('🎉 Automatyzacja zakończona sukcesem!', 'success');
        } else {
            showStatus('⚠️ Automatyzacja zakończona z błędami', 'warning');
        }
        
        // Auto-save session
        saveUserSession();
        
    } catch (error) {
        console.error('Automation error:', error);
        showStatus(`❌ Błąd wykonania: ${error.message}`, 'error');
        updateProgress(0);
    } finally {
        setProcessing(false);
    }
}

// Settings Management Functions
function loadSettings() {
    const savedSettings = localStorage.getItem('codialog_settings');
    if (savedSettings) {
        try {
            appState.settings = { ...appState.settings, ...JSON.parse(savedSettings) };
        } catch (error) {
            console.error('Settings load error:', error);
        }
    }
    applySettings();
}

function saveSettings() {
    localStorage.setItem('codialog_settings', JSON.stringify(appState.settings));
}

function updateSettings() {
    const themeSelect = document.getElementById('theme-select');
    const autoSaveCheck = document.getElementById('auto-save-check');
    const notificationsCheck = document.getElementById('notifications-check');
    
    if (themeSelect) appState.settings.theme = themeSelect.value;
    if (autoSaveCheck) appState.settings.autoSave = autoSaveCheck.checked;
    if (notificationsCheck) appState.settings.notifications = notificationsCheck.checked;
    
    saveSettings();
    applySettings();
    showNotification('✅ Ustawienia zapisane', 'success');
}

function applySettings() {
    // Apply theme
    document.body.className = `theme-${appState.settings.theme}`;
    
    // Apply other settings as needed
    if (!appState.settings.notifications) {
        // Disable notifications if needed
    }
}

function loadSettingsUI() {
    const themeSelect = document.getElementById('theme-select');
    const autoSaveCheck = document.getElementById('auto-save-check');
    const notificationsCheck = document.getElementById('notifications-check');
    
    if (themeSelect) themeSelect.value = appState.settings.theme;
    if (autoSaveCheck) autoSaveCheck.checked = appState.settings.autoSave;
    if (notificationsCheck) notificationsCheck.checked = appState.settings.notifications;
}

// Clear form
function clearForm() {
    if (confirm('🗑️ Czy na pewno chcesz wyczyścić wszystkie dane?')) {
        // Clear form inputs
        document.getElementById('fullname').value = '';
        document.getElementById('email').value = '';
        document.getElementById('username').value = '';
        document.getElementById('password').value = '';
        document.getElementById('phone').value = '';
        const targetUrl = document.getElementById('target-url');
        const dslScript = document.getElementById('dsl-script');
        const cvFile = document.getElementById('cv-file');
        const cvPath = document.getElementById('cv-path');
        
        if (targetUrl) targetUrl.value = '';
        if (dslScript) dslScript.value = '';
        if (cvFile) cvFile.value = '';
        if (cvPath) {
            cvPath.textContent = '';
            cvPath.dataset.path = '';
        }
        
        // Reset file label
        const label = document.querySelector('.file-label');
        if (label) {
            label.textContent = '📄 Wybierz plik CV';
            label.classList.remove('file-selected');
        }
        
        // Reset state
        appState.currentPageHTML = '';
        updateProgress(0);
        showStatus('✨ Formularz został wyczyszczony', 'info');
        
        // Disable buttons
        const generateBtn = document.getElementById('generate-btn');
        const runBtn = document.getElementById('run-btn');
        if (generateBtn) generateBtn.disabled = true;
        if (runBtn) runBtn.disabled = true;
        
        // Auto-save session
        saveUserSession();
    }
}

// Utility Functions
function collectUserData() {
    const fullnameField = document.getElementById('fullname');
    const emailField = document.getElementById('email');
    const usernameField = document.getElementById('username');
    const passwordField = document.getElementById('password');
    const phoneField = document.getElementById('phone');
    const cvPathField = document.getElementById('cv-path');
    
    return {
        fullname: fullnameField?.value?.trim() || '',
        email: emailField?.value?.trim() || '',
        username: usernameField?.value?.trim() || '',
        password: passwordField?.value?.trim() || '',
        phone: phoneField?.value?.trim() || '',
        cv_path: cvPathField?.dataset?.path || ''
    };
}

function restoreFormData(userData) {
    if (!userData) return;
    
    const fullnameField = document.getElementById('fullname');
    const emailField = document.getElementById('email');
    const usernameField = document.getElementById('username');
    const passwordField = document.getElementById('password');
    const phoneField = document.getElementById('phone');
    
    if (fullnameField && userData.fullname) fullnameField.value = userData.fullname;
    if (emailField && userData.email) emailField.value = userData.email;
    if (usernameField && userData.username) usernameField.value = userData.username;
    if (passwordField && userData.password) passwordField.value = userData.password;
    if (phoneField && userData.phone) phoneField.value = userData.phone;
}

function validateUserData(userData) {
    if (!userData.fullname) {
        showStatus('❌ Podaj imię i nazwisko', 'error');
        return false;
    }
    
    if (!userData.email || !isValidEmail(userData.email)) {
        showStatus('❌ Podaj prawidłowy adres email', 'error');
        return false;
    }
    
    return true;
}

function isValidURL(string) {
    try {
        new URL(string);
        return true;
    } catch (_) {
        return false;
    }
}

function isValidEmail(email) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

function validateURL() {
    const targetUrl = document.getElementById('target-url');
    const analyzeBtn = document.getElementById('analyze-btn');
    
    if (!targetUrl || !analyzeBtn) return;
    
    const url = targetUrl.value.trim();
    analyzeBtn.disabled = !(url && isValidURL(url));
}

function validateEmail() {
    const emailField = document.getElementById('email');
    if (!emailField) return;
    
    const email = emailField.value.trim();
    
    emailField.classList.remove('valid', 'invalid');
    
    if (email && isValidEmail(email)) {
        emailField.classList.add('valid');
    } else if (email) {
        emailField.classList.add('invalid');
    }
}

// Copy to clipboard utility
async function copyToClipboard(text) {
    try {
        await navigator.clipboard.writeText(text);
        showNotification('📋 Skopiowano do schowka', 'success');
    } catch (error) {
        console.error('Clipboard error:', error);
        showNotification('❌ Błąd kopiowania', 'error');
    }
}

// HTML escaping utility
function escapeHtml(unsafe) {
    if (typeof unsafe !== 'string') return '';
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

// Load predefined templates
function loadTemplate(templateType) {
    const userData = collectUserData();
    let templateScript = '';
    
    switch (templateType) {
        case 'job':
            templateScript = `click "#accept-cookies"
hover "#careers-link"
click "#careers-link"
click "#apply-now"
type "#first-name" "${userData.fullname.split(' ')[0] || ''}"
type "#last-name" "${userData.fullname.split(' ')[1] || ''}"
type "#email" "${userData.email}"
type "#phone" "${userData.phone}"
upload "#resume" "${userData.cv_path}"
click "#gdpr-consent"
click "#submit-application"`;
            break;
            
        case 'linkedin':
            templateScript = `click "#sign-in"
type "#username" "${userData.email}"
type "#password" "${userData.password}"
click "#sign-in-submit"
click ".jobs-apply-button"
upload "#resume-upload" "${userData.cv_path}"
type "#phone" "${userData.phone}"
click "#follow-company"
click "#submit-application"`;
            break;
            
        case 'registration':
            templateScript = `click "#register"
type "#username" "${userData.username}"
type "#email" "${userData.email}"
type "#password" "${userData.password}"
type "#confirm-password" "${userData.password}"
click "#terms-checkbox"
click "#create-account"`;
            break;
    }
    
    const dslScript = document.getElementById('dsl-script');
    if (dslScript) {
        dslScript.value = templateScript;
        showStatus(`📋 Załadowano szablon: ${templateType}`, 'info');
        
        // Enable run button
        const runBtn = document.getElementById('run-btn');
        if (runBtn) runBtn.disabled = false;
        
        // Auto-save session
        saveUserSession();
    }
}

// Status and notification functions
function showStatus(message, type) {
    const status = document.getElementById('status');
    if (!status) return;
    
    status.textContent = message;
    status.className = `status ${type}`;
    
    // Auto-hide success messages after 5 seconds
    if (type === 'success') {
        setTimeout(() => {
            if (status.textContent === message) {
                status.textContent = 'Gotowy do następnej operacji';
                status.className = 'status';
            }
        }, 5000);
    }
}

function showNotification(message, type) {
    if (!appState.settings.notifications) return;
    
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    
    // Add to notification container or create one
    let container = document.getElementById('notifications');
    if (!container) {
        container = document.createElement('div');
        container.id = 'notifications';
        container.className = 'notifications-container';
        document.body.appendChild(container);
    }
    
    container.appendChild(notification);
    
    // Auto-remove after 3 seconds
    setTimeout(() => {
        notification.remove();
    }, 3000);
}

function updateProgress(percentage) {
    const progressBar = document.getElementById('progress');
    if (!progressBar) return;
    
    progressBar.style.width = `${percentage}%`;
    
    if (percentage === 0) {
        progressBar.style.width = '0%';
    }
}

function setProcessing(processing) {
    appState.isProcessing = processing;
    
    // Disable/enable buttons during processing
    const buttons = document.querySelectorAll('button');
    buttons.forEach(btn => {
        if (processing) {
            btn.disabled = true;
            btn.classList.add('processing');
        } else {
            btn.classList.remove('processing');
            // Re-enable based on form state
            validateURL();
            if (appState.currentPageHTML) {
                const generateBtn = document.getElementById('generate-btn');
                if (generateBtn) generateBtn.disabled = false;
            }
            const dslScript = document.getElementById('dsl-script');
            if (dslScript?.value?.trim()) {
                const runBtn = document.getElementById('run-btn');
                if (runBtn) runBtn.disabled = false;
            }
        }
    });
}

// Auto-save functionality
setInterval(() => {
    if (appState.settings.autoSave && !appState.isProcessing) {
        saveUserSession();
    }
}, 30000); // Auto-save every 30 seconds

// Utility functions
function collectUserData() {
    return {
        fullname: document.getElementById('fullname').value.trim(),
        email: document.getElementById('email').value.trim(),
        username: document.getElementById('username').value.trim(),
        password: document.getElementById('password').value.trim(),
        phone: document.getElementById('phone').value.trim(),
        cv_path: document.getElementById('cv-path').dataset.path || ''
    };
}

function validateUserData(userData) {
    if (!userData.fullname) {
        showStatus('❌ Podaj imię i nazwisko', 'error');
        return false;
    }
    
    if (!userData.email || !isValidEmail(userData.email)) {
        showStatus('❌ Podaj prawidłowy adres email', 'error');
        return false;
    }
    
    return true;
}

function isValidURL(string) {
    try {
        new URL(string);
        return true;
    } catch (_) {
        return false;
    }
}

function isValidEmail(email) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

function validateURL() {
    const url = document.getElementById('target-url').value.trim();
    const analyzeBtn = document.getElementById('analyze-btn');
    
    if (url && isValidURL(url)) {
        analyzeBtn.disabled = false;
    } else {
        analyzeBtn.disabled = true;
    }
}

function validateEmail() {
    const email = document.getElementById('email').value.trim();
    const emailField = document.getElementById('email');
    
    if (email && isValidEmail(email)) {
        emailField.classList.remove('invalid');
        emailField.classList.add('valid');
    } else if (email) {
        emailField.classList.remove('valid');
        emailField.classList.add('invalid');
    } else {
        emailField.classList.remove('valid', 'invalid');
    }
}

function showStatus(message, type) {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = `status ${type}`;
    
    // Auto-hide success messages after 5 seconds
    if (type === 'success') {
        setTimeout(() => {
            if (status.textContent === message) {
                status.textContent = 'Gotowy do następnej operacji';
                status.className = 'status';
            }
        }, 5000);
    }
}

function updateProgress(percentage) {
    const progressBar = document.getElementById('progress');
    progressBar.style.width = `${percentage}%`;
    
    if (percentage === 0) {
        progressBar.style.width = '0%';
    }
}

function setProcessing(processing) {
    isProcessing = processing;
    
    // Disable/enable buttons during processing
    const buttons = document.querySelectorAll('button');
    buttons.forEach(btn => {
        if (processing) {
            btn.disabled = true;
            btn.classList.add('processing');
        } else {
            btn.classList.remove('processing');
            // Re-enable based on form state
            validateURL();
            if (currentPageHTML) {
                document.getElementById('generate-btn').disabled = false;
            }
            if (document.getElementById('dsl-script').value.trim()) {
                document.getElementById('run-btn').disabled = false;
            }
        }
    });
}
