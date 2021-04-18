"use strict";
var page = new Page("user");

$(function () {
    getUserList();
});

function getUserList() {
    $.getJSON("/api/v1/user/view_all?limit=10&&offset=" + page.page * 10, function (result) {
        if (result.data.length < 10) {
            $("#next").attr({ "disabled": "disabled" });
        }
        for (var index in result.data) {
            result.data[index].create_time = moment.utc(result.data[index].create_time).local().format("YYYY-MM-DD HH:mm:ss");
            if (result.data[index].groups === 0) {
                result.data[index].group_name = "Admin"
            } else {
                result.data[index].group_name = "User"
            }
        }
        var html = template("tpl-user-list", result);
        $("tbody").append(html);
        deleteButton();
        permissionButton();
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
                url: "/api/v1/user/delete/" + id,
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

function permissionButton() {
    $(".permission").on("click", function (event) {
        event.preventDefault();
        var permission = $(this).parent().prev().prev().prev().children().attr("data-id") === "1" ? 0 : 1;
        var permission_element = $(this).parent().prev().prev().prev().children();
        var id = $(this).attr("data-id");
        $.ajax({
            url: "/api/v1/user/permission",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "id": id, "permission": permission }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    permission_element.attr("data-id", permission);
                    if (permission === 0) {
                        permission_element.text("Admin")
                    } else {
                        permission_element.text("User")
                    }
                }
            }
        })
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
    getUserList();
});

$("#next").click(function (event) {
    event.preventDefault();
    page.add();
    if (page.page > 0) {
        $("#previous").removeAttr("disabled");
    }
    $("tbody").html("");
    getUserList();
});
