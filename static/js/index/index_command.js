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
    this.setTrue = function () {
        this.command = true;
    };
    this.add = function () {
        this.order += 1;
    };
    this.statusChange = function () {
        this.status = !this.status
    };
    this.clear = function () {
        this.order = 0;
        this.command = true;
        this.status = true;
    }
}

// New an object
var command = new Command();
