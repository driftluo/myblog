"use strict";

$(function () {
    $.getJSON("/api/v1/tag/view?limit=50&&offset=0", function (result) {
        console.log(result);
        var data = [];
        var data_tag = [];
        for (var i in result.data) {
            if (result.data[i].count > 0) {
                data.push({ value: result.data[i].count, name: result.data[i].tag });
                data_tag.push(result.data[i].tag);
            }
        }
        var myChart = echarts.init($('#tags')[0]);
        myChart.setOption({
            tooltip: {
                trigger: 'item',
                formatter: "{a} <br/>{b} : {c} ({d}%)"
            },
            title: {
                text: 'Tag 分布',
                x: 'center'
            },
            legend: {
                orient: 'vertical',
                left: 'left',
                data: data_tag
            },
            roseType: 'angle',
            color: ['red', 'blue', 'yellow', 'black', 'purple', 'pink', 'orange'],
            itemStyle: {
                normal: {
                    shadowBlur: 200,
                    shadowColor: 'rgba(0, 0, 0, 0.5)'
                },
                emphasis: {
                    shadowBlur: 200,
                    shadowColor: 'rgba(0, 0, 0, 0.5)'
                }
            },
            series: [
                {
                    name: 'Tag',
                    type: 'pie',
                    radius: '75%',
                    data: data
                }

            ]
        })
    })
});

$(function () {
    var ydata = [];
    var xdata = [];
    $.getJSON("/api/v1/article/month", function (result) {
        for (var i in result.data) {
            ydata.push(result.data[i].dimension);
            xdata.push(result.data[i].quantity);
        }
        var myChart = echarts.init($('#month')[0]);
        var option = {
            title: {
                x: 'center',
                text: '月文章发布量'
            },
            tooltip: {
                trigger: 'axis',
                axisPointer: {
                    type: 'cross'
                }
            },
            legend: {
                left: 'left',
                data: ['发布量']
            },
            xAxis: {
                data: ydata
            },
            yAxis: {},
            series: [{
                name: '发布量',
                type: 'line',
                data: xdata
            }]
        };
        myChart.setOption(option);
    })
});
