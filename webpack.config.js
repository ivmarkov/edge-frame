const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const autoprefixer = require("autoprefixer");

const dist = path.resolve(__dirname, "dist");

module.exports = [
{
    entry: "./js/app.scss",
    output: {
      // This is necessary for webpack to compile,
      // but we never use style-bundle.js.
      filename: "style-bundle.js",
    },
    module: {
      rules: [{
        test: /\.scss$/,
        use: [
          {
            loader: "file-loader",
            options: {
              name: "bundle.css",
            },
          },
          { loader: "extract-loader" },
          { loader: "css-loader" },
          { 
            loader: "postcss-loader",
            options: {
              postcssOptions: {
                plugins: () => [autoprefixer()]
              }
            }
          },  
          {
            loader: "sass-loader",
            options: {
              // Prefer Dart Sass
              implementation: require("sass"),

              // See https://github.com/webpack-contrib/sass-loader/issues/804
              webpackImporter: false,
              sassOptions: {
                includePaths: ["./node_modules"]
              },
            },
          },
        ]
      }]
    },
  },
  {
  mode: "production",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static")
    ]),

    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ]
}
];
