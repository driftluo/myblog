"use strict";

// Record the status of the request asynchronously again
function Command() {
    this.command = true;
    this.order = 0;
    this.change = function () {
        this.command = !this.command;
    };
    this.setFalse = function () {
        this.command = false;
    };
    this.add = function () {
        this.order += 1;
    }
}

// New an object
var command = new Command();

// Asynchronous request for article list
function getList() {
    if (command.command) {
        $.getJSON("/api/v1/article/view_all?limit=20&&offset=" + command.order * 20, function (result) {
            // console.log(result.status);
            command.add();
            if (result.data.length === 0) {
                command.change();
            }
            $.each(result, function (key, values) {
                var i;
                for (i in values) {
                    var p = $("<p class='post-meta'>Posted on " + moment.utc(values[i].create_time).local().format() + "</p>");
                    var title = $("<a href='#'>" + "<h2>" + values[i].title + "<br><small>测试副标题</small>" + "</h2>" + "</a>");
                    var blog = $("<div class='text-center'></div>").append(title).append(p);
                    $("div.col-md-10").append(blog);
                }

            })
        });
    }
}

// First visit, asynchronously access article list
$(document).ready(function () {
        getList();
    }
);

// After scroll on the end, asynchronous access to follow-up article list
$(window).scroll(function () {
    if ($(window).scrollTop() + $(window).height() >= $(document).height() * 0.99) {
        getList();
    }
});
