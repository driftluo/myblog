"use strict";
$(function () {
    getTagAndModifyTime();
    hightlight($("pre code"));
    getComments();
    command.statusChange();
});


// loading comments
$("#load").click(function () {
    command.statusChange();
    getComments();
});

// Highlight code
function hightlight($doms) {
    $doms.each(function(i, block) {
        hljs.highlightBlock(block);
    });
}

$("body").on("click", "ul.comment li a.delete", function () {
    var user_id = $(this).parent().children().first().attr("user-id");
    var comment_id = $(this).parent().children().first().attr("comment-id");
    var li = $(this).parent();
    $.ajax({
        url: "/api/v1/comment/delete",
        type: "post",
        dataType: "json",
        data: JSON.stringify({ "comment_id": comment_id, "user_id": user_id }),
        headers: { "Content-Type": "application/json" },
        success: function (res) {
            if (res.status) {
                li.hide(1000, function () {
                    li.remove()
                })
            } else {
                openInfo("你似乎并没有这种权限！", "danger")
            }
        }
    })
});

$("body").on("click", "ul.comment li a.reply", function () {
    var data = new Object();
    var re_user = $(this).parent().children(".head").children("a");
    data.re_comment = $(this).parent().children(".re-comment").html();
    data.re_user_name = re_user.text();
    data.re_user_url = re_user.attr('href');
    var html = template("tpl-reply", data);
    $(".w-e-text").focus();
    editor.txt.html(html)
});

function getTagAndModifyTime() {
    var id = $(".col-md-offset-1[data-id]").attr("data-id");
    $.getJSON("/api/v1/article/view?id=" + id, function (result) {
        $(".col-md-offset-1[data-id]")
            .append("<blockquote class='pull-right'><h5 class='post-meta'>Last Modified:</h5>" +
            "<p class='pull-right post-meta'>" + moment.utc(result.data.modify_time).local().format() +
            "</p></blockquote>");
        var tags = { data: [] };

        $.each(result.data.tags, function (index, value) {
            if (value !== null) {
                tags.data.push([result.data.tags_id[index], value])
            }
        });
        var html = template("tpl-tag-list", tags);
        $(".col-md-offset-1[data-id]").children().first().after(html)
    });
}

function getComments() {
    if (command.command) {
        var id = $(".col-md-offset-1[data-id]").attr("data-id");
        $.getJSON("/api/v1/article/view_comment/" + id + "?limit=10&&offset=" + command.order * 10, function (result) {
            command.add();
            if (result.data.length < 5) {
                command.change();
                $("#load").children().text("没有更多了");
            }
            for (var index in result.data) {
                result.data[index].create_time = moment.utc(result.data[index].create_time).local().format();
                result.data[index]["admin"] = result.admin;
                if (result.user) {
                    result.data[index]["user"] = result.user;
                }
            }
            var html = template("tpl-comment-list", result);
            $("ul.comment").append(html);
            command.statusChange();
        })
    }
}
