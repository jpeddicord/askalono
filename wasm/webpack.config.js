const path = require('path');

module.exports = {
  entry: './demo.js',
  output: {
    path: `${__dirname}/dist`,
    filename: 'demo.js',
  },
  mode: 'development'
};
