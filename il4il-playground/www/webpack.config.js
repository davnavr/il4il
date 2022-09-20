const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    mode: "development",
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
    },
    experiments: {
        asyncWebAssembly: true,
    },
    module: {
        rules: [
            {
                test: /\.grammar$/,
                use: {
                    loader: path.resolve("./lezer-loader.js"),
                },
            }
        ]
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                "index.html",
                "style.css",
            ]
        }),
    ]
};
