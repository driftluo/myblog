"use strict";

// Navigation bar color gradient
$(window).scroll(function () {
    if ($(".navbar").offset().top > 350) {
        $(".navbar-fixed-top").addClass("top-nav");
    } else {
        $(".navbar-fixed-top").removeClass("top-nav");
    }
});
