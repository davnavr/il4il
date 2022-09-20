const { buildParserFile } = require("@lezer/generator");

exports.default = function loader(source) {
    return buildParserFile(source).parser;
};
