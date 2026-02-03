"use strict";

// Bootstrap 5 Modal Helper
function showModal(selector) {
  var el = document.querySelector(selector);
  if (el) {
    var modal = bootstrap.Modal.getOrCreateInstance(el);
    modal.show();
  }
}

$("#submit").click(function (event) {
  event.preventDefault();
  var title = $("#title").val();
  var content = $("#editor").val();
  // Get the id of the selected existing tag
  var exist_tags = $("ul.tag li a[data-id].a_click")
    .map(function () {
      return $(this).attr("data-id");
    })
    .toArray();
  // Get the text information of the newly added tag
  var new_tags = $("ul.tag li a:not([data-id]).a_click")
    .map(function () {
      return $(this).html();
    })
    .toArray();
  if (title === "" || content === "") {
    showModal("#mistake");
  } else {
    $.ajax({
      url: "/api/v1/article/new",
      type: "post",
      dataType: "json",
      data: JSON.stringify({
        title: title,
        raw_content: content,
        exist_tags: exist_tags,
        new_tags: new_tags,
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
