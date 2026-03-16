const typeColors = {
    "axiom": "#ff5555",
    "definition": "#ffb86c",
    "lemma": "#8be9fd",
    "theorem": "#bd93f9",
    "ghost": "rgba(68, 71, 90, 0.6)"
};

// --- UI Controller ---
const UIController = {
    panel: document.getElementById('side-panel'),
    closeBtn: document.getElementById('close-panel-btn'),
    badge: document.getElementById('node-badge'),
    title: document.getElementById('node-title'),
    idDisplay: document.getElementById('node-id'),
    
    mathImage: document.getElementById('node-math-image'),
    placeholder: document.getElementById('node-placeholder'),
    btnCopy: document.getElementById('btn-copy'),
    btnDownload: document.getElementById('btn-download'),

    tabBtns: document.querySelectorAll('.tab-btn'),
    tabPanes: document.querySelectorAll('.tab-pane'),
    rawCodeDisplay: document.getElementById('node-raw-code'),
    depsListDisplay: document.getElementById('node-dependencies'),

    currentNode: null,

    init() {
        this.closeBtn.addEventListener('click', () => this.close());
        
        this.btnCopy.addEventListener('click', () => {
            if (this.currentNode && this.currentNode.body) {
                navigator.clipboard.writeText(this.currentNode.body).then(() => {
                    const originalText = this.btnCopy.textContent;
                    this.btnCopy.textContent = "Copied!";
                    setTimeout(() => this.btnCopy.textContent = originalText, 2000);
                });
            }
        });

        this.btnDownload.addEventListener('click', () => {
            if (this.currentNode && !this.currentNode.isGhost) {
                const primary = API_BASE ? `${API_BASE}/api/nodes/${this.currentNode.id}.pdf` : null;
                const hostFallback = API_BASE ? `${API_BASE}/nodes/${this.currentNode.id}.pdf` : null;
                const localFallback = `./nodes/${this.currentNode.id}.pdf`;

                const tryUrls = [primary, hostFallback, localFallback].filter(Boolean);

                const tryDownload = async () => {
                    for (const url of tryUrls) {
                        try {
                            const resp = await fetch(url, { mode: 'cors' });
                            if (!resp.ok) {
                                console.warn('[atlas] PDF fetch failed', url, resp.status);
                                continue;
                            }
                            const blob = await resp.blob();
                            const blobUrl = URL.createObjectURL(blob);
                            const link = document.createElement('a');
                            link.href = blobUrl;
                            link.download = `${this.currentNode.id}.pdf`;
                            document.body.appendChild(link);
                            link.click();
                            document.body.removeChild(link);
                            URL.revokeObjectURL(blobUrl);
                            return true;
                        } catch (e) {
                            console.warn('[atlas] PDF fetch error', url, e);
                        }
                    }
                    return false;
                };

                (async () => {
                    const ok = await tryDownload();
                    if (ok) return;
                    try {
                        console.warn('[atlas] PDF download failed, attempting client-side render');
                        const blobUrl = await renderTypstPdf(this.currentNode);
                        const link = document.createElement('a');
                        link.href = blobUrl;
                        link.download = `${this.currentNode.id}.pdf`;
                        document.body.appendChild(link);
                        link.click();
                        document.body.removeChild(link);
                        URL.revokeObjectURL(blobUrl);
                    } catch (e) {
                        console.error('[atlas] client-side PDF render failed', e);
                    }
                })();
            }
        });

        this.tabBtns.forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.tabBtns.forEach(b => b.classList.remove('active'));
                this.tabPanes.forEach(p => p.classList.remove('active'));
                
                e.target.classList.add('active');
                const targetId = e.target.getAttribute('data-target');
                document.getElementById(targetId).classList.add('active');
            });
        });
    },

    open(node) {
    this.currentNode = node; 
        
        this.badge.textContent = node.type;
        this.badge.style.backgroundColor = typeColors[node.type] || '#6272a4';
        this.idDisplay.textContent = node.id;
        
    this.title.textContent = displayNameFromId(node.id);

        this.tabBtns.forEach(b => b.classList.remove('active'));
        this.tabPanes.forEach(p => p.classList.remove('active'));
        this.tabBtns[0].classList.add('active');
        this.tabPanes[0].classList.add('active');

        this.rawCodeDisplay.textContent = node.body || "// No source code available.";

        this.depsListDisplay.innerHTML = '';
        if (node.deps && node.deps.length > 0) {
            node.deps.forEach(depId => {
                const target = nodesById.get(depId);
                const label = target ? displayNameFromId(target.id) : displayNameFromId(depId) || depId;
                const li = document.createElement('li');
                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'dep-link';
                btn.textContent = label;
                btn.title = depId;
                btn.addEventListener('click', () => {
                    const toOpen = nodesById.get(depId) || { id: depId, type: 'ghost', isGhost: true, deps: [] };
                    recenterOnNextOpen = depId;
                    UIController.open(toOpen);
                });
                li.appendChild(btn);
                this.depsListDisplay.appendChild(li);
            });
        } else {
            const li = document.createElement('li');
            li.textContent = "This node has no prerequisites.";
            li.className = "no-deps";
            this.depsListDisplay.appendChild(li);
        }

        if (node.isGhost) {
            this.mathImage.style.display = 'none';
            this.placeholder.style.display = 'block';
            this.placeholder.textContent = "This node hasn't been written yet.";
            this.placeholder.style.color = '#ffb86c';
            
            this.btnCopy.style.display = 'none';
            this.btnDownload.style.display = 'none';
        } else {
            this.placeholder.style.display = 'none';
            this.mathImage.style.display = 'block';
            
            this.btnCopy.style.display = 'block';
            this.btnDownload.style.display = 'block';
            
            const cacheBuster = new Date().getTime();
            const primarySrc = API_BASE ? `${API_BASE}/api/nodes/${node.id}.svg?t=${cacheBuster}` : null;
            const hostFallback = API_BASE ? `${API_BASE}/nodes/${node.id}.svg?t=${cacheBuster}` : null;
            const localFallback = `./nodes/${node.id}.svg?t=${cacheBuster}`;
            let triedHostFallback = false;
            let triedLocal = false;

            this.mathImage.onerror = (e) => {
                if (!triedHostFallback && primarySrc && this.mathImage.src === primarySrc) {
                    triedHostFallback = true;
                    console.warn('[atlas] SVG failed from API /api/nodes, retrying /nodes', primarySrc);
                    this.mathImage.src = hostFallback || localFallback;
                    return;
                }
                if (!triedLocal && hostFallback && this.mathImage.src === hostFallback) {
                    triedLocal = true;
                    console.warn('[atlas] SVG failed from host /nodes, retrying local relative', hostFallback);
                    this.mathImage.src = localFallback;
                    return;
                }
                console.error('[atlas] SVG failed to load', this.mathImage.src, e);
                this.mathImage.style.display = 'none';
                this.placeholder.style.display = 'block';
                this.placeholder.textContent = 'Preview failed to load. Trying local render...';
                if (this.currentNode) {
                    renderTypstFallback(this.currentNode);
                }
            };

            this.mathImage.src = primarySrc || hostFallback || localFallback;
        }

        this.panel.classList.add('open');

        // Recenter graph when opening via dependency click
        if (recenterOnNextOpen && Graph) {
            const target = nodesById.get(recenterOnNextOpen);
            if (target && typeof target.x === 'number' && typeof target.y === 'number') {
                const targetZoom = 6;
                const transitionTime = 800;
                Graph.zoom(targetZoom, transitionTime);
                const screenShiftX = window.innerWidth / 4;
                const canvasShiftX = screenShiftX / targetZoom;
                Graph.centerAt(target.x + canvasShiftX, target.y, transitionTime);
            }
            recenterOnNextOpen = null;
        }
    },

    close() {
        this.panel.classList.remove('open');
        setTimeout(() => {
            this.mathImage.src = '';
            this.currentNode = null;
        }, 300); 
    }
};

UIController.init();

let hoverNode = null;
const neighbors = new Set();
const neighborLinks = new Set();
const searchFilteredNodes = new Set();
let currentDimAlpha = 1.0; 
let targetDimAlpha = 1.0;
let animationFrameId = null;
let Graph; 
let searchInput = null;
let searchResults = null;
let currentSelectedIndex = -1;
const commandOverlay = document.getElementById('command-overlay');
const commandPanel = document.getElementById('command-panel');
let searchDebounceId = null;

// ─── Typst fallback renderer (lazy-loaded, used if SVG fetch fails) ───────────
let typstPromise = null;
let typstInstance = null;
let mathGraphTypCache = null;

async function loadTypstRuntime() {
    if (typstPromise) return typstPromise;
    typstPromise = (async () => {
        const CDN_URL = "https://esm.sh/@myriaddreamin/typst.ts@0.4.1/dist/esm/contrib/snippet.mjs?bundle=all";
        const res = await fetch(CDN_URL);
        if (!res.ok) throw new Error(`typst.ts CDN fetch failed: ${res.status}`);
        let code = await res.text();
        code = code.replace(/from\s+["'](\/[^"]+)["']/g, 'from "https://esm.sh$1"');
        code = code.replace(/import\s+["'](\/[^"]+)["']/g, 'import "https://esm.sh$1"');
        const blob = new Blob([code], { type: "application/javascript" });
        const blobURL = URL.createObjectURL(blob);
        try {
            const mod = await import(blobURL);
            typstInstance = mod.$typst;
        } finally {
            URL.revokeObjectURL(blobURL);
        }

        // wasm init
        await typstInstance.setCompilerInitOptions({
            getModule: () => fetch("https://unpkg.com/@myriaddreamin/typst-ts-web-compiler@0.4.1/pkg/typst_ts_web_compiler_bg.wasm")
                .then(r => r.arrayBuffer())
                .then(buf => new Uint8Array(buf)),
        });
        await typstInstance.setRendererInitOptions({
            getModule: () => fetch("https://unpkg.com/@myriaddreamin/typst-ts-renderer@0.4.1/pkg/typst_ts_renderer_bg.wasm")
                .then(r => r.arrayBuffer())
                .then(buf => new Uint8Array(buf)),
        });
        return typstInstance;
    })();
    return typstPromise;
}

async function renderTypstFallback(node) {
    try {
        await loadTypstRuntime();
        if (!mathGraphTypCache) {
            const res = await fetch("./math-graph.typ");
            if (res.ok) {
                mathGraphTypCache = await res.text();
            } else {
                console.error('[atlas] fallback: failed to fetch math-graph.typ', res.status);
                mathGraphTypCache = '';
            }
        }

        const body = node.body || '';
        const renderedBody = body.replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, id, text) => `#link(\"${LINK_BASE}/#${id}\")[${text || id}]`);

        const wrapped = `#set page(width: 500pt, height: auto, margin: 10pt, fill: rgb(\"#282a36\"))\n#set text(fill: rgb(\"#f8f8f2\"), size: 14pt)\n\n${mathGraphTypCache}\n\n${renderedBody}`;
        const svg = await typstInstance.svg({ mainContent: wrapped });
        UIController.mathImage.style.display = 'none';
        UIController.placeholder.style.display = 'block';
        UIController.placeholder.innerHTML = svg;
        UIController.placeholder.style.color = '#f8f8f2';
    } catch (e) {
        console.error('[atlas] fallback Typst render failed', e);
        UIController.placeholder.style.display = 'block';
        UIController.placeholder.textContent = 'Preview failed to load (fallback render error).';
    }
}

async function renderTypstPdf(node) {
    await loadTypstRuntime();
    if (!mathGraphTypCache) {
        const res = await fetch("./math-graph.typ");
        mathGraphTypCache = res.ok ? await res.text() : '';
    }
    const body = node.body || '';
    const renderedBody = body.replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, id, text) => `#link(\"${LINK_BASE}/#${id}\")[${text || id}]`);

    const wrapped = `#set page(width: 595pt, height: auto, margin: (x: 56pt, y: 48pt), fill: rgb(\"#282a36\"))\n#set text(fill: rgb(\"#f8f8f2\"), size: 12pt)\n\n${mathGraphTypCache}\n\n${renderedBody}`;
    const pdfBytes = await typstInstance.pdf({ mainContent: wrapped });
    const blob = new Blob([pdfBytes], { type: 'application/pdf' });
    return URL.createObjectURL(blob);
}

function animateOpacity() {
    const speed = 0.08; 
    currentDimAlpha += (targetDimAlpha - currentDimAlpha) * speed;

    if (Graph) {
        Graph
            .nodeColor(Graph.nodeColor())
            .linkColor(Graph.linkColor())
            .linkDirectionalArrowColor(Graph.linkDirectionalArrowColor());
    }

    if (Math.abs(currentDimAlpha - targetDimAlpha) > 0.01) {
        animationFrameId = requestAnimationFrame(animateOpacity);
    } else {
        currentDimAlpha = targetDimAlpha; 
    }
}

const API_BASE = (window.ATLAS_API_URL || '').replace(/\/$/, '');
const LINK_BASE = (window.ATLAS_LINK_BASE || API_BASE || window.location.origin).replace(/\/$/, '');
const nodesById = new Map();
let recenterOnNextOpen = null;
let commandPaletteOpen = false;

function displayNameFromId(id) {
    if (!id) return '';
    const stripped = id.replace(/^(thm|def|ax|lem)-/, '').replace(/-/g, ' ');
    return stripped.charAt(0).toUpperCase() + stripped.slice(1);
}

function processSearchQuery(query) {
    if (!searchInput || !searchResults || !Graph) return;
    searchResults.innerHTML = '';
    currentSelectedIndex = -1; 
    searchFilteredNodes.clear();

    if (!query) {
        searchResults.classList.remove('visible');
        Graph.nodeColor(Graph.nodeColor());
        return;
    }

    const currentNodes = Graph.graphData().nodes;

    // --- COMMAND PARSER ---
    if (query.startsWith('type:')) {
        const typeQuery = query.split(':')[1].trim();
        if (typeQuery) {
            currentNodes.forEach(n => {
                if (n.type.toLowerCase().includes(typeQuery)) searchFilteredNodes.add(n.id);
            });
            Graph.nodeColor(Graph.nodeColor());
        }
        searchResults.classList.remove('visible');
        return; 
    }

    if (query.startsWith('tag:')) {
        const tagQuery = query.split(':')[1].trim();
        if (tagQuery) {
            currentNodes.forEach(n => {
                if (n.tags && n.tags.some(t => t.toLowerCase().includes(tagQuery))) {
                    searchFilteredNodes.add(n.id);
                }
            });
            Graph.nodeColor(Graph.nodeColor());
        }
        searchResults.classList.remove('visible');
        return;
    }

    if (query.startsWith('deps:')) {
        const targetQuery = query.split(':')[1].trim();
        if (targetQuery) {
            const startNode = currentNodes.find(n => n.id.toLowerCase().includes(targetQuery));
            
            if (startNode) {
                const queue = [startNode.id];
                
                while (queue.length > 0) {
                    const currentId = queue.shift();
                    
                    if (!searchFilteredNodes.has(currentId)) {
                        searchFilteredNodes.add(currentId);
                        
                        const nodeObj = currentNodes.find(n => n.id === currentId);
                        if (nodeObj && nodeObj.deps) {
                            nodeObj.deps.forEach(dep => queue.push(dep));
                        }
                    }
                }
            }
            Graph.nodeColor(Graph.nodeColor());
        }
        searchResults.classList.remove('visible');
        return; 
    }

    // --- STANDARD SEARCH (Dropdown) ---
    const matches = currentNodes.filter(n => {
        const cleanName = n.id.replace(/^(thm|def|ax|lem)-/, '').replace(/-/g, ' ');
        return n.id.toLowerCase().includes(query) || cleanName.toLowerCase().includes(query);
    }).slice(0, 8); 

    if (matches.length > 0) {
        searchResults.classList.add('visible');
        
        matches.forEach((node, index) => {
            const li = document.createElement('li');
            const cleanName = node.id.replace(/^(thm|def|ax|lem)-/, '').replace(/-/g, ' ');
            const typeClass = typeColors[node.type] ? node.type : 'default';
            
            li.innerHTML = `
                <span>${cleanName.charAt(0).toUpperCase() + cleanName.slice(1)}</span>
                <span class="search-match-type match-type-${typeClass}">${node.type}</span>
            `;
            
            li.addEventListener('click', () => {
                const targetZoom = 6;
                const transitionTime = 800;
                Graph.zoom(targetZoom, transitionTime);
                
                const screenShiftX = window.innerWidth / 4; 
                const canvasShiftX = screenShiftX / targetZoom;
                Graph.centerAt(node.x + canvasShiftX, node.y, transitionTime);
                
                UIController.open(node);

                closeCommandPalette();
                
                searchInput.value = '';
                searchResults.classList.remove('visible');
                currentSelectedIndex = -1;
                
                // Clear filters if someone clicked a result
                searchFilteredNodes.clear();
                Graph.nodeColor(Graph.nodeColor());
            });
            
            searchResults.appendChild(li);
        });
    } else {
        searchResults.classList.remove('visible');
    }
}

function openCommandPalette() {
    if (!commandOverlay) return;
    commandPaletteOpen = true;
    commandOverlay.classList.add('open');
    if (searchInput) {
        searchInput.value = '';
        searchInput.focus({ preventScroll: true });
    }
    if (searchResults) {
        searchResults.innerHTML = '';
        searchResults.classList.remove('visible');
    }
    currentSelectedIndex = -1;
    searchFilteredNodes.clear();
    if (Graph) Graph.nodeColor(Graph.nodeColor());
}

function closeCommandPalette() {
    if (!commandOverlay) return;
    commandPaletteOpen = false;
    commandOverlay.classList.remove('open');
    if (searchResults) searchResults.classList.remove('visible');
    currentSelectedIndex = -1;
}

async function fetchGraphJson() {
    const endpoints = [];
    if (API_BASE) {
        endpoints.push(`${API_BASE}/api/graph`);
        endpoints.push(`${API_BASE}/api/json/graph.json`);
    }
    endpoints.push('./json/graph.json');

    let lastErr;
    for (const url of endpoints) {
        try {
            const resp = await fetch(url, { mode: 'cors' });
            if (!resp.ok) {
                console.error('[atlas] graph.json fetch failed', url, resp.status, resp.statusText);
                lastErr = new Error(`fetch failed ${resp.status}`);
                continue;
            }
            const data = await resp.json();
            if (!Array.isArray(data)) {
                console.error('[atlas] graph.json unexpected shape', url, data);
            }
            return data;
        } catch (e) {
            console.error('[atlas] graph.json fetch error', url, e);
            lastErr = e;
        }
    }
    throw lastErr || new Error('graph.json fetch failed');
}

async function initGraph() {
    try {
        const data = await fetchGraphJson();

        const nodes = data.map(node => ({ 
            id: node.id, 
            type: node.node_type,
            body: node.body,
            tags: node.tags || [],
            deps: Array.isArray(node.deps) ? node.deps : [],
            isGhost: false
        }));

        const validNodeIds = new Set(nodes.map(n => n.id));

        const links = [];
        data.forEach(node => {
            node.deps.forEach(dependency => {
                
                if (!validNodeIds.has(dependency)) {
                    nodes.push({ 
                        id: dependency, 
                        type: "ghost", 
                        isGhost: true,
                        deps: []
                    });
                    validNodeIds.add(dependency);
                }
                
                links.push({ source: node.id, target: dependency });
            });
        });

        nodes.forEach(n => nodesById.set(n.id, n));

        const elem = document.getElementById('graph-container');

        Graph = ForceGraph()(elem)
            .graphData({ nodes, links })
            .nodeId('id')
            .nodeLabel('id')
            .backgroundColor('#282a36')
            .linkDirectionalArrowLength(5)
            .linkDirectionalArrowRelPos(1)
            .linkCurvature(0.1);

        Graph
            .nodeColor(node => {
                // 1. Command Palette Isolation Override
                if (searchFilteredNodes.size > 0) {
                    return searchFilteredNodes.has(node.id) 
                        ? (typeColors[node.type] || "#6272a4") 
                        : 'rgba(68, 71, 90, 0.1)';
                }
                
                // 2. Standard Hover Logic
                if (!hoverNode) return typeColors[node.type] || "#6272a4";
                return neighbors.has(node.id) 
                    ? typeColors[node.type] 
                    : `rgba(98, 114, 164, ${currentDimAlpha})`; 
            })
            .linkColor(link => {
                if (searchFilteredNodes.size > 0) return 'rgba(68, 71, 90, 0.05)'; 
                
                if (!hoverNode) return link.target.isGhost ? 'rgba(68, 71, 90, 0.4)' : '#6272a4';
                return neighborLinks.has(link) 
                    ? '#f8f8f2' 
                    : `rgba(98, 114, 164, ${currentDimAlpha * 0.5})`; 
            })
            .linkDirectionalArrowColor(link => {
                // Must mirror the linkColor logic so arrows dim too
                if (searchFilteredNodes.size > 0) return 'rgba(68, 71, 90, 0.05)';
                
                if (!hoverNode) return link.target.isGhost ? 'rgba(68, 71, 90, 0.4)' : '#6272a4';
                return neighborLinks.has(link) ? '#f8f8f2' : `rgba(98, 114, 164, ${currentDimAlpha * 0.5})`;
            })
            .onNodeHover(node => {
                neighbors.clear();
                neighborLinks.clear();
                hoverNode = node;

                if (node) {
                    neighbors.add(node.id);
                    Graph.graphData().links.forEach(link => {
                        if (link.source.id === node.id || link.target.id === node.id) {
                            neighborLinks.add(link);
                            neighbors.add(link.source.id);
                            neighbors.add(link.target.id);
                        }
                    });
                    targetDimAlpha = 0.05; 
                } else {
                    targetDimAlpha = 1.0; 
                }

                elem.style.cursor = node ? 'pointer' : null;
                if (animationFrameId) cancelAnimationFrame(animationFrameId);
                animateOpacity();
            })
	        .onNodeClick(node => {
                const targetZoom = 6;
                const transitionTime = 800;

                Graph.zoom(targetZoom, transitionTime);
                
                const screenShiftX = window.innerWidth / 4; 
                const canvasShiftX = screenShiftX / targetZoom;

                Graph.centerAt(node.x + canvasShiftX, node.y, transitionTime);
                
                UIController.open(node);
            });

        // --- Search Engine Logic ---
    searchInput = document.getElementById('search-input');
    searchResults = document.getElementById('search-results');
        
    if (searchInput && searchResults) {
            searchInput.addEventListener('input', (e) => {
                const query = e.target.value.toLowerCase().trim();
                if (searchDebounceId) clearTimeout(searchDebounceId);
                searchDebounceId = setTimeout(() => {
                    processSearchQuery(query);
                }, 80);
            });

            // Handle Keyboard Navigation
            searchInput.addEventListener('keydown', (e) => {
                const listItems = searchResults.querySelectorAll('li');
                if (listItems.length === 0 || !searchResults.classList.contains('visible')) return;

                if (e.key === 'ArrowDown' || e.key === 'Tab') {
                    e.preventDefault(); 
                    currentSelectedIndex++;
                    if (currentSelectedIndex >= listItems.length) currentSelectedIndex = 0; 
                    updateSelection(listItems);
                } else if (e.key === 'ArrowUp') {
                    e.preventDefault();
                    currentSelectedIndex--;
                    if (currentSelectedIndex < 0) currentSelectedIndex = listItems.length - 1; 
                    updateSelection(listItems);
                } else if (e.key === 'Enter') {
                    e.preventDefault();
                    if (currentSelectedIndex >= 0 && currentSelectedIndex < listItems.length) {
                        listItems[currentSelectedIndex].click(); 
                    } else if (listItems.length > 0) {
                        listItems[0].click(); 
                    }
                }
            });

            // Helper to apply the visual highlight class
            function updateSelection(listItems) {
                listItems.forEach((li, index) => {
                    if (index === currentSelectedIndex) {
                        li.classList.add('selected');
                        li.scrollIntoView({ block: 'nearest' }); 
                    } else {
                        li.classList.remove('selected');
                    }
                });
            }

            // Hide dropdown if clicked outside
            document.addEventListener('click', (e) => {
                if (commandOverlay && commandOverlay.classList.contains('open') && e.target === commandOverlay) {
                    closeCommandPalette();
                    return;
                }
                if (!searchInput.contains(e.target) && !searchResults.contains(e.target)) {
                    searchResults.classList.remove('visible');
                    currentSelectedIndex = -1;
                }
            });
        }

    } catch (error) {
        console.error("Failed to load or parse graph.json in atlas:", error);
    }
}

setupAuth('btn-login-github', 'auth-status');
initGraph();

document.addEventListener('keydown', (e) => {
    if (!commandPaletteOpen && e.key === ':' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        openCommandPalette();
    } else if (commandPaletteOpen && e.key === 'Escape') {
        e.preventDefault();
        closeCommandPalette();
    }
});
