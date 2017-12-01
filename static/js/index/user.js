"use strict";

$("#information_btn").click(function () {
    informationPage();
});
$("#sign_out_btn").click(function () {
    $("#information").css("display", "none");
    $("#sign_out").css("display", "block");
    $("#modify").css("display", "none");
    $("#change_password").css("display", "none")
});
$("#modify_btn").click(function () {
    $("#information").css("display", "none");
    $("#sign_out").css("display", "none");
    $("#modify").css("display", "block");
    $("#change_password").css("display", "none");
    getInfo()
});
$("#change_password_btn").click(function () {
    $("#information").css("display", "none");
    $("#sign_out").css("display", "none");
    $("#modify").css("display", "none");
    $("#change_password").css("display", "block");
    clearPassword()
});

$(document).ready(function () {
    getUserInfo();
}
);

$("#sign_out :button").click(function () {
    $.ajax({
        url: "/api/v1/user/sign_out",
        type: "get",
        dataType: "json",
        data: JSON.stringify({}),
        headers: { "Content-Type": "application/json" },
        success: function (res) {
            if (res.status) {
                window.location = "/index"
            }
        }
    })
});

$("#modify :button").click(function (event) {
    event.preventDefault();
    var nickname = $("#nickname").val().replace(/(^\s*)|(\s*$)/g, "");
    var say = $("#say").val();
    var email = $("#email").val().replace(/(^\s*)|(\s*$)/g, "");
    $(".text-danger").remove();
    if (emailVerification(email) && nickname !== "") {
        $.ajax({
            url: "/api/v1/user/edit",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "nickname": nickname, "say": say, "email": email }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    getUserInfo();
                    informationPage();
                }
            }
        });
    } else {
        $(this).before("<span class='text-danger' style='display: block'>Email格式错误或昵称为空</span>")
    }
});

$("#change_password :button").click(function (event) {
    event.preventDefault();
    $(".text-danger").remove();
    var old_password = $("#old_password").val();
    var re_password = $("#re_password").val();
    var new_password = $("#new_password").val();
    if (old_password.length < 5) {
        $("#old_password").after("<span class='text-danger' style='display: block'>密码长度小于5</span>")
    } else if (new_password.length < 5) {
        $("#new_password").after("<span class='text-danger' style='display: block'>密码长度小于5</span>")
    } else if (new_password !== re_password) {
        $("#re_password").after("<span class='text-danger' style='display: block'>两次密码不一致</span>")
    } else {
        old_password = randomString(6) + old_password;
        new_password = randomString(6) + new_password;
        $.ajax({
            url: "/api/v1/user/change_pwd",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "old_password": old_password, "new_password": new_password }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    informationPage();
                } else {
                    $("#re_password").after("<span class='text-danger' style='display: block'>密码错误</span>")
                }
            }
        });
    }
});

function getUserInfo() {
    $.getJSON("/api/v1/user/view", function (result) {
        result.data.create_time = moment.utc(result.data.create_time).local().format();
        var html = template("tpl-user", result);
        $("#information").empty();
        $("#information").append(html)
    })
}

function getInfo() {
    $("#nickname").val($(".nickname").text());
    $("#say").val($(".say").text());
    $("#email").val($(".email").text())
}

function emailVerification(email) {
    var reg_email = /^([a-zA-Z0-9]+[_|\_|\.]?)*[a-zA-Z0-9]+@([a-zA-Z0-9]+[_|\_|\.]?)*[a-zA-Z0-9]+\.[a-zA-Z]{2,3}$/;
    return reg_email.test(email);
}

function informationPage() {
    $("#information").css("display", "block");
    $("#sign_out").css("display", "none");
    $("#modify").css("display", "none");
    $("#change_password").css("display", "none")
}

function clearPassword() {
    $("#old_password").val("");
    $("#re_password").val("");
    $("#new_password").val("");
    $("#old_password").focus();
}

function randomString(len) {
    len = len || 32;
    var $chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var maxPos = $chars.length;
    var pwd = '';
    for (var i = 0; i < len; i++) {
        //0~32的整数
        pwd += $chars.charAt(Math.floor(Math.random() * (maxPos + 1)));
    }
    return pwd;
}
