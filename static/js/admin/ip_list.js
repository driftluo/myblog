"use strict";
$(function () {
    get_ip()
});

function get_ip() {
    $.getJSON("/api/v1/ip/view?limit=10&offset=" + page.page * 10, function (result) {
        if (result.data.length < 10) {
            $("#next").attr({ "disabled": "disabled" });
        }

        for(var i = 0; i < result.data.length; i++){
            var data = JSON.parse(result.data[i]);
            data.timestamp = moment.utc(data.timestamp).local().format();
            var html = template("tpl-ip", data);
            $("tbody").append(html);
            result.data[i] = data;
        }

        for (var i = 0; i < result.data.length; i++) {
            $.getJSON("//www.freegeoip.net/json/" + result.data[i].ip, function (res) {
                $(".region[data-ip='" + res.ip+ "']").text(res.region_name);
                $(".country[data-ip='" + res.ip+ "']").text(res.country_name);
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
