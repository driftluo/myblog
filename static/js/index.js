"use strict";
// Record the status of the request asynchronously again
function Command() {
    this.command = true;
}

// New an object
var command = new Command();

// Asynchronous request for article list
function getList() {
    if (command.command) {
        var offset = $("div.col-md-8 h2").length;
        $.getJSON("/api/v1/article/admin/view_all?limit=5&&offset=" + offset, function (result) {
            // console.log(result.status);
            if (result.data.length === 0) {
                command.command = false
            }
            $.each(result, function (key, values) {
                var i;
                for (i in values) {
                    var p = $("<p class='post-meta'>Posted on " + moment.utc(values[i].create_time).local().format() + "</p>");
                    var title = $("<a href='#'>" + "<h2>" + values[i].title + "<br><small>测试副标题</small>" + "</h2>" + "</a>");
                    var blog = $("<div class='text-center'></div>").append(title).append(p);
                    $("div.col-md-8").append(blog);
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
    if ($(document).scrollTop() + $(window).height() >= $(document).height()) {
        getList();
    }
});

// Navigation bar color gradient
$(window).scroll(function () {
    if ($(".navbar").offset().top > 350) {
        $(".navbar-fixed-top").addClass("top-nav");
    } else {
        $(".navbar-fixed-top").removeClass("top-nav");
    }
});

// According to the tag to get the corresponding article list
// todo: finish it
$("button").click(function () {
    console.log($(this).parent().attr("data-id"))
});
