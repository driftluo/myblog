const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readProjectFile(...segments) {
  return fs.readFileSync(path.join(__dirname, "..", "..", ...segments), "utf8");
}

test("article pages use the shared navigation script", () => {
  const visitorTemplate = readProjectFile("views", "visitor", "article_view.html");
  const adminTemplate = readProjectFile("views", "admin", "article_view.html");
  const navigationScript = readProjectFile(
    "static",
    "js",
    "article_view",
    "article_navigation.js",
  );

  assert.equal(
    visitorTemplate.includes('/js/article_view/article_navigation.js'),
    true,
  );
  assert.equal(
    adminTemplate.includes('/js/article_view/article_navigation.js'),
    true,
  );
  assert.equal(navigationScript.includes("function loadArticleNavigation("), true);
});

test("mobile article navigation keeps the original button order", () => {
  const articleCss = readProjectFile("static", "css", "article_view.css");

  assert.equal(articleCss.includes("order: -1;"), false);
});

test("navigation logic is not duplicated in page-specific scripts", () => {
  const visitorScript = readProjectFile("static", "js", "article_view", "article_view.js");
  const adminTemplate = readProjectFile("views", "admin", "article_view.html");

  assert.equal(visitorScript.includes("function loadArticleNavigation("), false);
  assert.equal(adminTemplate.includes("function loadArticleNavigation("), false);
});
