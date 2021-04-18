"use strict";
function Page() {
    var page = sessionStorage.getItem("page");
    if (!page && typeof(page)!="undefined") {
        this.page = 0;
    } else {
        if (page > 0) {
            $("#previous").removeAttr("disabled");
        }
        this.page = Number(page);
    }
    this.add = function () {
        this.page += 1;
        sessionStorage.setItem("page", this.page);
    };
    this.sub = function () {
        this.page -= 1;
        sessionStorage.setItem("page", this.page);
    }
}

var page = new Page();

$(function () {
    getArticleList();
});

function getArticleList() {
    $.getJSON("/api/v1/article/view_all?limit=20&&offset=" + page.page * 20, function (result) {
        if (result.data.length < 20) {
            $("#next").attr({ "disabled": "disabled" });
        }
        for (var index in result.data) {
            result.data[index].create_time = moment.utc(result.data[index].create_time).local().format("YYYY-MM-DD HH:mm:ss");
            result.data[index].modify_time = moment.utc(result.data[index].modify_time).local().format("YYYY-MM-DD HH:mm:ss");
        }
        var html = template("tpl-article-list", result);
        $("tbody").append(html);
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
