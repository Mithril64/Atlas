const typeColors = {
    "axiom": "#ff5555",
    "definition": "#ffb86c",
    "lemma": "#8be9fd",
    "theorem": "#bd93f9"
};

let hoverNode = null;
const neighbors = new Set();
const neighborLinks = new Set();

async function initGraph() {
    try {
        const response = await fetch('./graph.json');
        const data = await response.json();

        const nodes = data.map(node => ({ id: node.id, type: node.node_type }));
        const links = [];
        data.forEach(node => {
            node.deps.forEach(dependency => {
                links.push({ source: node.id, target: dependency });
            });
        });

        const elem = document.getElementById('graph-container');
        const Graph = ForceGraph()(elem)
            .graphData({ nodes, links })
            .nodeId('id')
            .nodeLabel('id')
            .linkDirectionalArrowLength(5)
            .linkDirectionalArrowRelPos(1)
            .linkCurvature(0.1)
            .backgroundColor('#282a36')
            
            .onNodeHover(node => {
                neighbors.clear();
                neighborLinks.clear();
                hoverNode = node;

                if (node) {
                    neighbors.add(node.id);
                    links.forEach(link => {
                        if (link.source.id === node.id || link.target.id === node.id) {
                            neighborLinks.add(link);
                            neighbors.add(link.source.id);
                            neighbors.add(link.target.id);
                        }
                    });
                }
                
                elem.style.cursor = node ? 'pointer' : null;
            })
            
            .nodeCanvasObjectMode(() => 'after')
            .nodeColor(node => {
                if (!hoverNode) return typeColors[node.type] || "#6272a4";
                return neighbors.has(node.id) 
                    ? typeColors[node.type] 
                    : 'rgba(98, 114, 164, 0.2)';
            })
            
            .linkColor(link => {
                if (!hoverNode) return '#6272a4';
                return neighborLinks.has(link) 
                    ? '#f8f8f2'
                    : 'rgba(98, 114, 164, 0.1)';
            })
            .linkWidth(link => neighborLinks.has(link) ? 2 : 1)
            .linkDirectionalArrowColor(link => neighborLinks.has(link) ? '#f8f8f2' : 'rgba(98, 114, 164, 0.1)');

    } catch (error) {
        console.error("Failed to load or parse graph.json:", error);
    }
}

initGraph();
