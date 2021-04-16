"use strict";
marked.setOptions({
    renderer: new marked.Renderer(),
    gfm: true,
    tables: true,
    breaks: false,
    pedantic: false,
    sanitize: false,
    smartLists: true,
    smartypants: false,
    highlight: function (code) {
        return hljs.highlightAuto(code).value;
    }
});

function editorPage() {
    $("#preview_page").css("display", "none");
    $("#preview_note").removeClass("active");
    $("#editor_page").css("display", "block");
    $("#editor_note").addClass("active");
}

function previewPage() {
    $("#editor_page").css("display", "none");
    $("#editor_note").removeClass("active");
    $("#preview_page").css("display", "block");
    $("#preview_note").addClass("active");
    var html = marked($("#editor").val());
    $("#preview_page").html(html);
    $("pre code").addClass("hljs")
}

$("textarea").on(
    'keydown',
    function (e) {
        if (e.keyCode === 9) {
            e.preventDefault();
            var indent = '    ';
            var start = this.selectionStart;
            var end = this.selectionEnd;
            var selected = window.getSelection().toString();
            selected = indent + selected.replace(/\n/g, '\n' + indent);
            this.value = this.value.substring(0, start) + selected
                + this.value.substring(end);
            this.setSelectionRange(start + indent.length, start
                + selected.length);
        }
    });

$("#quit").click(function (event) {
    event.preventDefault();
    window.location = "/admin/list"
});
$("#success_btn").click(function (event) {
    event.preventDefault();
    window.location = "/admin/list"
});

$("ul.tag").on("click", "li", function () {
    if ($(this).children().hasClass("a_click")) {
        $(this).children().removeClass("a_click")
    } else {
        $(this).children().addClass("a_click")
    }
});
$("#tag_btn").on("click", function () {
    $("#tag-name").val("");
    $("#tag").modal("show")
});
$("#add_tag").click(function () {
    var tag_name = $("#tag-name").val().replace(/(^\s*)|(\s*$)/g, "");
    var tags = $('ul.tag li a').map(function () {
        return $(this).html();
    }).toArray();
    if (tag_name === "" || $.inArray(tag_name, tags) !== -1) {
        $(".text-danger").remove();
        $(this).parent().prev().append("<span class='text-danger'>标签已存在或为空</span>")
    } else {
        $('#tag').modal('hide');
        $("ul.tag").append("<li><a class='a_click'>" + tag_name + "</a></li>")
    }

});

$("button.pull-left").click(function (event) {
    event.preventDefault();
    $("#upload_modal").modal("show");
    $(".modal-upload").click(function () {
        var files = document.getElementById("file").files;
        var form = new FormData();
        for (var i = 0; i < files.length; i++) {
            form.append("files", files[i], files[i].name);
        }

        var request = new XMLHttpRequest();
        request.open("POST", "/api/v1/upload", true);
        request.onload = function(evt) {
            var res = JSON.parse(evt.target.responseText);
            if(res.status) {
                alert(res.data)
            }
        }
        request.send(form);
    });
});
