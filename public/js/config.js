// public/js/config.js

if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    window.ATLAS_API_URL = 'http://127.0.0.1:3000';
} else {
    window.ATLAS_API_URL = 'https://alva-keyed-unexplainably.ngrok-free.dev';
}
