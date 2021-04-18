"use strict";
function Page(name) {
    var page = sessionStorage.getItem(name);
    if (!page && typeof(page)!="undefined") {
        this.page = 0;
    } else {
        if (page > 0) {
            $("#previous").removeAttr("disabled");
        }
        this.page = Number(page);
    }
    this.add = function () {
        this.page += 1;
        sessionStorage.setItem(name, this.page);
    };
    this.sub = function () {
        this.page -= 1;
        sessionStorage.setItem(name, this.page);
    }
}