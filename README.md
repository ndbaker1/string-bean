<p align="center">
	<h1 align="center">🧵 string-bean</h1>
</p>
<p align="center">A String Art Generation Library</p>

## [🌐 Demo](https://ndbaker1.github.io/string-bean/)

1. Upload an image
2. Preview the greyscale conversion from source image
3. View thread anchor sequence drawn onto canvas
4. Play with input parameters to change output

#### Sample Output
> drawn by me ◕_◕ 

![string-bean-sample](https://github.com/ndbaker1/string-bean/assets/48701178/1eb4b052-7ed6-4d74-a211-e6052b08fa71)

## ⚡ Usage

### 📦 Cargo

You can pull the crate from git and I will maybe publish at some point

```toml
[dependencies]
string-bean = { git = 'https://github.com/ndbaker1/string-bean' }
```

The CLI has some good example usage: https://github.com/ndbaker1/string-bean/blob/2daab678d45f400408a8dbdb39555c916c21ada6/string-bean-cli/src/main.rs#L50-L61
### ⌨️ CLI

Though minimal, the repo includes a CLI that you can install by running:

```shell
cargo install --git https://github.com/ndbaker1/string-bean
```

It currently supports:
* generating thread art for a circular boundary inside an input image and outputing an SVG file

## 🤔 Motivation

A recent youtube video about string art, [The Mathematics of String Art](https://youtu.be/WGccIFf6MF8), showed up in my recommended and was pretty interesting so i decided to try implementing it in 🦀 Rust.  

There are also very few well documented projects for computational string/thread-art, so i figured i would try to make one that was easily accessible and avoided using libraries to do heavy lifting magic

