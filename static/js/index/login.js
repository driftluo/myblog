"use strict";
$().ready(function () {
    loginValidate();
    registerValidate();
    document.onkeydown = function (e) {
        var ev = document.all ? window.event : e;
        if (ev.keyCode === 13) {
            if ($("#login_form").css("display") === 'block') {
                login()
            } else {
                register()
            }
        }
    }
});

$("#register_btn").click(function () {
    $("#register_form").css("display", "block");
    $("#login_form").css("display", "none");
    $("input[type!='button']").val("");
    $("#account").focus();
});
$("#back_btn").click(function () {
    $("#register_form").css("display", "none");
    $("#login_form").css("display", "block");
    $("input[type!='button']").val("");
    $("#login_account").focus();
});

$("#login").click(function (event) {
    event.preventDefault();
    login()
}
);

$("#sign_btn").click(function (event) {
    event.preventDefault();
    register()
}
);

function login() {
    if (loginValidate().form()) {
        var account = $("#login_account").val();
        var password = randomString(6) + $("#login_password").val();
        var remember = $(".checkbox").children().is(':checked');
        $.ajax({
            url: "/api/v1/user/login",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "account": account, "password": password, "remember": remember }),
            headers: { 'Content-Type': 'application/json' },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    $(".text-danger").remove();
                    if (res.error === "NotFound") {
                        $(".checkbox").parent().before("<span class='text-danger'>用户被锁定或未创建</span>")
                    } else {
                        $(".checkbox").parent().before("<span class='text-danger'>用户名或密码错误</span>")
                    }
                }
            }
        })
    }
}

function register() {
    if (registerValidate().form()) {
        var account = $("#account").val();
        var password = randomString(6) + $("#register_password").val();
        var nickname = $("#nickname").val();
        var email = $("#email").val();
        $.ajax({
            url: "/api/v1/user/new",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "account": account, "password": password, "nickname": nickname, "email": email }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    $(".text-danger").remove();
                    $("#sign_btn").parent().before("<span class='text-danger'>用户已创建</span>")
                }
            }
        })
    }
}

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
