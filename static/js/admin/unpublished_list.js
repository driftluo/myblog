"use strict";
var page = new Page("unpublished");

$(function () {
    getArticleList();
});

function getArticleList() {
    $.getJSON("/api/v1/article/admin/view_unpublished?limit=10&&offset=" + page.page * 10, function (result) {
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

$("button.pull-right").click(function (event) {
    event.preventDefault();
    window.location = "/admin/new"
});

function publishButton() {
    $(".publish").on("click", function (event) {
        event.preventDefault();
        var raw = $(this).parent().parent().children("td:nth-child(3)").children().html();
        var status = raw === "false" ? false : true;
        $.ajax({
            url: "/api/v1/article/publish",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "id": $(this).attr("data-id"), "publish": !status }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {

                }
            }
        });
        $(this).parent().parent().children("td:nth-child(3)").children().html((!status).toString())
    });
}

function deleteButton() {
    $(".delete").on("click", function (event) {
        event.preventDefault();
        $("#delete_modal").modal("show");
        var tr = $(this).parent().parent();
        var id = $(this).attr("data-id");
        $(".modal-delete").click(function () {
            $.ajax({
                url: "/api/v1/article/delete/" + id,
                type: "post",
                dataType: "json",
                data: "",
                headers: { "Content-Type": "application/json" },
                success: function (res) {
                    if (res.status) {
                        tr.hide(1000, function () {
                            tr.remove()
                        })
                    }
                }
            })
        })
    })
}

function modifyButton() {
    $(".modify").on("click", function (event) {
        event.preventDefault();
        var id = $(this).attr("data-id");
        window.location = "/admin/article/edit?id=" + id;
    })
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
