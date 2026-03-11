// The snippet.mjs has deep internal bare-specifier imports that break every CDN's
// ES module pipeline (jsDelivr /+esm, esm.sh, etc.)
//
// The fix: fetch the pre-bundled JS as plain text (fetch() has permissive CORS),
// wrap it in a Blob with the correct MIME type, then dynamic-import from that
// Blob URL. Blob URLs are same-origin, so the browser imposes zero module restrictions.

let editor = null;
const preview = document.getElementById('preview-pane');
const status  = document.getElementById('compiler-status');

let compileTimeout;
let $typst = null;
let mathSchemaCode = null;

// auth.js is loaded as a separate <script> before this module — setupAuth/getAuthToken are globals
setupAuth('btn-login-github', 'auth-status');

function setStatus(message, className) {
    status.textContent = message;
    status.className   = className;
}

async function loadTypst() {
    // ?bundle=all tells esm.sh to inline ALL transitive dependencies into one file,
    // so the resulting Blob has no remaining bare specifiers to resolve.
    const CDN_URL = "https://esm.sh/@myriaddreamin/typst.ts@0.4.1/dist/esm/contrib/snippet.mjs?bundle=all";

    const res = await fetch(CDN_URL);
    if (!res.ok) throw new Error(`CDN fetch failed: ${res.status}`);

    let code    = await res.text();
    
    // esm.sh replaces bare specifiers with absolute paths like "/idb@^7.1.1/..." 
    // which fail to resolve when evaluated from a Blob URL.
    // We rewrite them to absolute esm.sh URLs.
    code = code.replace(/from\s+["'](\/[^"']+)["']/g, 'from "https://esm.sh$1"');
    code = code.replace(/import\s+["'](\/[^"']+)["']/g, 'import "https://esm.sh$1"');

    const blob    = new Blob([code], { type: "application/javascript" });
    const blobURL = URL.createObjectURL(blob);

    try {
        const mod = await import(blobURL);
        return mod.$typst;
    } finally {
        URL.revokeObjectURL(blobURL); // clean up immediately after import
    }
}

async function initCompiler() {
    try {
        $typst = await loadTypst();

        // .wasm files are raw binary assets — no module-import restrictions apply
        // In typst.ts v0.4.0+ getModule must return the module bytes (ArrayBuffer/Uint8Array) 
        // or a compiled WebAssembly.Module. The string URL is no longer accepted.
        await $typst.setCompilerInitOptions({
            getModule: () =>
                fetch("https://unpkg.com/@myriaddreamin/typst-ts-web-compiler@0.4.1/pkg/typst_ts_web_compiler_bg.wasm")
                    .then(response => response.arrayBuffer())
                    .then(buffer => new Uint8Array(buffer)),
        });

        await $typst.setRendererInitOptions({
            getModule: () =>
                fetch("https://unpkg.com/@myriaddreamin/typst-ts-renderer@0.4.1/pkg/typst_ts_renderer_bg.wasm")
                    .then(response => response.arrayBuffer())
                    .then(buffer => new Uint8Array(buffer)),
        });

        setStatus("WASM: Ready", "status-ready");
        renderPreview();
    } catch (error) {
        console.error("Failed to load Typst WASM:", error);
        setStatus("WASM: Failed to load", "status-error");
        preview.innerHTML = `<div class="preview-loading" style="color:var(--accent-red)">
            Compilation engine failed to load. Check the console.
        </div>`;
    }
}

async function renderPreview() {
    if (!$typst || !editor) return;
    setStatus("Compiling...", "status-loading");

    const rawCode = editor.getValue();

    // Strip the atlas metadata header (everything before and including ---)
    const bodyMatch = rawCode.match(/---[\s\S]*$/);
    const bodyCode  = bodyMatch ? bodyMatch[0].replace('---', '').trim() : rawCode;

    if (mathSchemaCode === null) {
        try {
            const res = await fetch("./math-graph.typ?timestamp=" + Date.now());
            if (res.ok) mathSchemaCode = await res.text();
            else console.error("Failed to fetch math-graph.typ:", res.status);
        } catch (e) {
            console.error("Network error fetching schema:", e);
        }
    }

    const wrappedCode = `
#set page(width: 500pt, height: auto, margin: 10pt, fill: rgb("#282a36"))
#set text(fill: rgb("#f8f8f2"), size: 14pt)

${mathSchemaCode || ""}

${bodyCode}
    `;

    try {
        const svg = await $typst.svg({ mainContent: wrappedCode });
        preview.innerHTML = svg;
        setStatus("WASM: Ready", "status-ready");
    } catch (error) {
        console.warn("Typst syntax error:", error);
        setStatus("Syntax Error", "status-error");
    }
}

initCompiler();

const initialCode = `// id: my-theorem
// type: theorem
// deps: []
// tags: [algebra]
---
#statement[
  Let $x$ be a real number. Then $x^2 >= 0$.
]

#intuition[
  A negative times a negative is positive.
]

#proof[
  Trivial by the axioms of the real numbers.
]`;

require(['vs/editor/editor.main'], function () {
    monaco.languages.register({ id: 'typst' });

    monaco.languages.setMonarchTokensProvider('typst', {
        tokenizer: {
            root: [
                [/^\/\/.*$/, 'comment'],
                [/\/\*[\s\S]*?\*\//, 'comment'],
                [/#(statement|intuition|proof|theorem|lemma|definition|corollary|remark|axiom|let|set|import)/, 'keyword'],
                [/\$[\s\S]*?\$|\$[^$]*\$/, 'string.html'],
                [/---/, 'keyword'],
            ]
        }
    });

    monaco.languages.setLanguageConfiguration('typst', {
        wordPattern: /#?[a-zA-Z0-9_]+/,
        brackets: [
            ['{', '}'],
            ['[', ']'],
            ['(', ')']
        ],
        autoClosingPairs: [
            { open: '{', close: '}' },
            { open: '[', close: ']' },
            { open: '(', close: ')' },
            { open: '"', close: '"' },
            { open: "'", close: "'" },
            { open: '$', close: '$' }
        ]
    });

    monaco.languages.registerCompletionItemProvider('typst', {
        provideCompletionItems: (model, position) => {
            const suggestions = [
                {
                    label: '#statement',
                    kind: monaco.languages.CompletionItemKind.Snippet,
                    insertText: '#statement[\n  $1\n]',
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    documentation: 'Insert a theorem statement block'
                },
                {
                    label: '#intuition',
                    kind: monaco.languages.CompletionItemKind.Snippet,
                    insertText: '#intuition[\n  $1\n]',
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    documentation: 'Insert an intuition block'
                },
                {
                    label: '#proof',
                    kind: monaco.languages.CompletionItemKind.Snippet,
                    insertText: '#proof[\n  $1\n]',
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    documentation: 'Insert a proof block'
                },
                {
                    label: '#remark',
                    kind: monaco.languages.CompletionItemKind.Snippet,
                    insertText: '#remark[\n  $1\n]',
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    documentation: 'Insert a remark callout block'
                },
                {
                    label: '#corollary',
                    kind: monaco.languages.CompletionItemKind.Snippet,
                    insertText: '#corollary[\n  $1\n]',
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    documentation: 'Insert a corollary callout block'
                }
            ];
            return { suggestions };
        }
    });

    const savedCode = localStorage.getItem('atlas-ide-content');

    editor = monaco.editor.create(document.getElementById('editor-container'), {
        value: savedCode !== null ? savedCode : initialCode,
        language: 'typst',
        theme: 'vs-dark',
        minimap: { enabled: false },
        automaticLayout: true,
        wordWrap: 'on',
        fontSize: 14,
        fontFamily: "'Menlo', 'Monaco', monospace"
    });

    editor.onDidChangeModelContent(() => {
        localStorage.setItem('atlas-ide-content', editor.getValue());
        clearTimeout(compileTimeout);
        compileTimeout = setTimeout(renderPreview, 300);
    });
    
    // Initial compile once editor is ready
    if ($typst) renderPreview();
});

// --- Download Handlers ---

function downloadBlob(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a   = document.createElement('a');
    a.href     = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

document.getElementById('btn-download-typ').addEventListener('click', () => {
    if (!editor) return;
    const content = editor.getValue();
    const blob = new Blob([content], { type: 'text/plain' });
    downloadBlob(blob, 'document.typ');
});

document.getElementById('btn-download-pdf').addEventListener('click', async () => {
    if (!$typst || !editor) return;

    const rawCode = editor.getValue();
    const bodyMatch = rawCode.match(/---[\s\S]*$/);
    const bodyCode  = bodyMatch ? bodyMatch[0].replace('---', '').trim() : rawCode;

    const wrappedCode = `
#set page(width: 500pt, margin: 10pt, fill: rgb("#282a36"))
#set text(fill: rgb("#f8f8f2"), size: 14pt)

${mathSchemaCode || ""}

${bodyCode}
    `;

    setStatus("Exporting PDF...", "status-loading");
    try {
        const pdfDv = await $typst.pdf({ mainContent: wrappedCode });
        const blob  = new Blob([pdfDv], { type: 'application/pdf' });
        downloadBlob(blob, 'document.pdf');
        setStatus("WASM: Ready", "status-ready");
    } catch (err) {
        console.error("PDF export failed:", err);
        setStatus("PDF Error", "status-error");
    }
});

// --- Publish Handler ---

const ideStatusMessage = document.getElementById('ide-status-message');
const btnPublish = document.getElementById('btn-publish');

let ideStatusTimer = null;

function showIdeStatus(type, html) {
    clearTimeout(ideStatusTimer);
    ideStatusMessage.className = `show ${type}`;
    ideStatusMessage.innerHTML = html;

    // Auto-dismiss success toasts after 6 seconds
    if (type === 'success') {
        ideStatusTimer = setTimeout(() => {
            ideStatusMessage.className = '';
            ideStatusMessage.innerHTML = '';
        }, 6000);
    }
}

btnPublish.addEventListener('click', async () => {
    if (!editor) return;

    const content = editor.getValue();

    // Validate: must have a metadata header with id and type before ---
    const headerMatch = content.match(/^([\s\S]*?)---/);
    if (!headerMatch) {
        showIdeStatus('error', '⚠ Missing metadata header. Your file must start with <code>// id: ...</code> and <code>// type: ...</code> before the <code>---</code> separator.');
        return;
    }
    const header = headerMatch[1];
    if (!header.includes('// id:') || !header.includes('// type:')) {
        showIdeStatus('error', '⚠ Metadata header is incomplete. Both <code>// id: ...</code> and <code>// type: ...</code> are required before the <code>---</code> separator.');
        return;
    }

    // Build a File object (named so the server gets a .typ filename)
    const blob = new Blob([content], { type: 'text/plain' });
    const file = new File([blob], 'document.typ', { type: 'text/plain' });

    const formData = new FormData();
    formData.append('file', file);

    btnPublish.disabled = true;
    btnPublish.textContent = 'Publishing...';
    showIdeStatus('loading', '<span class="loading-spinner"></span> Submitting to Atlas...');

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

        const rawText = await response.text();
        let data;
        try {
            data = JSON.parse(rawText);
        } catch (e) {
            // Raw text error from the Rust backend
            showIdeStatus('error', `⚠ ${rawText}`);
            btnPublish.disabled = false;
            btnPublish.textContent = 'Publish to Atlas';
            return;
        }

        if (response.ok) {
            if (data.pr_url) {
                showIdeStatus('success', `✓ Submitted successfully!<br><a href="${data.pr_url}" target="_blank" class="pr-link-btn">View Pull Request ↗</a>`);
            } else {
                showIdeStatus('success', `✓ ${data.message}`);
            }
            btnPublish.textContent = 'Published!';
        } else {
            showIdeStatus('error', `⚠ ${data.message || data.error || 'An unknown error occurred.'}`);
            btnPublish.disabled = false;
            btnPublish.textContent = 'Publish to Atlas';
        }

    } catch (err) {
        showIdeStatus('error', `⚠ Network error: ${err.message}. Is the Atlas server running?`);
        btnPublish.disabled = false;
        btnPublish.textContent = 'Publish to Atlas';
    }
});
