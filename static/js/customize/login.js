"use strict";
$().ready(function () {
    loginValidate();
    registerValidate();
});

$("#register_btn").click(function () {
    $("#register_form").css("display", "block");
    $("#login_form").css("display", "none");
});
$("#back_btn").click(function () {
    $("#register_form").css("display", "none");
    $("#login_form").css("display", "block");
});
$("#sign_btn").click(function () {
    if (registerValidate().form()) {
        console.log($("#account").val())
    }
});
$("#login").click(function () {
    if (loginValidate().form()) {
        var account = $("#login_account").val();
        var password = randomString(6) + $("#login_password").val();
        // $.post("/api/v1/user/login", {"account": account, "password": password}, function (result) {
        //     console.log(result.status);
        // })
        $.ajax({
            url: "/api/v1/user/login",
            type: "post",
            dataType: "json",
            data: JSON.stringify({"account": account, "password": password}),
            headers: {'Content-Type': 'application/json'},
            success: function (res) {
                console.log(res.status);
                window.location = "/home"
            }
        })
    }
});

function registerValidate() {
    return $("#register_form").validate({
        rules: {
            account: "required",
            password: {
                required: true,
                minlength: 5
            },
            rpassword: {
                equalTo: "#register_password"
            },
            email: {
                required: true,
                email: true
            },
            nickname: "required"
        },
        messages: {
            account: "请输入账号",
            password: {
                required: "请输入密码",
                minlength: $.validator.format("密码不能小于{0}个字 符")
            },
            rpassword: {
                required: "请输入密码",
                equalTo: "两次密码不一样"
            },
            email: {
                required: "请输入邮箱",
                email: "请输入有效邮箱"
            },
            nickname: "请输入昵称"
        }
    })
}

function loginValidate() {
    return $("#login_form").validate({
        rules: {
            account: "required",
            password: {
                required: true,
                minlength: 5
            }
        },
        messages: {
            account: "请输入账号",
            password: {
                required: "请输入密码",
                minlength: $.validator.format("密码不能小于{0}个字 符")
            }
        }
    })
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
