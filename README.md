<h1 align="center">
  <a href="https://lapce.dev" target="_blank">
  <img src="extra/images/logo.png" width=200 height=200/><br>
  Lapce
  </a>
</h1>

<h4 align="center">高速でパワフルなコードエディター</h4>

<div align="center">
  <a href="https://github.com/lapce/lapce/actions/workflows/ci.yml" target="_blank">
    <img src="https://github.com/lapce/lapce/actions/workflows/ci.yml/badge.svg" />
  </a>
  <a href="https://discord.gg/n8tGJ6Rn6D" target="_blank">
    <img src="https://img.shields.io/discord/946858761413328946?logo=discord" />
  </a>
  <a href="https://matrix.to/#/#lapce-editor:matrix.org" target="_blank">
    <img src="https://img.shields.io/matrix/lapce-editor:matrix.org?color=turquoise&logo=Matrix" />
  </a>
  <a href="https://docs.lapce.dev" target="_blank">
      <img src="https://img.shields.io/static/v1?label=Docs&message=docs.lapce.dev&color=blue" alt="Lapce Docs">
  </a>
</div>
<br/>


<<<<<<< HEAD
Lapceは純粋なRustで書かれており、UIは [Druid](https://github.com/linebender/druid) (これもRustで作成されています) のUIを使用して純粋なRustで記述されています。 [Rope Science](https://xi-editor.io/docs/rope_science_00.html) の [Xi-Editor](https://github.com/xi-editor/xi-editor) を使って設計されているので、計算が非常に速いです。レンダリングには [OpenGL](https://www.opengl.org/) を利用しています。Lapce の機能についての詳細は [Lapceのウェブサイト](https://lapce.dev) から、ユーザードキュメントは [GitBook](https://docs.lapce.dev/) から閲覧が可能です。
=======
Lapce (IPA: /læps/) is written in pure Rust with a UI in [Druid](https://github.com/linebender/druid) (which is also written in Rust). It is designed with [Rope Science](https://xi-editor.io/docs/rope_science_00.html) from the [Xi-Editor](https://github.com/xi-editor/xi-editor) which makes for lightning-fast computation, and leverages [OpenGL](https://www.opengl.org/) for rendering. More information about the features of Lapce can be found on the [main website](https://lapce.dev) and user documentation can be found on [GitBook](https://docs.lapce.dev/).
>>>>>>> 9845c1a0b3fbfd6afdc0fb1504d3be78764e0c7b

![](https://github.com/lapce/lapce/blob/master/extra/images/screenshot.png?raw=true)

## Features

* Built-in LSP ([Language Server Protocol](https://microsoft.github.io/language-server-protocol/)) support to give you intelligent code features such as: completion, diagnostics and code actions
* Modal editing support as first class citizen (Vim-like, and toggleable)
* Built-in remote development support inspired by [VSCode Remote Development](https://code.visualstudio.com/docs/remote/remote-overview). Enjoy the benefits of a "local" experience, and seamlessly gain the full power of a remote system.
* Plugins can be written in programming languages that can compile to the [WASI](https://wasi.dev/) format (C, Rust, [AssemblyScript](https://www.assemblyscript.org/))
* Built-in terminal, so you can execute commands in your workspace, without leaving Lapce.

## Installation

You can find pre-built releases for Windows, Linux and macOS [here](https://github.com/lapce/lapce/releases), or [installing with a package manager](docs/installing-with-package-manager.md).
If you'd like to compile from source, you can find the [guide](docs/building-from-source.md).

## Contributing

Guidelines for contributing to Lapce can be found in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Feedback & Contact

The most popular place for Lapce developers and users is on the [Discord server](https://discord.gg/n8tGJ6Rn6D).

Or, join the discussion on [Reddit](https://www.reddit.com/r/lapce/) where we are just getting started.

There is also a [Matrix Space](https://matrix.to/#/#lapce-editor:matrix.org), which is linked to the content from the Discord server.

## License

Lapce is released under the Apache License Version 2, which is an open source license. You may contribute to this project, or use the code as you please as long as you adhere to its conditions. You can find a copy of the license text here: [`LICENSE`](LICENSE).
