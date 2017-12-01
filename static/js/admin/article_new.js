"use strict";
$("#submit").click(function (event) {
    event.preventDefault();
    var title = $("#title").val();
    var content = $("#editor").val();
    // 获取选中的已有 tag 的 id
    var exist_tags = $("ul.tag li a[data-id].a_click").map(function () { return $(this).attr("data-id"); }).toArray();
    // 获取新增 tag 的文字信息
    var new_tags = $("ul.tag li a:not([data-id]).a_click").map(function () { return $(this).html(); }).toArray();
    if (title === "" || content === "") {
        $("#mistake").modal("show")
    } else {
        $.ajax({
            url: "/api/v1/article/new",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "title": title, "raw_content": content, "exist_tags": exist_tags, "new_tags": new_tags }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    $("#success").modal("show")
                }
            }
        })
    }
});