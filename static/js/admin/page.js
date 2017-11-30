"use strict";
function Page() {
    this.page = 0;
    this.add = function () {
        this.page += 1;
    };
    this.sub = function () {
        this.page -= 1;
    }
}

var page = new Page();