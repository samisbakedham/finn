# finn - ビルド、設定、動作確認

*この文章を他の言語で読む: [English](../build.md), [Español](build_ES.md), [Korean](build_KR.md), [简体中文](build_ZH-CN.md).*

## 動作環境

finnのプログラミング言語である`rust`はほぼ全ての環境に対応している。

現在の動作環境

* Linux x86\_64とmacOS [finn、マイニング、開発]
* Windows 10は未対応 [一部のビルドはできるがマイニングがまだ。助けを募集中!]

## 要件

* rust 1.34+ ([rustup]((https://www.rustup.rs/))を使えば`curl https://sh.rustup.rs -sSf | sh; source $HOME/.cargo/env`でインストール可)
  * rustをインストール済みの場合は`rustup update`を実行
* clang
* ncursesとそのライブラリ (ncurses, ncursesw5)
* zlibとそのライブラリ (zlib1g-dev または zlib-devel)
* pkg-config
* libssl-dev
* linux-headers (Alpine linuxでは必要)
* llvm

Debianベースのディストリビューション(Debian、Ubuntu、Mintなど)ではrustを除き1コマンドでインストールできる:

```sh
apt install build-essential cmake git libgit2-dev clang libncurses5-dev libncursesw5-dev zlib1g-dev pkg-config libssl-dev llvm
```

Mac:

```sh
xcode-select --install
brew install --with-toolchain llvm
brew install pkg-config
brew install openssl
```

## ビルド手順

```sh
git clone https://github.com/mimblewimble/finn.git
cd finn
cargo build --release
```

finnはデバッグモードでもビルド可能(`--release`を付けない状態で、`--debug`か`--verbose`を付ける)。
しかし暗号の計算のオーバーヘッドが大きく、高速同期が著しく遅くなる。

## ビルドエラー

[Troubleshooting](https://github.com/mimblewimble/docs/wiki/Troubleshooting)

## 何がビルドされるか

ビルドの成果物

* `target/release/finn` - finnの実行ファイル

finnのデータ、設定ファイル、ログファイルはデフォルトでは(ホームディレクトリ配下の)`~/.finn`のディレクトリに格納されている。
全ての設定値は`~/.finn/main/finn-server.toml`を編集することで変更可能。

データファイルをカレントディレクトリに出力することも可能。
そのためには以下のコマンドを実行。

```sh
finn server config
```

カレントディレクトリに`finn-server.toml`がある場合、カレントディレクトリにデータが出力される。
finnを`finn-server.toml`を含むディレクトリで起動する場合、デフォルトである`~/.finn/main/finn-server.toml`よりも優先される。

テスト中はfinnのバイナリにpathを通す:

```sh
export PATH=`pwd`/target/release:$PATH
```

ただし、finnをインストールしたルートディレクトリから実行することを想定している。

これにより`finn`が直接実行可能になる(オプションは`finn help`で調べられる)。

## 設定

finnは気の利いたデフォルト設定で起動するようになっており、さらに`finn-server.toml`のファイルを通じて設定可能。
このファイルはfinnの初回起動で作成され、利用可能なオプションに関するドキュメントを含んでいる。

`finn-server.toml`を通じて設定することが推奨されるが、全ての設定はコマンドラインで上書きすることも可能。

コマンドライン関連のヘルプについてはこちらを実行:

```sh
finn help
finn server --help
finn client --help
```

## Docker

```sh
docker build -t finn -f etc/Dockerfile .
```
floonetを使用する場合、代わりに`etc/Dockerfile.floonet`を指定。

コンテナ内で実行する場合、finnのキャッシュをバインドマウントすることも可能。

```sh
docker run -it -d -v $HOME/.finn:/root/.finn finn
```
dockerの名前付きボリュームを使用する場合、代わりに`-v dotfinn:/root/.finn`を指定。
ボリュームが作成される前に、名前付きボリュームがコピーされる。

## クロスプラットフォームビルド

rust(cargo)はあらゆるプラットフォームでビルド可能なので、`finn`をバリデーションノードとして省電力なデバイスで実行することも可能である。
x86のLinux上で`finn`をクロスコンパイルしARMバイナリを作成し、Raspberry Piで実行することも可能。

## finnの使用

機能やトラブルシューティングなどに関するより多くの情報については[Wallet User Guide](https://github.com/mimblewimble/docs/wiki/Wallet-User-Guide)。


## finnのマイニング

finnのマイニングに関する全ての機能は[finn-miner](https://github.com/mimblewimble/finn-miner)と呼ばれるスタンドアローンなパッケージに分離されていることに注意。

finn-minerをfinnノードと通信させるためには、`finn-server.toml`の設定ファイルで`enable_stratum_server = true`と設定し、ウォレットリスナーを起動(`finn-wallet listen`)しておく必要がある。
