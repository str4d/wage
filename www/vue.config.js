module.exports = {
    chainWebpack: config => {
        config.entry('polyfills').add('./src/polyfills.js').end();
        config.plugin('html').tap(args => {
            // Load polyfills before the main app.
            args[0].chunksSortMode = function (a, b) {
                var order = ["polyfills", "app"];
                return order.indexOf(a.names[0]) - order.indexOf(b.names[0]);
            };
            return args;
        });
        config.resolve.symlinks(false);
    }
}
