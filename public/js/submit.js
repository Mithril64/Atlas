const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('typ-file');
const fileInfo = document.getElementById('file-info');
const fileName = document.getElementById('file-name');
const fileSize = document.getElementById('file-size');
const btnSubmit = document.getElementById('btn-submit');
const btnClear = document.getElementById('btn-clear');
const statusMessage = document.getElementById('status-message');

let selectedFile = null;

// Auth is loaded as a separate <script defer> before this one — setupAuth/getAuthToken are globals
setupAuth('btn-login-github', 'auth-status');

// Click to select file
dropZone.addEventListener('click', () => fileInput.click());

// Drag and drop
dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('active');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('active');
});

dropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropZone.classList.remove('active');
    
    const files = e.dataTransfer.files;
    if (files.length > 0) {
	handleFileSelect(files[0]);
    }
});

// File input change
fileInput.addEventListener('change', (e) => {
    if (e.target.files.length > 0) {
	handleFileSelect(e.target.files[0]);
    }
});

function handleFileSelect(file) {
    if (!file.name.endsWith('.typ')) {
	showStatus('error', 'Please select a .typ file');
	return;
    }

    selectedFile = file;
    fileName.textContent = file.name;
    fileSize.textContent = (file.size / 1024).toFixed(2) + ' KB';
    
    fileInfo.classList.add('show');
    btnSubmit.disabled = false;
    dropZone.style.borderColor = '#50fa7b';
    dropZone.style.background = '#2a2c38';
}

btnClear.addEventListener('click', () => {
    selectedFile = null;
    fileInput.value = '';
    fileInfo.classList.remove('show');
    btnSubmit.disabled = true;
    dropZone.style.borderColor = '#8be9fd';
    dropZone.style.background = '#1e1f29';
    statusMessage.classList.remove('show');
});

btnSubmit.addEventListener('click', async () => {
        const file = fileInput.files[0];
        if (!file) {
            showStatus('error', 'Please select a file first.');
            return;
        }

        btnSubmit.disabled = true;
        btnSubmit.textContent = 'Uploading...';
        showStatus('', '');

        const formData = new FormData();
        formData.append('file', file);

        const headers = {};
        const token = getAuthToken();
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        try {
            const response = await fetch(`${window.ATLAS_API_URL}/api/submit`, {
                method: 'POST',
                body: formData,
                headers,
            });

            // 1. Read the raw text first so we don't crash on plain-text errors
            const rawText = await response.text();
            let data;
            
            try {
                // 2. Try to parse it as JSON (for successful responses)
                data = JSON.parse(rawText);
            } catch (e) {
                // 3. If it's not JSON, it's a raw text error from our Rust backend!
                showStatus('error', rawText);
                btnSubmit.disabled = false;
                btnSubmit.textContent = 'Push to Graph';
                return; 
            }

            // 4. Handle standard JSON responses
            if (response.ok) {
                if (data.pr_url) {
                    showStatus('success', `Success! Your math has been compiled.<br> <a href="${data.pr_url}" target="_blank" class="pr-link-btn">View Pull Request ↗</a>`);
                } else {
                    showStatus('success', `${data.message}`);
                }
                
                fileInput.value = '';
                document.getElementById('file-info').style.display = 'none';
                document.getElementById('drop-zone').style.display = 'block';
                
                btnSubmit.textContent = 'Pushed!';
                btnSubmit.disabled = true; 
            } else {
                showStatus('error', data.message || data.error || "An unknown error occurred.");
                btnSubmit.textContent = 'Push to Graph';
                btnSubmit.disabled = false;
            }

        } catch (error) {
            showStatus('error', `Network error: ${error.message}`);
            // Reset button on network error
            btnSubmit.textContent = 'Push to Graph';
            btnSubmit.disabled = false;
        } 
    });

function showStatus(type, message) {
    statusMessage.className = `status-message show ${type}`;
    statusMessage.innerHTML = message;
}

