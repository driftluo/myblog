"use strict";

// Bootstrap 5 Modal Helper
function showModal(selector) {
  var el = document.querySelector(selector);
  if (el) {
    bootstrap.Modal.getOrCreateInstance(el).show();
  }
}

$("#submit").click(function (event) {
  event.preventDefault();
  var title = $("#title").val();
  var content = $("#editor").val();
  var id = $("#title").attr("article-id");
  // Get the ids of newly selected existing tags
  var new_choice_already_exists_tags = $(
    "ul.tag li a[data-id].a_click:not([have_chosen])",
  )
    .map(function () {
      return $(this).attr("data-id");
    })
    .toArray();
  // Get the ids of deselected tags
  var deselect_tags = $("ul.tag li a[have_chosen]")
    .map(function () {
      if (!$(this).hasClass("a_click")) {
        return $(this).attr("data-id");
      }
    })
    .toArray();
  // Get the text of newly added tags
  var new_tag = $("ul.tag li a:not([data-id]).a_click")
    .map(function () {
      return $(this).html();
    })
    .toArray();
  if (title === "" || content === "") {
    showModal("#mistake");
  } else {
    $.ajax({
      url: "/api/v1/article/edit",
      type: "post",
      dataType: "json",
      data: JSON.stringify({
        id: id,
        title: title,
        raw_content: content,
        new_choice_already_exists_tags: new_choice_already_exists_tags,
        deselect_tags: deselect_tags,
        new_tags: new_tag,
      }),
      headers: { "Content-Type": "application/json" },
      success: function (res) {
        if (res.status) {
          showModal("#success");
        }
      },
    });
  }
});
$(function () {
  var id = $("#title").attr("article-id");
  $.getJSON("/api/v1/article/admin/view_raw?id=" + id, function (result) {
    $("#title").val(result.data.title);
    $("#editor").val(result.data.content);
    $("ul.tag li a").map(function () {
      if ($.inArray($(this).attr("data-id"), result.data.tags_id) !== -1) {
        $(this).addClass("a_click");
        $(this).attr("have_chosen", true);
      }
    });
  });
});
