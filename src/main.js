const API_URL = 'http://localhost:4000';

// Global state
let currentPageHTML = '';
let isProcessing = false;

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    initializeEventListeners();
    checkBackendConnection();
});

function initializeEventListeners() {
    // File upload handling
    document.getElementById('cv-file').addEventListener('change', handleFileUpload);
    
    // Button event listeners
    document.getElementById('analyze-btn').addEventListener('click', analyzePage);
    document.getElementById('generate-btn').addEventListener('click', generateDSL);
    document.getElementById('run-btn').addEventListener('click', runAutomation);
    document.getElementById('clear-btn').addEventListener('click', clearForm);
    
    // Template buttons
    document.querySelectorAll('.template-btn').forEach(btn => {
        btn.addEventListener('click', (e) => loadTemplate(e.target.dataset.template));
    });
    
    // Form validation
    document.getElementById('target-url').addEventListener('input', validateURL);
    document.getElementById('email').addEventListener('input', validateEmail);
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
        document.getElementById('cv-path').textContent = `📄 Wybrany plik: ${file.name}`;
        document.getElementById('cv-path').dataset.path = path;
        
        // Update file label
        const label = document.querySelector('.file-label');
        label.textContent = `📄 ${file.name}`;
        label.classList.add('file-selected');
        
        showStatus('✅ Plik CV został wybrany', 'success');
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
        currentPageHTML = data.html || '';
        
        updateProgress(100);
        showStatus('✅ Strona przeanalizowana pomyślnie', 'success');
        
        // Enable generate button
        document.getElementById('generate-btn').disabled = false;
        
    } catch (error) {
        console.error('Analysis error:', error);
        showStatus(`❌ Błąd analizy: ${error.message}`, 'error');
        updateProgress(0);
    } finally {
        setProcessing(false);
    }
}

// Generate DSL script
async function generateDSL() {
    if (!currentPageHTML) {
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
                html: currentPageHTML,
                user_data: userData
            })
        });
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        updateProgress(75);
        
        const data = await response.json();
        const script = data.script || '';
        
        document.getElementById('dsl-script').value = script;
        
        updateProgress(100);
        showStatus('✅ Skrypt DSL wygenerowany pomyślnie', 'success');
        
        // Enable run button
        document.getElementById('run-btn').disabled = false;
        
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
    const script = document.getElementById('dsl-script').value.trim();
    
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
        
    } catch (error) {
        console.error('Automation error:', error);
        showStatus(`❌ Błąd wykonania: ${error.message}`, 'error');
        updateProgress(0);
    } finally {
        setProcessing(false);
    }
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
        document.getElementById('target-url').value = '';
        document.getElementById('dsl-script').value = '';
        document.getElementById('cv-file').value = '';
        document.getElementById('cv-path').textContent = '';
        document.getElementById('cv-path').dataset.path = '';
        
        // Reset file label
        const label = document.querySelector('.file-label');
        label.textContent = '📄 Wybierz plik CV';
        label.classList.remove('file-selected');
        
        // Reset state
        currentPageHTML = '';
        updateProgress(0);
        showStatus('✨ Formularz został wyczyszczony', 'info');
        
        // Disable buttons
        document.getElementById('generate-btn').disabled = true;
        document.getElementById('run-btn').disabled = true;
    }
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
    
    document.getElementById('dsl-script').value = templateScript;
    showStatus(`📋 Załadowano szablon: ${templateType}`, 'info');
    
    // Enable run button
    document.getElementById('run-btn').disabled = false;
}

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
