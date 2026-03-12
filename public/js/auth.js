// Shared auth logic for GitHub login
// ATLAS_API_URL is set by config.js (loaded before this script)
(function () {

    window.getAuthToken = function () {
        return localStorage.getItem('github_token');
    };

    window.setupAuth = function (btnLoginId, statusContainerId) {
        const btnLogin = document.getElementById(btnLoginId);
        const authStatus = document.getElementById(statusContainerId);

        if (!btnLogin || !authStatus) return;

        async function renderStatus() {
            const token = localStorage.getItem('github_token');
            if (token) {
                try {
                    // Attempt to fetch profile info 
                    const res = await fetch((window.ATLAS_API_URL || 'http://127.0.0.1:3000') + '/api/auth/profile', {
                        headers: { 
                            'Authorization': 'Bearer ' + token
                        }
                    });
                    
                    if (res.ok) {
                        const profile = await res.json();
                        authStatus.innerHTML = '';
                        btnLogin.innerHTML = `<img src="${profile.avatar_url}" class="nav-avatar" alt="Avatar"><span>${profile.username}</span>`;
                        btnLogin.classList.add('btn-profile');
                        btnLogin.title = 'View Profile';
                        
                        btnLogin.onclick = () => {
                            window.location.href = 'profile.html';
                        };
                    } else {
                        localStorage.removeItem('github_token');
                        renderLoggedOut();
                    }
                } catch (e) {
                    console.error('Failed to load profile', e);
                    renderLoggedOut();
                }
            } else {
                renderLoggedOut();
            }
        }

        function renderLoggedOut() {
            authStatus.innerHTML = '';
            btnLogin.innerHTML = 'Login with GitHub';
            btnLogin.classList.remove('btn-profile');
            btnLogin.title = '';
            btnLogin.onclick = handleLoginClick;
        }

        function handleLoginClick() {
            const width = 600;
            const height = 700;
            const left = (window.screen.width / 2) - (width / 2);
            const top = (window.screen.height / 2) - (height / 2);

            window.open(
                (window.ATLAS_API_URL || 'http://127.0.0.1:3000') + '/api/auth/github',
                'GitHub Login',
                `width=${width},height=${height},top=${top},left=${left}`
            );
        }

        // Removed the global btn-logout since logout is handled exclusively on the profile page
        renderStatus();

        window.addEventListener('message', (event) => {
            if (event.data && event.data.type === 'github-auth') {
                localStorage.setItem('github_token', event.data.token);
                renderStatus();
            } else if (event.data && event.data.type === 'github-auth-error') {
                alert('GitHub Authentication failed: ' + event.data.error);
            }
        });
    };
})();
