const path = require('path');

module.exports = {
  entry: './index.js',
  output: {
    path: `${__dirname}/dist`,
    filename: 'bundle.js',
  },
  mode: 'development'
};
