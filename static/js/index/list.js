"use strict";
function Page() {
    this.page = 0;
    this.add = function () {
        this.page += 1;
    };
    this.sub = function () {
        this.page -= 1;
    }
}

var page = new Page();

$(function () {
    getArticleList();
});

function getArticleList() {
    $.getJSON("/api/v1/article/admin/view_all?limit=10&&offset=" + page.page * 10, function (result) {
        if (result.data.length < 10) {
            $("#next").attr({ "disabled": "disabled" });
        }
        for (var index in result.data) {
            result.data[index].create_time = moment.utc(result.data[index].create_time).local().format("YYYY-MM-DD HH:mm:ss");
            result.data[index].modify_time = moment.utc(result.data[index].modify_time).local().format("YYYY-MM-DD HH:mm:ss");
        }
        var html = template("tpl-article-list", result);
        $("tbody").append(html);
        // register for click events
        publishButton();
        deleteButton();
        modifyButton();
    });

}

$("#previous").click(function (event) {
    event.preventDefault();
    page.sub();
    $("#next").removeAttr("disabled");
    if (page.page === 0) {
        $("#previous").attr({ "disabled": "disabled" });
    }
    $("tbody").html("");
    getArticleList();
});

$("#next").click(function (event) {
    event.preventDefault();
    page.add();
    if (page.page > 0) {
        $("#previous").removeAttr("disabled");
    }
    $("tbody").html("");
    getArticleList();
});
