// One retained DOM surface per resident session; no animation loop or document polling.
export function createView(canvas, email, callback, sandbox, relay) {
    if (email && (sandbox !== 'allow-same-origin' || relay)) throw new Error('Email permissions cannot be relaxed');
    if (!email && sandbox.includes('allow-same-origin') !== Boolean(relay)) throw new Error('Origin-preserving webpages require a relay');
    const frame = document.createElement('iframe');
    frame.dataset.arguiWebview = '';
    frame.title = email ? 'Email content' : 'Webpage';
    // Email allows parent-side DOM link handling, but NEVER permits document scripts.
    // Webpages can execute scripts, but NEVER share the application's origin.
    frame.setAttribute('sandbox', sandbox);
    frame.referrerPolicy = 'no-referrer';
    frame.setAttribute('allow', "camera 'none'; microphone 'none'; geolocation 'none'; payment 'none'; clipboard-read 'none'; clipboard-write 'none'; fullscreen 'none'");
    frame.style.cssText = 'position:fixed;border:0;margin:0;display:none;background:Canvas;z-index:1;transform-origin:top left;box-sizing:border-box';
    const bridge = relay ? relayBridge(frame, relay, sandbox, callback) : null;
    let rect = [0, 0, 0, 0], shown = false, ready = false, source = '', disposed = false;
    let documentEvents = null;
    let clipping = null;
    const events = new AbortController();
    function position() {
        if (disposed) return;
        const box = canvas.getBoundingClientRect();
        const [x, y, w, h] = rect;
        const scale = window.devicePixelRatio || 1;
        const sx = canvas.width ? box.width * scale / canvas.width : 1;
        const sy = canvas.height ? box.height * scale / canvas.height : 1;
        const clip = clipping ?? [x, y, w, h];
        const left = Math.max(0, -x, clip[0] - x);
        const top = Math.max(0, -y, clip[1] - y);
        const right = Math.max(0, x + w - canvas.width / scale, x + w - clip[0] - clip[2]);
        const bottom = Math.max(0, y + h - canvas.height / scale, y + h - clip[1] - clip[3]);
        Object.assign(frame.style, {
            left: `${box.left + x * sx}px`, top: `${box.top + y * sy}px`,
            width: `${w}px`, height: `${h}px`, transform: `scale(${sx}, ${sy})`,
            display: shown && canvas.isConnected && w > 0 && h > 0 ? 'block' : 'none',
            pointerEvents: email && !ready ? 'none' : 'auto',
            clipPath: `inset(${top}px ${right}px ${bottom}px ${left}px)`,
        });
    }
    frame.addEventListener('load', () => {
        if (disposed || !source) return;
        documentEvents?.abort();
        documentEvents = new AbortController();
        if (email) {
            const doc = frame.contentDocument;
            if (!doc || !doc.querySelector('meta[http-equiv="Content-Security-Policy"]')) return;
            const navigate = event => {
                const link = event.target.closest?.('a[href]');
                if (!link) return;
                event.preventDefault();
                const url = link.getAttribute('href');
                if (/^https?:\/\//i.test(url)) callback('navigation', url);
            };
            for (const name of ['click', 'auxclick']) doc.addEventListener(name, navigate, { capture: true, signal: documentEvents.signal });
            doc.addEventListener('contextmenu', event => event.preventDefault(), { signal: documentEvents.signal });
        }
        ready = true;
        position();
        // An iframe load does not prove that a remote server permitted embedding.
        if (email) callback('loaded', 'about:srcdoc');
    }, { signal: events.signal });
    window.addEventListener('scroll', position, { capture: true, passive: true, signal: events.signal });
    window.addEventListener('resize', position, { passive: true, signal: events.signal });
    const observer = new ResizeObserver(position);
    observer.observe(canvas);
    document.body.append(frame);
    return {
        load(value) {
            source = value; ready = false;
            documentEvents?.abort();
            callback('loading', email ? 'about:srcdoc' : value);
            if (email) frame.srcdoc = value;
            else if (bridge) bridge.load(value);
            else frame.src = value;
            position();
        },
        bounds(x, y, w, h) { rect = [x, y, w, h]; position(); },
        clip(x, y, w, h) {
            if (clipping?.every((v, i) => v === [x, y, w, h][i])) return;
            clipping = [x, y, w, h]; position();
        },
        visible(value) { shown = value; position(); },
        focus() { if (shown) frame.focus(); },
        dispose() {
            if (disposed) return;
            disposed = true;
            bridge?.dispose();
            events.abort(); documentEvents?.abort(); observer.disconnect(); frame.remove();
        },
    };
}

// The relay URL is trusted deployment configuration, never a website-supplied URL.
function relayBridge(frame, relay, sandbox, callback) {
    const url = new URL(relay);
    if (!['http:', 'https:'].includes(url.protocol) || url.origin === location.origin || url.username || url.password || url.search || url.hash) {
        throw new Error('Compatible mode requires a trusted relay on a distinct HTTP(S) origin');
    }
    const protocol = 'argui-webview-v1';
    const events = new AbortController();
    let pending = null, ready = false, sequence = 0, timer = null;
    function send() {
        if (ready && pending) frame.contentWindow.postMessage(pending, url.origin);
    }
    frame.addEventListener('load', () => { ready = true; send(); }, { signal: events.signal });
    window.addEventListener('message', event => {
        if (event.source !== frame.contentWindow || event.origin !== url.origin) return;
        const data = event.data;
        if (!pending || !data || data.protocol !== protocol || data.id !== pending.id) return;
        if (data.type !== 'accepted' && data.type !== 'error') return;
        clearTimeout(timer);
        pending = null;
        if (data.type === 'error') callback('error', String(data.value));
    }, { signal: events.signal });
    frame.src = url.href;
    return {
        load(value) {
            pending = { protocol, type: 'load', id: ++sequence, url: value, sandbox };
            clearTimeout(timer);
            timer = setTimeout(() => {
                pending = null;
                callback('error', 'Relay did not acknowledge navigation; check its origin, CSP and configuration');
            }, 10000);
            send();
        },
        dispose() { clearTimeout(timer); pending = null; events.abort(); },
    };
}

export function createWakeTimer(callback) {
    let timer = null;
    return {
        schedule(milliseconds) {
            clearTimeout(timer);
            timer = milliseconds < 0 ? null : setTimeout(callback, Math.min(2147483647, Math.ceil(milliseconds)));
        },
        dispose() { clearTimeout(timer); timer = null; },
    };
}
