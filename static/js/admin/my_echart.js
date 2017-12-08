"use strict";

$(function () {
    $.getJSON("/api/v1/tag/view?limit=50&&offset=0", function (result) {
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
                    shadowBlur: 50,
                    shadowColor: 'rgba(0, 0, 0, 0.5)'
                },
                emphasis: {
                    shadowBlur: 50,
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
            dataZoom: [
                {
                    type: 'slider',    //支持鼠标滚轮缩放
                    start: 0,            //默认数据初始缩放范围为10%到90%
                    end: 100
                },
                {
                    type: 'inside',    //支持单独的滑动条缩放
                    start: 0,            //默认数据初始缩放范围为10%到90%
                    end: 100
                }
            ],
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
                symbol:'emptycircle',
                smooth:true,
                type: 'line',
                data: xdata
            }]
        };
        myChart.setOption(option);
    })
});
