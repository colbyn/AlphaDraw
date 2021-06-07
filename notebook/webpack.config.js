const path = require('path');

module.exports = {
    mode: 'development',
    entry: './src/index.ts',
    devtool: 'inline-source-map',
    module: {
        rules: [
            {
                use: {
                    loader: "babel-loader",
                },
                exclude: [/node_modules/]
            }
        ]
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    output: {
        filename: "bundle.js",
        publicPath: "/dist/",
    },
    devServer: {
        contentBase: path.join(__dirname, '.'),
        filename: 'dist/bundle.js',
        compress: false,
        port: 3000,
    },
    optimization: {
        minimize: false,
    },
};