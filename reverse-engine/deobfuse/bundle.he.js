function issueAjaxRequest(
    method,             // "post"
    url,                // "/f/w7sfvwatp70x/go"
    element,            // 触发请求的 DOM 元素
    triggerEvent,       // click
    options = {},
    confirmed
) {
    let resolve = null;
    let reject = null;

    let promise;
    if (options.returnPromise && typeof Promise !== "undefined") {
        promise = new Promise((res, rej) => {
            resolve = res;
            reject = rej;
        });
    }

    // 默认 target
    const target = options.targetOverride || getTarget(element);
    if (!target) {
        reject?.();
        return promise;
    }

    // 元素内部状态
    const internal = getInternalData(element);

    // hx-confirm
    const confirmText = getAttributeValue(element, "hx-confirm");
    if (!confirmed && confirmText && !confirm(confirmText)) {
        resolve?.();
        return promise;
    }

    // 创建 XMLHttpRequest
    const xhr = new XMLHttpRequest();

    internal.xhr = xhr;
    internal.abortable = false;

    // 请求结束后的清理
    function cleanup() {
        internal.xhr = null;
        internal.abortable = false;

        if (internal.queuedRequests?.length) {
            internal.queuedRequests.shift()();
        }
    }

    // 默认请求头
    let headers = getHeaders(element, target);

    // POST 默认 Content-Type
    if (method !== "get") {
        headers["Content-Type"] =
            "application/x-www-form-urlencoded";
    }

    // 收集表单数据
    const formData = getInputValues(element);

    // 当前运行结果：
    // formData == {}

    const parameters = filterValues(formData);

    // GET 时拼接 QueryString
    let finalUrl = url;

    if (method === "get" && Object.keys(parameters).length) {
        finalUrl += "?" + encodeParams(parameters);
    }

    xhr.open(method.toUpperCase(), finalUrl, true);

    xhr.overrideMimeType("text/html");

    // 设置 Header
    for (const key in headers) {
        xhr.setRequestHeader(key, headers[key]);
    }

    const requestInfo = {
        xhr,
        target,
        requestConfig: {
            method,
            url,
            parameters,
            headers
        }
    };

    // 请求成功
    xhr.onload = () => {
        defaultResponseHandler(element, requestInfo);
        resolve?.();
        cleanup();
    };

    // 请求失败
    xhr.onerror = () => {
        reject?.();
        cleanup();
    };

    xhr.onabort = () => {
        reject?.();
        cleanup();
    };

    xhr.ontimeout = () => {
        reject?.();
        cleanup();
    };

    // 当前运行结果：
    // parameters == {}
    // body == ""

    const body =
        method === "get"
            ? null
            : encodeForm(parameters);

    xhr.send(body);

    return promise;
}