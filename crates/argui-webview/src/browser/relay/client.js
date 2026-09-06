import { config } from './config.js';

const protocol = 'argui-webview-v1';
const flags = new Set(['allow-scripts', 'allow-forms', 'allow-same-origin',
    'allow-popups', 'allow-popups-to-escape-sandbox', 'allow-downloads']);
let frame = null, sandbox = null;
window.addEventListener('message', event => {
    if (event.source !== parent || event.origin !== config.appOrigin || location.origin !== config.relayOrigin) return;
    const data = event.data;
    if (!data || data.protocol !== protocol || data.type !== 'load' || !Number.isSafeInteger(data.id)) return;
    const reply = (type, value = '') => parent.postMessage({ protocol, type, id: data.id, value }, config.appOrigin);
    try {
        const url = new URL(data.url);
        if (!config.allowedOrigins.includes(url.origin) || url.username || url.password) throw new Error('Website origin is not allowed by the relay');
        if (typeof data.sandbox !== 'string') throw new Error('Invalid sandbox');
        const tokens = data.sandbox.split(' ');
        if (tokens.some(token => !flags.has(token)) || !['allow-scripts', 'allow-forms', 'allow-same-origin'].every(token => tokens.includes(token))) throw new Error('Invalid sandbox');
        if (sandbox !== null && sandbox !== data.sandbox) throw new Error('Session permissions are immutable');
        sandbox = data.sandbox;
        if (!frame) {
            frame = document.createElement('iframe');
            frame.title = 'Webpage';
            frame.setAttribute('sandbox', sandbox);
            frame.referrerPolicy = 'no-referrer';
            frame.setAttribute('allow', "camera 'none'; microphone 'none'; geolocation 'none'; payment 'none'; clipboard-read 'none'; clipboard-write 'none'; fullscreen 'none'");
            document.body.append(frame);
        }
        frame.src = url.href;
        // Acknowledges configuration only, not successful remote rendering.
        reply('accepted');
    } catch (error) { reply('error', error.message); }
});
