"use strict";

function loadArticleNavigation($article) {
  if (!$article.length) {
    return;
  }

  var articleId = $article.attr("data-id");
  var viewMode = $article.attr("data-view-mode") || "visitor";
  var navigationUrl =
    viewMode === "admin"
      ? "/api/v1/article/admin/navigation?id=" + articleId
      : "/api/v1/article/navigation?id=" + articleId;

  updateDirectionalButton(
    $(".article-nav-previous"),
    null,
    viewMode,
    "暂无更早文章",
  );
  updateDirectionalButton(
    $(".article-nav-next"),
    null,
    viewMode,
    "暂无更新文章",
  );

  $.getJSON(navigationUrl, function (result) {
    if (!result.status) {
      return;
    }

    updateDirectionalButton(
      $(".article-nav-previous"),
      result.data.previous,
      viewMode,
      "暂无更早文章",
    );
    updateDirectionalButton(
      $(".article-nav-next"),
      result.data.next,
      viewMode,
      "暂无更新文章",
    );
  }).fail(function () {
    console.warn("Failed to load article navigation");
  });
}

function updateDirectionalButton($button, article, viewMode, emptyTitle) {
  var href = article
    ? getArticleUrl(article.id, viewMode)
    : "javascript:void(0)";
  $button.attr("href", href);
  $button.find(".article-nav-title").text(article ? article.title : emptyTitle);

  if (article) {
    $button.removeClass("is-disabled").removeAttr("aria-disabled");
  } else {
    $button.addClass("is-disabled").attr("aria-disabled", "true");
  }
}

function getArticleUrl(articleId, viewMode) {
  if (viewMode === "admin") {
    return "/admin/article/view?id=" + articleId;
  }

  return "/article/" + articleId;
}
