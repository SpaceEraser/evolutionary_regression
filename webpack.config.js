const path = require("path");
const webpack = require('webpack');
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    mode: "production",
    performance: { hints: false },
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bundle.js",
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            extraArgs: "--no-typescript",
        }),
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, "index.html"),
        }),
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
            TextDecoder: ['@sinonjs/text-encoding', 'TextDecoder'],
            TextEncoder: ['@sinonjs/text-encoding', 'TextEncoder'],
        }),
    ],
};
