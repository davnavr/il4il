const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    mode: "development",
    entry: "./index.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "index.js",
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
