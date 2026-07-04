function parseHtml(html, depth) {
    // 解析 HTML 字符串
    let node = new DOMParser()
        .parseFromString(html, "text/html")
        .body;

    // 沿着 firstChild 向下走 depth 层
    while (depth > 0) {
        depth--;
        node = node.firstChild;
    }

    // 如果不存在对应节点，则返回空 DocumentFragment
    if (node == null) {
        node = document.createDocumentFragment();
    }

    return node;
}