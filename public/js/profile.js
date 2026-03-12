document.addEventListener('DOMContentLoaded', async () => {
    const token = localStorage.getItem('github_token');
    
    if (!token) {
        window.location.href = 'index.html';
        return;
    }

    try {
        const res = await fetch((window.ATLAS_API_URL || 'http://127.0.0.1:3000') + '/api/auth/profile', {
            headers: { 
                'Authorization': 'Bearer ' + token
            }
        });
        
        if (res.ok) {
            const profile = await res.json();
            document.getElementById('profile-page-avatar').src = profile.avatar_url;
            document.getElementById('profile-page-username').textContent = profile.username;
            document.getElementById('profile-page-commits').textContent = profile.commits;
            document.getElementById('profile-page-reviews').textContent = profile.reviews;
            document.getElementById('profile-page-trust').textContent = profile.trust_rating;
            renderContributionGraph(profile.contribution_days || []);
            
        } else {
            localStorage.removeItem('github_token');
            window.location.href = 'index.html';
        }
    } catch (e) {
        console.error('Failed to load profile details on profile.html', e);
        localStorage.removeItem('github_token');
        window.location.href = 'index.html';
    }

    document.getElementById('profile-page-logout').addEventListener('click', () => {
        localStorage.removeItem('github_token');
        window.location.href = 'index.html';
    });
});

function renderContributionGraph(days) {
    const graph = document.getElementById('contrib-graph');
    if (!graph) return;

    const map = new Map(days.map(d => [d.date, d.count]));
    const today = new Date();
    const cells = [];

    for (let i = 364; i >= 0; i--) {
        const dt = new Date(today);
        dt.setDate(today.getDate() - i);
        const key = dt.toISOString().slice(0, 10);
        const count = map.get(key) || 0;
        const level = count === 0 ? 0 : count < 2 ? 1 : count < 4 ? 2 : count < 7 ? 3 : 4;
        cells.push({ key, count, level });
    }

    graph.innerHTML = '';
    for (const cell of cells) {
        const div = document.createElement('div');
        div.className = `contrib-cell level-${cell.level}`;
        div.title = `${cell.count} contribution${cell.count === 1 ? '' : 's'} on ${cell.key}`;
        graph.appendChild(div);
    }
}
