#!/bin/sh

bun build editor-setup.js --outdir=dist --minify

cp flickity.js ./dist/
