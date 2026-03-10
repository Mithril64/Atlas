const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('typ-file');
const fileInfo = document.getElementById('file-info');
const fileName = document.getElementById('file-name');
const fileSize = document.getElementById('file-size');
const btnSubmit = document.getElementById('btn-submit');
const btnClear = document.getElementById('btn-clear');
const statusMessage = document.getElementById('status-message');

let selectedFile = null;

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
    if (!selectedFile) {
	showStatus('error', 'No file selected');
	return;
    }

    showStatus('loading', '<span class="loading-spinner"></span> Submitting and validating...');
    btnSubmit.disabled = true;

    try {
	const formData = new FormData();
	formData.append('file', selectedFile);

	const response = await fetch('/api/submit', {
	    method: 'POST',
	    body: formData
	});

	const data = await response.json();

	if (response.ok) {
	    showStatus('success', `${data.message}`);
	    btnClear.click();
	    setTimeout(() => {
		alert(`Success! Your submission "${data.id}" has been added to the graph.`);
	    }, 1000);
	} else {
	    showStatus('error', `${data}`);
	}
    } catch (error) {
	showStatus('error', `Network error: ${error.message}`);
    } finally {
	btnSubmit.disabled = !selectedFile;
    }
});

function showStatus(type, message) {
    statusMessage.className = `status-message show ${type}`;
    statusMessage.innerHTML = message;
}

