const path = require("path");
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  entry: {
    index: "./tests/web/demo.ts"
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: "bundle.js",
    clean: true,
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      title: 'GMSOL SDK',
      template: path.resolve(__dirname, 'tests/web/index.html'),
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--scope gmsol-labs --features wasm",
    }),
  ],
  devServer: {
    static: './dist',
    open: true,
  },
  mode: "development",
};