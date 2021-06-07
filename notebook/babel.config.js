module.exports = function (api) {
    api.cache(true);
    return {
      "presets": ["@babel/preset-typescript"],
      "plugins": [
        "@babel/plugin-syntax-dynamic-import",
        "@babel/proposal-class-properties",
        "@babel/plugin-transform-classes",
      ]
    }
}