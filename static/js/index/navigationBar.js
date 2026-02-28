"use strict";

// Navigation bar color gradient
$(window).scroll(function () {
    if ($(".navbar").offset().top > 350) {
        $(".fixed-top").addClass("top-nav");
    } else {
        $(".fixed-top").removeClass("top-nav");
    }
});
