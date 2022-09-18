const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    mode: "development",
    entry: "./init.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "init.js",
    },
    plugins: [
        new CopyWebpackPlugin(['index.html']),
    ]
};
