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

        function renderStatus() {
            const token = localStorage.getItem('github_token');
            if (token) {
                authStatus.innerHTML = '<span style="color: #50fa7b;">✓ GitHub</span>';
                btnLogin.textContent = 'Log Out';
                btnLogin.title = 'Logged in via GitHub — click to log out';
            } else {
                authStatus.innerHTML = '';
                btnLogin.textContent = 'Login with GitHub';
                btnLogin.title = '';
            }
        }

        renderStatus();

        btnLogin.addEventListener('click', () => {
            if (localStorage.getItem('github_token')) {
                localStorage.removeItem('github_token');
                renderStatus();
                return;
            }

            const width = 600;
            const height = 700;
            const left = (window.screen.width / 2) - (width / 2);
            const top = (window.screen.height / 2) - (height / 2);

            window.open(
                (window.ATLAS_API_URL || 'http://127.0.0.1:3000') + '/api/auth/github',
                'GitHub Login',
                `width=${width},height=${height},top=${top},left=${left}`
            );
        });

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
