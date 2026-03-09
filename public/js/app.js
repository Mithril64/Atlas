const typeColors = {
    "axiom": "#ff5555",
    "definition": "#ffb86c",
    "lemma": "#8be9fd",
    "theorem": "#bd93f9",
    "ghost": "rgba(68, 71, 90, 0.6)"
};

// --- 1. UI Controller ---
const UIController = {
    panel: document.getElementById('side-panel'),
    closeBtn: document.getElementById('close-panel-btn'),
    badge: document.getElementById('node-badge'),
    title: document.getElementById('node-title'),
    idDisplay: document.getElementById('node-id'),

    init() {
        this.closeBtn.addEventListener('click', () => this.close());
    },

    open(node) {
        this.badge.textContent = node.type;
        this.badge.style.backgroundColor = typeColors[node.type] || '#6272a4';
        this.idDisplay.textContent = node.id;
        
        const cleanName = node.id.replace(/^(thm|def|ax|lem)-/, '').replace(/-/g, ' ');
        this.title.textContent = cleanName.charAt(0).toUpperCase() + cleanName.slice(1);

        this.panel.classList.add('open');
    },

    close() {
        this.panel.classList.remove('open');
    }
};

UIController.init();

let hoverNode = null;
const neighbors = new Set();
const neighborLinks = new Set();
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

        const nodes = data.map(node => ({ id: node.id, type: node.node_type }));
        
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
                if (!hoverNode) return typeColors[node.type] || "#6272a4";
                return neighbors.has(node.id) 
                    ? typeColors[node.type] 
                    : `rgba(98, 114, 164, ${currentDimAlpha})`; 
            })
            .linkColor(link => {
                if (!hoverNode) {
                    return link.target.isGhost ? 'rgba(68, 71, 90, 0.4)' : '#6272a4';
                }
                return neighborLinks.has(link) 
                    ? '#f8f8f2' 
                    : `rgba(98, 114, 164, ${currentDimAlpha * 0.5})`; 
            })
            .linkDirectionalArrowColor(link => {
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
            
            // Click Events
	    .onNodeClick(node => {
                const targetZoom = 6;
                const transitionTime = 800;

                Graph.zoom(targetZoom, transitionTime);
                
                // 2. Calculate the camera shift
                // We want the node to sit in the middle of the left half of the screen.
                // That means shifting the camera's target 25% of the screen width to the right.
                // We divide by targetZoom so the shift matches the canvas scale.
                const screenShiftX = window.innerWidth / 4; 
                const canvasShiftX = screenShiftX / targetZoom;

                Graph.centerAt(node.x + canvasShiftX, node.y, transitionTime);
                
                UIController.open(node);
            })

    } catch (error) {
        console.error("Failed to load or parse graph.json in Atlas:", error);
    }
}

initGraph();
