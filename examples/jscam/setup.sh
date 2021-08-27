#!/bin/sh

mkdir public
mkdir dist
cd ../../
sh make-npm.sh
cd examples/jscam || return
npm install --save-dev webpack webpack-cli webpack-dev-server
npm install --save ../../nokhwajs
npm run build
