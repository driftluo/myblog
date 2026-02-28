"use strict";
var page = new Page("tag");

// Bootstrap 5 Modal Helpers
function showModal(selector) {
  var el = document.querySelector(selector);
  if (el) {
    var modal = bootstrap.Modal.getOrCreateInstance(el);
    modal.show();
  }
}

function hideModal(selector) {
  var el = document.querySelector(selector);
  if (el) {
    var modal = bootstrap.Modal.getOrCreateInstance(el);
    modal.hide();
  }
}

$(function () {
  getTags();
});

function getTags() {
  $.getJSON(
    "/api/v1/tag/view?limit=10&&offset=" + page.page * 10,
    function (result) {
      if (result.data.length < 10) {
        $("#next").attr({ disabled: "disabled" });
      }
      var html = template("tpl-tag-list", result);
      $("tbody").append(html);
      deleteButton();
      modifyButton();
    },
  );
}

$("button.float-end").click(function (event) {
  event.preventDefault();
  $("#tag-name").val("");
  showModal("#tag");
});

$("#add_tag").click(function () {
  var tag_name = $("#tag-name")
    .val()
    .replace(/(^\s*)|(\s*$)/g, "");
  if (tag_name === "") {
    $(".text-danger").remove();
    $(this).parent().prev().append("<span class='text-danger'>标签为空</span>");
  } else {
    hideModal("#tag");
    $.ajax({
      url: "/api/v1/tag/new",
      type: "post",
      dataType: "json",
      data: JSON.stringify({ tag: tag_name }),
      headers: { "Content-Type": "application/json" },
      success: function (res) {
        if (res.status) {
        }
      },
    });
  }
});

function deleteButton() {
  $(".delete").on("click", function (event) {
    event.preventDefault();
    showModal("#delete_modal");
    var tr = $(this).parent().parent();
    var id = $(this).attr("data-id");
    $(".modal-delete").off("click").on("click", function () {
      $.ajax({
        url: "/api/v1/tag/delete/" + id,
        type: "post",
        dataType: "json",
        data: "",
        headers: { "Content-Type": "application/json" },
        success: function (res) {
          if (res.status) {
            tr.hide(1000, function () {
              tr.remove();
            });
          }
        },
      });
    });
  });
}

function modifyButton() {
  $(".modify").on("click", function (event) {
    event.preventDefault();
    showModal("#modify");
    var pre_tag = $(this).parent().prev().prev().children().text();
    var pre_tag_element = $(this).parent().prev().prev().children();
    $("#modify-tag-name").val(pre_tag);
    var id = $(this).attr("data-id");
    $("#modify_tag").off("click").on("click", function () {
      var tag_name = $("#modify-tag-name")
        .val()
        .replace(/(^\s*)|(\s*$)/g, "");
      if (tag_name === "" || tag_name === pre_tag) {
        $(".text-danger").remove();
        $(this)
          .parent()
          .prev()
          .append("<span class='text-danger'>标签为空或未修改</span>");
      } else {
        $.ajax({
          url: "/api/v1/tag/edit",
          type: "post",
          dataType: "json",
          data: JSON.stringify({ id: id, tag: tag_name }),
          headers: { "Content-Type": "application/json" },
          success: function (res) {
            if (res.status) {
              pre_tag_element.text(tag_name);
              hideModal("#modify");
            }
          },
        });
      }
    });
  });
}

$("#previous").click(function (event) {
  event.preventDefault();
  page.sub();
  $("#next").removeAttr("disabled");
  if (page.page === 0) {
    $("#previous").attr({ disabled: "disabled" });
  }
  $("tbody").html("");
  getTags();
});

$("#next").click(function (event) {
  event.preventDefault();
  page.add();
  if (page.page > 0) {
    $("#previous").removeAttr("disabled");
  }
  $("tbody").html("");
  getTags();
});
