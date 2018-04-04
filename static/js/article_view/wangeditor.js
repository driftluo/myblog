"use strict";
var editor = new wangEditor('#editor');
editor.create();
$(".w-e-text-container").css({ "height": "200px", "z-index": "" });
$("button.comment").click(function () {
    if (editor.txt.text() !== "") {
        var comment = editor.txt.html();
        var article_id = $(".col-md-offset-1[data-id]").attr("data-id");
        if ($(".w-e-text .post-meta a").length > 0) {
            var reply_user_id = $(".w-e-text .post-meta a").attr("href").split("/")[2];
        }
        $.ajax({
            url: "/api/v1/comment/new",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "comment": comment, "article_id": article_id, "reply_user_id": reply_user_id }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    openInfo("提交成功", "success");
                    editor.txt.clear();
                    $("ul.comment").empty();
                    $("#load").children().text("加载更多");
                    command.clear();
                    getComments();
                } else {
                    openInfo("似乎有点错误", "danger")
                }
            }
        })
    }
});
function closeInfo() {
    $(".alert").css("display", "none")
}
function openInfo(info, level) {
    var css = ".float_alert.alert-" + level;
    $(css).text(info);
    $(css).css("display", "block");
    window.setTimeout(closeInfo, 3000);
}
