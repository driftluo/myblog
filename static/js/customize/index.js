"use strict";

// Record the status of the request asynchronously again
function Command() {
    this.command = true;
    this.order = 0;
    this.status = true;
    this.change = function () {
        this.command = !this.command;
    };
    this.setFalse = function () {
        this.command = false;
    };
    this.add = function () {
      this.order += 1;
    };
    this.statusChange = function () {
        this.status = !this.status
    }
}

// New an object
var command = new Command();

// Asynchronous request for article list
function getList() {
    if (command.command) {
        $.getJSON("/api/v1/article/view_all?limit=5&&offset=" + command.order * 5, function (result) {
            // console.log(result.status);
            command.add();
            if (result.data.length === 0) {
                command.change();
            }
            for (var index in result.data) {
                result.data[index].create_time = moment.utc(result.data[index].create_time).local().format();
                result.data[index].modify_time = moment.utc(result.data[index].modify_time).local().format();
            }
            var html = template("tpl-article-list", result);
            $("div.col-md-8").append(html);
            command.statusChange();
        });
    }
}

// First visit, asynchronously access article list
$(document).ready(function () {
        getList();
        command.statusChange();
    }
);

// After scroll on the end, asynchronous access to follow-up article list
$(window).scroll(function () {
    if ($(window).scrollTop() + $(window).height() >= $(document).height() - 1 && command.status) {
        command.statusChange();
        getList();
    }
});

// According to the tag to get the corresponding article list
// todo: finish it
$("button").click(function () {
    console.log($(this).parent().attr("data-id"));
    command.setFalse();
});
