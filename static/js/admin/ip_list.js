"use strict";
$(function () {
    get_ip()
});

function get_ip() {
    $.getJSON("/api/v1/ip/view?limit=10&offset=" + page.page * 10, function (result) {
        if (result.data.length < 10) {
            $("#next").attr({ "disabled": "disabled" });
        }
        for (var i in result.data) {
            $.getJSON("http://www.freegeoip.net/json/" + result.data[i], function (res) {
                var html = template("tpl-ip", res);
                $("tbody").append(html);
            })
        }
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
    get_ip();
});

$("#next").click(function (event) {
    event.preventDefault();
    page.add();
    if (page.page > 0) {
        $("#previous").removeAttr("disabled");
    }
    $("tbody").html("");
    get_ip();
});
