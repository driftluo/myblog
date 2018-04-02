"use strict";
$(function () {
    get_ip()
});

function get_ip() {
    $.getJSON("/api/v1/ip/view?limit=25&offset=" + page.page * 25, function (result) {
        if (result.data.length < 10) {
            $("#next").attr({ "disabled": "disabled" });
        }
        var ip_unique, ip_list = [];
        for(var i = 0; i < result.data.length; i++){
            var data = JSON.parse(result.data[i]);
            data.timestamp = moment.utc(data.timestamp).local().format("YYYY-MM-DD HH:mm:ss");
            var html = template("tpl-ip", data);
            $("tbody").append(html);
            ip_list.push(data.ip)
        }

        ip_unique = ip_list.filter(function (element,index,self) { return self.indexOf(element) === index; });

        for (var i = 0; i < ip_unique.length; i++) {
            $.getJSON("//www.freegeoip.net/json/" + ip_unique[i], function (res) {
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
