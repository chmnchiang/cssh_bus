"use strict"

var UPDATE_PER_SEC = 30;

var app = new Vue({
    el: '#app',
    data: {
        routes: [],
    },
    mounted: function() {
        this.tickSec = -0.1;
        this.fetching = false;
        this.$progressBar = $('#progress');
        this.$progressBar.progress({ percent: 0 });
        this.tick()
        setInterval(this.tick, 60);
    },
    methods: {
        tick: function() {
            if (this.tickSec < 0) {
                if (!this.fetching) {
                    this.fetching = true;
                    this.$progressBar.progress({ percent: 100 });
                    this.fetch();
                }
            } else {
                this.tickSec -= 0.06;
                this.$progressBar.progress({
                    percent: (UPDATE_PER_SEC - Math.max(this.tickSec, 0)) / 30 * 100,
                });
            }
        },
        fetch: function() {
            var me = this;
            $.getJSON("ajax/routes", function(data) {
                data = data.filter(function(x) { return x.eta < 200; })
                    .sort(function(x, y) { return x.eta - y.eta; })
                    .slice(0, 10);
                me.routes = data;
            }).always(function() {
                me.tickSec = UPDATE_PER_SEC;
                me.fetching = false;
            });
        },
    }
})
