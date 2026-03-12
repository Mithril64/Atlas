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
                const link = document.createElement('a');
                link.href = `./nodes/${this.currentNode.id}.pdf`;
                link.download = `${this.currentNode.id}.pdf`;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
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
        
        const cleanName = node.id.replace(/^(thm|def|ax|lem)-/, '').replace(/-/g, ' ');
        this.title.textContent = cleanName.charAt(0).toUpperCase() + cleanName.slice(1);

        this.tabBtns.forEach(b => b.classList.remove('active'));
        this.tabPanes.forEach(p => p.classList.remove('active'));
        this.tabBtns[0].classList.add('active');
        this.tabPanes[0].classList.add('active');

        this.rawCodeDisplay.textContent = node.body || "// No source code available.";

        this.depsListDisplay.innerHTML = '';
        if (node.deps && node.deps.length > 0) {
            node.deps.forEach(dep => {
                const li = document.createElement('li');
                li.textContent = dep;
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
            this.mathImage.src = `./nodes/${node.id}.svg?t=${cacheBuster}`; 
        }

        this.panel.classList.add('open');
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

async function initGraph() {
    try {
        const response = await fetch('./json/graph.json');
        const data = await response.json();

        const nodes = data.map(node => ({ 
            id: node.id, 
            type: node.node_type,
            body: node.body,
            tags: node.tags || []
        }));

        const validNodeIds = new Set(nodes.map(n => n.id));

        const links = [];
        data.forEach(node => {
            node.deps.forEach(dependency => {
                
                if (!validNodeIds.has(dependency)) {
                    nodes.push({ 
                        id: dependency, 
                        type: "ghost", 
                        isGhost: true 
                    });
                    validNodeIds.add(dependency);
                }
                
                links.push({ source: node.id, target: dependency });
            });
        });

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
        const searchInput = document.getElementById('search-input');
        const searchResults = document.getElementById('search-results');
        
        let currentSelectedIndex = -1; 

        if (searchInput && searchResults) {
            searchInput.addEventListener('input', (e) => {
                const query = e.target.value.toLowerCase().trim();
                searchResults.innerHTML = '';
                currentSelectedIndex = -1; 
                searchFilteredNodes.clear(); // Reset visual filters on every keystroke

                if (!query) {
                    searchResults.classList.remove('visible');
                    Graph.nodeColor(Graph.nodeColor()); // Force re-render to restore colors
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
                        Graph.nodeColor(Graph.nodeColor()); // Force WebGL update
                    }
                    searchResults.classList.remove('visible'); // Hide dropdown
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
                        Graph.nodeColor(Graph.nodeColor()); // Force WebGL update
                    }
                    searchResults.classList.remove('visible'); // Hide dropdown
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
                        Graph.nodeColor(Graph.nodeColor()); // Force WebGL update
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
