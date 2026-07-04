function processHxRequest(element, nodeData, triggerSpecs) {
    let foundRequest = false;

    // w = ["get", "post", "put", "delete", "patch", ...]
    each(HTTP_METHODS, function (method) {

        // 是否存在 hx-get / hx-post ...
        if (hasAttribute(element, "hx-" + method)) {

            // 例如 "/f/w7sfvwatp70x/go"
            const path = getAttributeValue(element, "hx-" + method);

            foundRequest = true;

            nodeData.path = path;
            nodeData.verb = method;

            // 一个元素可能绑定多个 trigger
            triggerSpecs.forEach(function (triggerSpec) {

                // 注册事件
                addTriggerHandler(
                    element,
                    triggerSpec,
                    nodeData,

                    function (triggerElement, event) {

                        // 禁用元素则忽略
                        if (matches(triggerElement, Q.config.disableSelector)) {
                            ignoreElement(triggerElement);
                        } else {

                            // 发起 AJAX 请求
                            issueRequest(
                                method,
                                path,
                                triggerElement,
                                event
                            );
                        }
                    }
                );
            });
        }
    });

    return foundRequest;
}