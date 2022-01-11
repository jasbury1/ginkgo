# Ginkgo Editor

## A (heavily) WIP text editor entirely in Rust

![Logo Image](/images/ginkgoLogo.png)

Ginkgo is how I learned Rust!

It is a spin-off of my previous C++ text editor, [JED](https://github.com/jasbury1/jed), which itself was a spin-off of the popular miniature text editor [Kilo](https://github.com/antirez/kilo)

## Features

Ginkgo supports many of Kilo's features, such as CTRL commands for saving and quitting.
Ginkgo includes Vim features such as normal/insert modes as well as an ever-growing list of keybindings: 

- o/O (open)
- i/I (insert)
- a/A (append)
- h/j/k/l (movement)
- Esc/CTRL^c (exit insert)
- u (undo)
- CTRL^r (redo)

Ginkgo also includes mouse cursor support, including text selections.

![Screenshot Image](/images/screenshot.png)

## The Ginkgo tree

Ginkgo trees are beautiful. But anyone who has ever been around one knows that they have a very "distinct" smell.
Equivalently, this text editor aims to be beautiful and simplistic -- without (hopefully) smelling too bad!