# loda-rust-web - Development on macOS

This document is intended for the developer that is to develop on the LODA web editor (`loda-rust-web`).

Assumes that the machine runs macOS.

LODA web editor has several dependencies, that all needs to be installed, in order to do development on `loda-rust-web`.

These are the steps to get the development environment working.


## Backup the computer before installing anything

So that it's possible to restore your computer.


## Install "Visual Studio Code"


## Install "Node.js"

What is [Node.js](https://en.wikipedia.org/wiki/Node.js)

```
PROMPT> brew install node
... installs stuff ...
PROMPT> node --version
v18.4.0
PROMPT> npm --version
8.12.1
```


## Install "rollup.js"

What is [rollup.js](https://rollupjs.org/guide/en/)

```
PROMPT> npm install --global rollup
... installs stuff ...
PROMPT> rollup --version
rollup v2.75.6
```


## Install "typescript"

```
PROMPT> npm install -g typescript
... installs stuff ...
PROMPT> tsc --version
Version 4.7.4
```


## Install "ts-node"

```
PROMPT> npm install -g ts-node
... installs stuff ...
PROMPT> ts-node --version
v10.8.1
```


## Install "CodeMirror"

What is [CodeMirror](https://codemirror.net)

[Steps to install codemirror](https://www.npmjs.com/package/codemirror)

```
PROMPT> npm i codemirror
... installs stuff ...
PROMPT> npm info codemirror version
6.0.0
```

